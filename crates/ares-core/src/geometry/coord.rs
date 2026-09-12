use crate::Point2dList;

pub(crate) type Coord = i64;

const NORMAL_SCALE: f64 = 0.000_001;
const LARGE_BED_SCALE: f64 = 0.000_01;
const LARGE_BED_THRESHOLD_MM: f64 = 2_147.0;
const MIN_COORD_QUOTIENT: f64 = i64::MIN as f64;
const MAX_COORD_QUOTIENT_EXCLUSIVE: f64 = -MIN_COORD_QUOTIENT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CoordinateScale {
    Normal,
    LargeBed,
}

impl CoordinateScale {
    pub(crate) fn from_printable_area(printable_area: &Point2dList) -> Self {
        let Some(first) = printable_area.0.first() else {
            return Self::Normal;
        };

        let (min_x, max_x, min_y, max_y) = printable_area.0.iter().skip(1).fold(
            (first.x, first.x, first.y, first.y),
            |(min_x, max_x, min_y, max_y), point| {
                (
                    min_x.min(point.x),
                    max_x.max(point.x),
                    min_y.min(point.y),
                    max_y.max(point.y),
                )
            },
        );
        let span = (max_x - min_x).max(max_y - min_y);
        if span <= LARGE_BED_THRESHOLD_MM {
            Self::Normal
        } else {
            Self::LargeBed
        }
    }

    pub(crate) const fn factor(self) -> f64 {
        match self {
            Self::Normal => NORMAL_SCALE,
            Self::LargeBed => LARGE_BED_SCALE,
        }
    }

    pub(crate) fn checked_scale(self, coordinate: f64) -> Option<Coord> {
        let quotient = coordinate / self.factor();
        if quotient.is_finite()
            && (MIN_COORD_QUOTIENT..MAX_COORD_QUOTIENT_EXCLUSIVE).contains(&quotient)
        {
            Some(quotient as Coord)
        } else {
            None
        }
    }

    /// Like `checked_scale`, but rounding to the nearest lattice unit. Used
    /// where an mm value round-trips back to the integer it came from
    /// (`unscale(int) + offset` loses a unit to a truncating cast).
    pub(crate) fn checked_scale_rounded(self, coordinate: f64) -> Option<Coord> {
        let quotient = coordinate / self.factor();
        if quotient.is_finite()
            && (MIN_COORD_QUOTIENT..MAX_COORD_QUOTIENT_EXCLUSIVE).contains(&quotient)
        {
            Some(quotient.round() as Coord)
        } else {
            None
        }
    }

    pub(crate) fn unscale(self, coordinate: Coord) -> f64 {
        coordinate as f64 * self.factor()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Point {
    x: Coord,
    y: Coord,
}

impl Point {
    pub(crate) const fn new(x: Coord, y: Coord) -> Self {
        Self { x, y }
    }

    pub(crate) const fn x(self) -> Coord {
        self.x
    }

    pub(crate) const fn y(self) -> Coord {
        self.y
    }
}
