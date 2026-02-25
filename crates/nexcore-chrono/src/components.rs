//! Date/time component structs and day-of-week enum.

/// Components of a date-time value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct DateTimeComponents {
    pub year: i32,
    /// Month (1-12).
    pub month: u32,
    /// Day (1-31).
    pub day: u32,
    /// Hour (0-23).
    pub hour: u32,
    /// Minute (0-59).
    pub minute: u32,
    /// Second (0-59).
    pub second: u32,
    /// Microsecond (0-999999).
    pub microsecond: u32,
}

/// Components of a date value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct DateComponents {
    pub year: i32,
    /// Month (1-12).
    pub month: u32,
    /// Day (1-31).
    pub day: u32,
}

/// Day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DayOfWeek {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}

impl DayOfWeek {
    /// Convert from a number (0=Monday through 6=Sunday).
    #[must_use]
    pub const fn from_number(n: u32) -> Option<Self> {
        match n {
            0 => Some(Self::Monday),
            1 => Some(Self::Tuesday),
            2 => Some(Self::Wednesday),
            3 => Some(Self::Thursday),
            4 => Some(Self::Friday),
            5 => Some(Self::Saturday),
            6 => Some(Self::Sunday),
            _ => None,
        }
    }

    /// Returns the day number (0=Monday through 6=Sunday).
    #[must_use]
    pub const fn number(&self) -> u32 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }
}

impl core::fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        };
        write!(f, "{name}")
    }
}
