//! M73 progress-profile differential harness (diagnostic-only test tool).
//!
//! Compares two content-identical G-code streams (an OrcaSlicer reference and
//! the Ares output) by aligning their M73 emissions pairwise and checking each
//! emission's elapsed time against the boundaries implied by each stream's own
//! `; total estimated time` header:
//!
//! - `P = int(100 * elapsed / total)` (`GCodeProcessor.cpp:1261-1267`),
//! - `R = time_in_minutes(total - elapsed)` (`GCodeProcessor.cpp:1472-1485`).
//!
//! The Ares side joins by the `g1_times_cache` id: the dump
//! (`ARES_DUMP_ELAPSED`, `<id> <seconds>` per cache entry) is keyed by the
//! same motion-line counter the post-processor uses (`GCodeProcessor.cpp:
//! 1458-1470`: G0/G1 consume one id, G2/G3 one plus their internal segment
//! count — internal counts are not modeled, so arc-bearing streams report
//! mapping errors instead of false divergences).
//!
//! Env-gated like the other parity diagnostics:
//!
//! ```text
//! ARES_M73_PROFILE=1 \
//! ARES_M73_ORCA=/path/orca.gcode \
//! ARES_M73_ARES=/path/ares.gcode \
//! ARES_M73_ELAPSED=/path/cache-dump.txt
//! cargo nextest run -p ares-cli m73_profile --no-capture
//! ```

use std::collections::BTreeMap;

struct Emission {
    p: u32,
    r: u32,
    /// The `g1_times_cache` id the post-processor looked up when emitting.
    id: usize,
}

struct Stream {
    emissions: Vec<Emission>,
    total: f64,
}

fn load_stream(path: &std::path::Path, count_ids: bool) -> Result<Stream, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut total = None;
    let mut emissions = Vec::new();
    let mut id = 0_usize;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("; total estimated time: ") {
            total = Some(parse_dhms(rest)?);
            continue;
        }
        if let Some((_, rest)) = line
            .strip_prefix("; model printing time: ")
            .and_then(|rest| rest.split_once("; total estimated time: "))
        {
            total = Some(parse_dhms(rest)?);
            continue;
        }
        if line.starts_with("M73 ") {
            let (Some(p), Some(r)) = (scan_word(line, 'P'), scan_word(line, 'R')) else {
                continue;
            };
            emissions.push(Emission {
                p: p as u32,
                r: r as u32,
                id,
            });
            continue;
        }
        if count_ids {
            let command = line.split_whitespace().next().unwrap_or_default();
            if matches!(command, "G0" | "G1" | "G2" | "G3" | "G28") {
                id += 1;
            }
        }
    }
    total
        .map(|total| Stream { emissions, total })
        .ok_or_else(|| format!("{}: no total-time header", path.display()))
}

fn parse_dhms(text: &str) -> Result<f64, String> {
    let mut seconds = 0.0_f64;
    for token in text.split_whitespace() {
        let (number, unit) = token.split_at(token.len() - 1);
        let value: f64 = number
            .parse()
            .map_err(|_| format!("bad time token {token:?}"))?;
        seconds += match unit {
            "d" => value * 86_400.0,
            "h" => value * 3_600.0,
            "m" => value * 60.0,
            "s" => value,
            other => return Err(format!("bad time unit {other:?}")),
        };
    }
    Ok(seconds)
}

fn scan_word(line: &str, letter: char) -> Option<f64> {
    line.split_whitespace()
        .find_map(|word| word.strip_prefix(letter)?.parse().ok())
}

fn load_cache(path: &std::path::Path) -> Result<BTreeMap<usize, f64>, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(id), Some(value)) = (parts.next(), parts.next()) else {
            continue;
        };
        if let (Ok(id), Ok(value)) = (id.parse::<usize>(), value.parse::<f64>()) {
            map.insert(id, value);
        }
    }
    Ok(map)
}

fn env_path(name: &str) -> Result<std::path::PathBuf, String> {
    std::env::var_os(name)
        .map(std::path::PathBuf::from)
        .ok_or_else(|| format!("set {name} to the harness inputs"))
}

/// Diagnostic-only: prints the pairwise divergence table. Fails only on
/// missing/unreadable inputs, never on divergence.
#[test]
fn m73_progress_profile_report() {
    if std::env::var("ARES_M73_PROFILE").as_deref() != Ok("1") {
        eprintln!(
            "skipping: set ARES_M73_PROFILE=1 with ARES_M73_ORCA/ARES_M73_ARES/ARES_M73_ELAPSED"
        );
        return;
    }
    let orca = load_stream(&env_path("ARES_M73_ORCA").unwrap(), false)
        .unwrap_or_else(|error| panic!("{error}"));
    let ares = load_stream(&env_path("ARES_M73_ARES").unwrap(), true)
        .unwrap_or_else(|error| panic!("{error}"));
    let cache = load_cache(&env_path("ARES_M73_ELAPSED").unwrap())
        .unwrap_or_else(|error| panic!("{error}"));

    let mut matched = 0;
    let mut out_of_window = 0;
    let mut mapping_errors = 0;
    let mut missing = 0;
    let mut orca_only = 0;
    let mut oc = 0_usize;
    let mut ac = 0_usize;
    while oc < orca.emissions.len() && ac < ares.emissions.len() {
        let (left, right) = (&orca.emissions[oc], &ares.emissions[ac]);
        let order = (left.p, left.r).cmp(&(right.p, right.r));
        match order {
            std::cmp::Ordering::Equal => {
                matched += 1;
                match cache.get(&right.id) {
                    None => missing += 1,
                    Some(&elapsed) => {
                        // Self-check: the pair recomputed from Ares's own
                        // total must reproduce the emitted pair; otherwise
                        // the id mapping (arcs) is unreliable for this
                        // stream and the comparison would lie.
                        let self_p = (100.0 * elapsed / ares.total) as u32;
                        let self_r = ((ares.total - elapsed) / 60.0 + 1.0e-9) as u32;
                        if self_p != right.p || self_r != right.r {
                            mapping_errors += 1;
                        } else {
                            let lo = f64::from(left.p) * orca.total / 100.0;
                            let hi = f64::from(left.p + 1) * orca.total / 100.0;
                            let r_cap = orca.total - f64::from(left.r) * 60.0;
                            if !(elapsed >= lo - 0.01 && elapsed < hi && elapsed <= r_cap) {
                                out_of_window += 1;
                                eprintln!(
                                    "OUT pair {}/{}: orca id {} ares id {} ares_el={:.2} orca_window=[{:.1},{:.1}) r_cap={:.1}",
                                    left.p, left.r, oc, right.id, elapsed, lo, hi, r_cap
                                );
                            }
                        }
                    }
                }
                oc += 1;
                ac += 1;
            }
            std::cmp::Ordering::Less => {
                orca_only += 1;
                oc += 1;
            }
            std::cmp::Ordering::Greater => ac += 1,
        }
    }
    orca_only += orca.emissions.len() - oc;
    eprintln!(
        "matched {matched} (out-of-window {out_of_window}, mapping-errors {mapping_errors}, missing-cache {missing}), orca-only {orca_only}, totals orca {:.0}s ares {:.0}s",
        orca.total, ares.total
    );
}
