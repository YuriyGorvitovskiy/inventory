#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Length {
    Bounded(usize),
    Unbounded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Boolean,
    BigInt,
    DoublePrecision,
    TimestampWithTimeZone,
    VarChar850,
    #[cfg_attr(not(test), allow(dead_code))]
    VarChar64,
    Text,
}

#[cfg_attr(not(test), allow(dead_code))]
impl DataType {
    pub const VARCHAR_850_LENGTH: Length = Length::Bounded(850);
    pub const VARCHAR_64_LENGTH: Length = Length::Bounded(64);
    pub const BOOLEAN_LENGTH: Length = Length::Bounded(1);
    pub const BIGINT_LENGTH: Length = Length::Bounded(8);
    pub const DOUBLE_PRECISION_LENGTH: Length = Length::Bounded(8);
    pub const TIMESTAMP_WITH_TIME_ZONE_LENGTH: Length = Length::Bounded(8);
    pub const TEXT_LENGTH: Length = Length::Unbounded;

    pub fn lenght(&self) -> Length {
        match self {
            Self::Boolean => Self::BOOLEAN_LENGTH,
            Self::BigInt => Self::BIGINT_LENGTH,
            Self::DoublePrecision => Self::DOUBLE_PRECISION_LENGTH,
            Self::TimestampWithTimeZone => Self::TIMESTAMP_WITH_TIME_ZONE_LENGTH,
            Self::VarChar850 => Self::VARCHAR_850_LENGTH,
            Self::VarChar64 => Self::VARCHAR_64_LENGTH,
            Self::Text => Self::TEXT_LENGTH,
        }
    }

    pub fn sql(&self) -> &'static str {
        match self {
            Self::Boolean => "BOOLEAN",
            Self::BigInt => "BIGINT",
            Self::DoublePrecision => "DOUBLE PRECISION",
            Self::TimestampWithTimeZone => "TIMESTAMPTZ",
            Self::VarChar850 => "VARCHAR(850)",
            Self::VarChar64 => "VARCHAR(64)",
            Self::Text => "TEXT",
        }
    }
}
