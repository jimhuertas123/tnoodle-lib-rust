/// Registry of all WCA puzzle scramblers.
///
/// Port of Java's `PuzzleRegistry` enum. In Rust we use an enum with
/// methods instead of Java's reflection-based `LazySupplier`.
///
/// Each variant will lazily initialize its solver tables on first use.
/// Actual puzzle implementations will be added as they are ported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PuzzleType {
    Two,
    Three,
    Four,
    FourFast,
    Five,
    Six,
    Seven,
    ThreeNi,
    FourNi,
    FiveNi,
    ThreeFm,
    Pyra,
    Sq1,
    Mega,
    Clock,
    Skewb,
}

impl PuzzleType {
    pub const ALL: &[Self] = &[
        Self::Two,
        Self::Three,
        Self::Four,
        Self::FourFast,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::ThreeNi,
        Self::FourNi,
        Self::FiveNi,
        Self::ThreeFm,
        Self::Pyra,
        Self::Sq1,
        Self::Mega,
        Self::Clock,
        Self::Skewb,
    ];

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Two => "222",
            Self::Three => "333",
            Self::Four => "444",
            Self::FourFast => "444fast",
            Self::Five => "555",
            Self::Six => "666",
            Self::Seven => "777",
            Self::ThreeNi => "333ni",
            Self::FourNi => "444ni",
            Self::FiveNi => "555ni",
            Self::ThreeFm => "333fm",
            Self::Pyra => "pyram",
            Self::Sq1 => "sq1",
            Self::Mega => "minx",
            Self::Clock => "clock",
            Self::Skewb => "skewb",
        }
    }

    pub fn long_name(&self) -> &'static str {
        match self {
            Self::Two => "2x2x2",
            Self::Three => "3x3x3",
            Self::Four => "4x4x4",
            Self::FourFast => "4x4x4 (Random Turns)",
            Self::Five => "5x5x5",
            Self::Six => "6x6x6",
            Self::Seven => "7x7x7",
            Self::ThreeNi => "3x3x3 No Inspection",
            Self::FourNi => "4x4x4 No Inspection",
            Self::FiveNi => "5x5x5 No Inspection",
            Self::ThreeFm => "3x3x3 Fewest Moves",
            Self::Pyra => "Pyraminx",
            Self::Sq1 => "Square-1",
            Self::Mega => "Megaminx",
            Self::Clock => "Clock",
            Self::Skewb => "Skewb",
        }
    }
}
