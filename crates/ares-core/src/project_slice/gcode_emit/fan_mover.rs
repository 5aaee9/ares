//! Port of `GCode/FanMover.cpp` — the fan-speedup post-pass that delays
//! fan-up commands into the recent past (a time-buffered g-code rewriter).
//! Active only when `fan_speedup_time != 0 || fan_kickstart > 0`
//! (`GCode.cpp:3731-3740`).

use crate::{ExtrusionRole, GCodeFlavor};

#[derive(Clone, Debug, PartialEq)]
struct BufferData {
    raw: String,
    time: f32,
    fan_speed: i16,
    is_kickstart: bool,
    /// End coordinates / deltas of a G0/G1 line (`-1` when the word is
    /// absent, mirroring the reader's NaN-free sentinel usage via `has`).
    x: f32,
    y: f32,
    z: f32,
    e: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    de: f32,
}

impl BufferData {
    fn new(raw: String, time: f32, fan_speed: i16, is_kickstart: bool) -> Self {
        Self {
            raw,
            time,
            fan_speed,
            is_kickstart,
            x: -1.0,
            y: -1.0,
            z: -1.0,
            e: -1.0,
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
            de: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
struct Kickstart {
    fan_speed: i16,
    time: f32,
}

/// The position/speed state FanMover needs to track per line — filled by
/// the caller's line walk (upstream reads these off the GCodeReader).
#[derive(Clone, Copy, Default)]
pub(crate) struct LineMotion {
    pub(crate) has_f: bool,
    pub(crate) f_mm_s: f32,
    pub(crate) dist: f32,
    pub(crate) x: Option<f32>,
    pub(crate) y: Option<f32>,
    pub(crate) z: Option<f32>,
    pub(crate) e: Option<f32>,
    pub(crate) dx: f32,
    pub(crate) dy: f32,
    pub(crate) dz: f32,
    pub(crate) de: f32,
}

pub(crate) struct FanMover {
    nb_seconds_delay: f32,
    only_overhangs: bool,
    kickstart: f32,
    relative_e: bool,
    flavor: GCodeFlavor,
    buffer: Vec<BufferData>,
    buffer_time_size: f32,
    front_buffer_fan_speed: i16,
    back_buffer_fan_speed: i16,
    current_kickstart: Option<Kickstart>,
    current_kickstart_raw: String,
    current_speed: f32,
    is_custom_gcode: bool,
    current_role: ExtrusionRole,
    output: String,
    /// Tracked XYZ position (for move distances) and E (for absolute-E
    /// deltas), mirroring the upstream GCodeReader state.
    position: [f32; 3],
    e_position: f32,
    relative_xyz: bool,
}

impl FanMover {
    pub(crate) fn new(
        fan_speedup_time: f64,
        fan_kickstart: f64,
        only_overhangs: bool,
        relative_e: bool,
        flavor: GCodeFlavor,
    ) -> Self {
        let delay = if fan_speedup_time > 0.0 {
            (fan_speedup_time as f32).max(0.01)
        } else {
            0.0
        };
        Self {
            nb_seconds_delay: delay,
            only_overhangs,
            kickstart: fan_kickstart as f32,
            relative_e,
            flavor,
            buffer: Vec::new(),
            buffer_time_size: 0.0,
            front_buffer_fan_speed: -1,
            back_buffer_fan_speed: -1,
            current_kickstart: None,
            current_kickstart_raw: String::new(),
            current_speed: 0.0,
            is_custom_gcode: false,
            current_role: ExtrusionRole::None,
            output: String::new(),
            position: [0.0; 3],
            e_position: 0.0,
            relative_xyz: false,
        }
    }

    pub(crate) fn process_gcode(&mut self, gcode: &str, flush: bool) -> String {
        self.output.clear();
        self.buffer_time_size = 0.0;
        for data in &self.buffer {
            self.buffer_time_size += data.time;
        }
        for line in gcode.lines() {
            self.process_line(line);
        }
        if flush {
            while let Some(front) = self.buffer.first().cloned() {
                if front.fan_speed >= 0 {
                    self.front_buffer_fan_speed = front.fan_speed;
                }
                self.output.push_str(&front.raw);
                self.output.push('\n');
                self.buffer.remove(0);
            }
        }
        std::mem::take(&mut self.output)
    }

    fn process_line(&mut self, raw: &str) {
        let mut need_flush = false;
        let mut time: f32 = 0.0;
        let mut fan_speed: i16 = -1;
        let mut motion = LineMotion::default();
        let code = raw.split(';').next().unwrap_or_default().trim();
        let command = code.split_whitespace().next().unwrap_or_default();
        if command.len() > 1 {
            motion = parse_motion(code);
            if motion.has_f {
                self.current_speed = motion.f_mm_s;
            }
            match command.as_bytes()[0] {
                b'T' => need_flush = true,
                b'G' => {
                    let number: i32 = command[1..].parse().unwrap_or(-1);
                    if number == 0 || number == 1 {
                        let (dx, dy, dz) = (
                            motion.x.map_or(0.0, |value| {
                                if self.relative_xyz {
                                    value
                                } else {
                                    value - self.position[0]
                                }
                            }),
                            motion.y.map_or(0.0, |value| {
                                if self.relative_xyz {
                                    value
                                } else {
                                    value - self.position[1]
                                }
                            }),
                            motion.z.map_or(0.0, |value| {
                                if self.relative_xyz {
                                    value
                                } else {
                                    value - self.position[2]
                                }
                            }),
                        );
                        motion.dx = dx;
                        motion.dy = dy;
                        motion.dz = dz;
                        let dist_sq = dx * dx + dy * dy + dz * dz;
                        if dist_sq > 0.0 {
                            motion.dist = dist_sq.sqrt();
                            if self.current_speed > 0.0 {
                                time = motion.dist / self.current_speed;
                            }
                        }
                    } else if number == 90 {
                        self.relative_xyz = false;
                    } else if number == 91 {
                        self.relative_xyz = true;
                    }
                }
                b'M' => {
                    fan_speed = read_fan_speed(code, self.flavor);
                    if fan_speed >= 0 {
                        self.current_kickstart = None;
                        if !self.is_custom_gcode {
                            self.process_fan_command(raw, fan_speed, &mut time);
                        } else {
                            need_flush = true;
                        }
                        self.back_buffer_fan_speed = fan_speed;
                    }
                }
                _ => {}
            }
        } else if let Some(comment) = raw.strip_prefix(';') {
            if let Some(role) = comment.strip_prefix("TYPE:") {
                self.current_role = role_from_str(role);
            }
            if raw.contains("; custom gcode") {
                self.is_custom_gcode = !raw.contains("; custom gcode end");
            }
        }

        if time >= 0.0 {
            if let Some(x) = motion.x {
                self.position[0] = if self.relative_xyz {
                    self.position[0] + x
                } else {
                    x
                };
            }
            if let Some(y) = motion.y {
                self.position[1] = if self.relative_xyz {
                    self.position[1] + y
                } else {
                    y
                };
            }
            if let Some(z) = motion.z {
                self.position[2] = if self.relative_xyz {
                    self.position[2] + z
                } else {
                    z
                };
            }
            if let Some(e) = motion.e {
                if !self.relative_e {
                    motion.de = e - self.e_position;
                }
                self.e_position = if self.relative_e {
                    self.e_position + e
                } else {
                    e
                };
            }
            self.push_buffer(
                BufferData::new(raw.to_owned(), time, fan_speed, false),
                &motion,
            );
            if let Some(mut kick) = self.current_kickstart {
                if time > 0.0 {
                    kick.time -= time;
                    if kick.time < 0.0 {
                        let split = time + kick.time;
                        let data = BufferData::new(
                            std::mem::take(&mut self.current_kickstart_raw),
                            0.0,
                            kick.fan_speed,
                            true,
                        );
                        self.put_in_middle(self.buffer.len() - 1, split, data);
                    }
                    self.current_kickstart = (kick.time >= 0.0).then_some(kick);
                }
            }
        }

        while !self.buffer.is_empty()
            && (need_flush
                || self.nb_seconds_delay <= f32::EPSILON
                || self.buffer_time_size - self.buffer[0].time
                    > self.nb_seconds_delay - f32::EPSILON)
        {
            let front = self.buffer.remove(0);
            self.buffer_time_size -= front.time;
            if front.fan_speed < 0
                || front.fan_speed != self.front_buffer_fan_speed
                || front.is_kickstart
            {
                if front.is_kickstart && front.fan_speed < self.front_buffer_fan_speed {
                    self.output
                        .push_str(&format!("M106 S{}\n", fan_pwm(front.fan_speed)));
                    self.front_buffer_fan_speed = front.fan_speed;
                } else {
                    self.output.push_str(&front.raw);
                    self.output.push('\n');
                    if front.fan_speed >= 0 {
                        self.front_buffer_fan_speed = front.fan_speed;
                    }
                }
            }
        }
    }

    fn process_fan_command(&mut self, raw: &str, fan_speed: i16, time: &mut f32) {
        if self.back_buffer_fan_speed >= fan_speed {
            return;
        }
        if self.nb_seconds_delay > 0.0
            && (!self.only_overhangs || self.current_role == ExtrusionRole::OverhangPerimeter)
        {
            *time = -1.0;
            if self.kickstart > 0.0 && fan_speed > self.front_buffer_fan_speed {
                self.current_kickstart = None;
                self.remove_slow_fan(fan_speed, self.buffer_time_size + 1.0);
                self.remove_slow_fan(255, self.kickstart);
                if !self.buffer.is_empty()
                    && (self.buffer_time_size - self.buffer[0].time * 0.1) > self.nb_seconds_delay
                {
                    self.print_in_middle(
                        0,
                        self.buffer_time_size - self.nb_seconds_delay,
                        &set_fan_line(100),
                    );
                    self.buffer.remove(0);
                } else {
                    self.output.push_str(&set_fan_line(100));
                    self.output.push('\n');
                }
                let kickstart_duration =
                    self.kickstart * f32::from(fan_speed - self.front_buffer_fan_speed) / 100.0;
                let mut time_count = kickstart_duration;
                let mut index = 0;
                while index < self.buffer.len() && time_count > 0.0 {
                    time_count -= self.buffer[index].time;
                    if time_count < 0.0 {
                        let data = BufferData::new(raw.to_owned(), 0.0, fan_speed, true);
                        self.put_in_middle(index, self.buffer[index].time + time_count, data);
                        break;
                    }
                    index += 1;
                }
                if time_count > 0.0 {
                    self.current_kickstart = Some(Kickstart {
                        fan_speed,
                        time: time_count,
                    });
                    self.current_kickstart_raw = raw.to_owned();
                }
                self.front_buffer_fan_speed = fan_speed;
            } else {
                self.remove_slow_fan(fan_speed, self.buffer_time_size + 1.0);
                if !self.buffer.is_empty()
                    && (self.buffer_time_size - self.buffer[0].time * 0.1) > self.nb_seconds_delay
                {
                    self.print_in_middle(0, self.buffer_time_size - self.nb_seconds_delay, raw);
                    self.buffer.remove(0);
                } else {
                    self.output.push_str(raw);
                    self.output.push('\n');
                }
                self.front_buffer_fan_speed = fan_speed;
            }
        } else if self.kickstart <= 0.0 {
            // Nothing to do — printed in the buffer as other lines are.
        } else if self.current_kickstart.is_some() {
            if let Some(kick) = &mut self.current_kickstart {
                if self.back_buffer_fan_speed >= fan_speed {
                    self.current_kickstart = None;
                } else {
                    let kickstart_duration =
                        self.kickstart * f32::from(fan_speed - self.back_buffer_fan_speed) / 100.0;
                    kick.fan_speed = fan_speed;
                    kick.time += kickstart_duration;
                    self.current_kickstart = Some(*kick);
                    self.current_kickstart_raw = raw.to_owned();
                    *time = -1.0;
                }
            }
        } else if self.back_buffer_fan_speed < fan_speed - 10 {
            *time = -1.0;
            let kickstart_duration =
                self.kickstart * f32::from(fan_speed - self.back_buffer_fan_speed) / 100.0;
            self.push_buffer(
                BufferData::new(set_fan_line(100), 0.0, fan_speed, true),
                &LineMotion::default(),
            );
            self.current_kickstart = Some(Kickstart {
                fan_speed,
                time: kickstart_duration,
            });
            self.current_kickstart_raw = raw.to_owned();
        }
    }

    fn push_buffer(&mut self, data: BufferData, motion: &LineMotion) {
        self.buffer_time_size += data.time;
        let mut data = data;
        if let Some(x) = motion.x {
            data.x = x;
            data.dx = motion.dx;
        }
        if let Some(y) = motion.y {
            data.y = y;
            data.dy = motion.dy;
        }
        if let Some(z) = motion.z {
            data.z = z;
            data.dz = motion.dz;
        }
        if let Some(e) = motion.e {
            data.e = e;
            data.de = motion.de;
        }
        self.buffer.push(data);
    }

    /// `_put_in_middle_G1` (FanMover.cpp:124-140): split the buffered line
    /// at `nb_sec` from its start, inserting the fan line between.
    fn put_in_middle(&mut self, index: usize, nb_sec: f32, line: BufferData) {
        let item_time = self.buffer[index].time;
        if nb_sec > item_time * 0.9 {
            self.buffer.insert(index + 1, line);
        } else if nb_sec < item_time * 0.1 || item_time == 0.0 {
            self.buffer.insert(index, line);
        } else {
            let percent = nb_sec / item_time;
            let (before_raw, after_raw) =
                split_line(&mut self.buffer[index], percent, self.relative_e);
            let before = BufferData::new(before_raw, nb_sec, -1, false);
            let after_time = item_time - nb_sec;
            self.buffer[index].raw = after_raw;
            self.buffer[index].time = after_time;
            self.buffer[index].dx *= 1.0 - percent;
            self.buffer[index].dy *= 1.0 - percent;
            self.buffer[index].dz *= 1.0 - percent;
            self.buffer[index].de *= 1.0 - percent;
            self.buffer.insert(index, line);
            self.buffer.insert(index, before);
            self.buffer_time_size += 0.0;
        }
    }

    /// `_print_in_middle_G1` (FanMover.cpp:171-212): flush the front line
    /// (optionally split) with the fan command inside.
    fn print_in_middle(&mut self, index: usize, nb_sec: f32, fan_line: &str) {
        let item = self.buffer.remove(index);
        self.buffer_time_size -= item.time;
        if nb_sec < item.time * 0.1 {
            self.output.push_str(&item.raw);
            self.output.push('\n');
            self.output.push_str(fan_line);
            self.output.push('\n');
        } else if nb_sec > item.time * 0.9 || !item.raw.starts_with("G1 ") {
            self.output.push_str(fan_line);
            self.output.push('\n');
            self.output.push_str(&item.raw);
            self.output.push('\n');
        } else {
            let percent = nb_sec / item.time;
            let mut item = item;
            let (before, after) = split_line(&mut item, percent, self.relative_e);
            self.output.push_str(&before);
            self.output.push('\n');
            self.output.push_str(fan_line);
            self.output.push('\n');
            self.output.push_str(&after);
            self.output.push('\n');
        }
    }

    /// `_remove_slow_fan` (FanMover.cpp:214-227).
    fn remove_slow_fan(&mut self, min_speed: i16, mut past_sec: f32) {
        let mut index = 0;
        while index < self.buffer.len() && past_sec > 0.0 {
            past_sec -= self.buffer[index].time;
            if self.buffer[index].fan_speed >= 0 && self.buffer[index].fan_speed < min_speed {
                let removed = self.buffer.remove(index);
                self.buffer_time_size -= removed.time;
            } else {
                index += 1;
            }
        }
    }
}

fn set_fan_line(percent: i16) -> String {
    format!("M106 S{}", fan_pwm(percent))
}

fn fan_pwm(percent: i16) -> u32 {
    (255.5 * f32::from(percent) / 100.0) as u32
}

fn read_fan_speed(code: &str, flavor: GCodeFlavor) -> i16 {
    let command = code.split_whitespace().next().unwrap_or_default();
    if command == "M106" {
        let p = word(code, 'P');
        if let Some(p) = p {
            if flavor != GCodeFlavor::Mach3 && flavor != GCodeFlavor::Machinekit && p != 1.0 {
                return -1;
            }
        }
        word(code, 'S').map_or(-1, |s| (100.0 * s / 255.0) as i16)
    } else if command == "M127" || command == "M107" {
        0
    } else if command == "M126"
        && (flavor == GCodeFlavor::MakerWare || flavor == GCodeFlavor::Sailfish)
    {
        word(code, 'T').map_or(-1, |t| (100.0 * t / 255.0) as i16)
    } else {
        -1
    }
}

fn word(code: &str, letter: char) -> Option<f32> {
    code.split_whitespace().find_map(|token| {
        let mut characters = token.chars();
        let first = characters.next()?;
        (first.eq_ignore_ascii_case(&letter))
            .then(|| characters.as_str().parse::<f32>().ok())
            .flatten()
    })
}

fn parse_motion(code: &str) -> LineMotion {
    let mut motion = LineMotion::default();
    let mut x = None;
    let mut y = None;
    let mut z = None;
    let mut e = None;
    for token in code.split_whitespace().skip(1) {
        let mut characters = token.chars();
        let Some(first) = characters.next() else {
            continue;
        };
        let Ok(value) = characters.as_str().parse::<f32>() else {
            continue;
        };
        match first {
            'F' | 'f' => {
                motion.has_f = true;
                motion.f_mm_s = value / 60.0;
            }
            'X' | 'x' => x = Some(value),
            'Y' | 'y' => y = Some(value),
            'Z' | 'z' => z = Some(value),
            'E' | 'e' => e = Some(value),
            _ => {}
        }
    }
    motion.x = x;
    motion.y = y;
    motion.z = z;
    motion.e = e;
    motion
}

/// Splits a buffered G0/G1 line at `percent`, rewriting the axis words of
/// both halves (`change_axis_value`, FanMover.cpp:74-89). Returns
/// `(before, after)`.
fn split_line(data: &mut BufferData, percent: f32, relative_e: bool) -> (String, String) {
    let mut before = data.raw.clone();
    if data.dx != 0.0 {
        before = replace_word(&before, 'X', data.x + data.dx * percent, 3);
    }
    if data.dy != 0.0 {
        before = replace_word(&before, 'Y', data.y + data.dy * percent, 3);
    }
    if data.dz != 0.0 {
        before = replace_word(&before, 'Z', data.z + data.dz * percent, 3);
    }
    let mut after = data.raw.clone();
    if data.de != 0.0 {
        if relative_e {
            before = replace_word(&before, 'E', data.de * percent, 5);
            after = replace_word(&after, 'E', data.de * (1.0 - percent), 5);
        } else {
            before = replace_word(&before, 'E', data.e + data.de * percent, 5);
        }
    }
    (before, after)
}

fn replace_word(line: &str, axis: char, value: f32, digits: usize) -> String {
    let mut result = String::with_capacity(line.len() + 8);
    let code = line.split(';').next().unwrap_or(line);
    let comment = &line[code.len()..];
    let mut replaced = false;
    for token in code.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        if !replaced {
            let mut characters = token.chars();
            if let Some(first) = characters.next() {
                if first.eq_ignore_ascii_case(&axis) && characters.as_str().parse::<f32>().is_ok() {
                    result.push(first);
                    result.push_str(&format!("{value:.digits$}"));
                    replaced = true;
                    continue;
                }
            }
        }
        result.push_str(token);
    }
    result.push_str(comment);
    result
}

fn role_from_str(role: &str) -> ExtrusionRole {
    match role.trim() {
        "Outer wall" => ExtrusionRole::ExternalPerimeter,
        "Inner wall" => ExtrusionRole::Perimeter,
        "Overhang wall" => ExtrusionRole::OverhangPerimeter,
        _ => ExtrusionRole::None,
    }
}

#[cfg(test)]
mod corpus;
#[cfg(test)]
#[cfg(test)]
mod tests;
