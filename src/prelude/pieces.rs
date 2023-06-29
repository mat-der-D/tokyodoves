use strum_macros::{EnumIter, EnumString};

/// Two colors of player, just like black and white in chess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Color {
    Red,
    Green,
}

impl std::ops::Not for Color {
    type Output = Color;
    fn not(self) -> Self::Output {
        use Color::*;
        match self {
            Red => Green,
            Green => Red,
        }
    }
}

/// Six types of doves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumString)]
pub enum Dove {
    /// Represents **B**oss-hato,
    /// which can move to adjacent squares in all eight directions,
    /// just like the King in chess.
    /// A player loses when their boss-hato is completely surrounded.
    /// ```text
    /// +---+---+---+---+---+
    /// |   |   |   |   |   |
    /// +---+---+---+---+---+
    /// |   | * | * | * |   |
    /// +---+---+---+---+---+
    /// |   | * | B | * |   |
    /// +---+---+---+---+---+
    /// |   | * | * | * |   |
    /// +---+---+---+---+---+
    /// |   |   |   |   |   |
    /// +---+---+---+---+---+
    /// ```
    #[strum(ascii_case_insensitive)]
    B,
    /// Represents **A**niki-hato,
    /// which can move to adjacent squares in all eight directions,
    /// just like the King in chess.
    /// ```text
    /// +---+---+---+---+---+
    /// |   |   |   |   |   |
    /// +---+---+---+---+---+
    /// |   | * | * | * |   |
    /// +---+---+---+---+---+
    /// |   | * | A | * |   |
    /// +---+---+---+---+---+
    /// |   | * | * | * |   |
    /// +---+---+---+---+---+
    /// |   |   |   |   |   |
    /// +---+---+---+---+---+
    /// ```
    #[strum(ascii_case_insensitive)]
    A,
    /// Represents **Y**aibato,
    /// which can move to four adjacent squares,
    /// forward, backward or sideways,
    /// just like '+'.
    /// ```text
    /// +---+---+---+---+---+
    /// |   |   |   |   |   |
    /// +---+---+---+---+---+
    /// |   |   | * |   |   |
    /// +---+---+---+---+---+
    /// |   | * | Y | * |   |
    /// +---+---+---+---+---+
    /// |   |   | * |   |   |
    /// +---+---+---+---+---+
    /// |   |   |   |   |   |
    /// +---+---+---+---+---+
    /// ```
    #[strum(ascii_case_insensitive)]
    Y,
    /// Represents **M**amedeppo-bato,
    /// which can move to four diagonally adjacent squares,
    /// just like 'x'.
    /// ```text
    /// +---+---+---+---+---+
    /// |   |   |   |   |   |
    /// +---+---+---+---+---+
    /// |   | * |   | * |   |
    /// +---+---+---+---+---+
    /// |   |   | M |   |   |
    /// +---+---+---+---+---+
    /// |   | * |   | * |   |
    /// +---+---+---+---+---+
    /// |   |   |   |   |   |
    /// +---+---+---+---+---+
    /// ```
    #[strum(ascii_case_insensitive)]
    M,
    /// Represents **T**otsu-hato,
    /// which can move forward, backward or sideways,
    /// through any number of squares,
    /// just like the Rook in chess.
    /// ```text
    /// +---+---+---+---+---+
    /// |   |   | * |   |   |
    /// +---+---+---+---+---+
    /// |   |   | * |   |   |
    /// +---+---+---+---+---+
    /// | * | * | T | * | * |
    /// +---+---+---+---+---+
    /// |   |   | * |   |   |
    /// +---+---+---+---+---+
    /// |   |   | * |   |   |
    /// +---+---+---+---+---+
    /// ```
    #[strum(ascii_case_insensitive)]
    T,
    /// Represents **H**ajike-hato,
    /// which can move (or jump) like the Knight in chess.
    /// ```text
    /// +---+---+---+---+---+
    /// |   | * |   | * |   |
    /// +---+---+---+---+---+
    /// | * |   |   |   | * |
    /// +---+---+---+---+---+
    /// |   |   | H |   |   |
    /// +---+---+---+---+---+
    /// | * |   |   |   | * |
    /// +---+---+---+---+---+
    /// |   | * |   | * |   |
    /// +---+---+---+---+---+
    /// ```
    #[strum(ascii_case_insensitive)]
    H,
}

#[inline]
pub(crate) fn color_to_index(color: Color) -> usize {
    use Color::*;
    match color {
        Red => 0,
        Green => 1,
    }
}

#[inline]
pub(crate) fn dove_to_index(dove: Dove) -> usize {
    use Dove::*;
    match dove {
        B => 0,
        A => 1,
        Y => 2,
        M => 3,
        T => 4,
        H => 5,
    }
}

#[inline]
pub(crate) fn color_dove_to_char(color: Color, dove: Dove) -> char {
    use Color::*;
    use Dove::*;
    match (color, dove) {
        (Red, B) => 'B',
        (Green, B) => 'b',
        (Red, A) => 'A',
        (Green, A) => 'a',
        (Red, Y) => 'Y',
        (Green, Y) => 'y',
        (Red, M) => 'M',
        (Green, M) => 'm',
        (Red, T) => 'T',
        (Green, T) => 't',
        (Red, H) => 'H',
        (Green, H) => 'h',
    }
}

#[inline]
pub(crate) fn try_char_to_color_dove(c: char) -> Option<(Color, Dove)> {
    use Color::*;
    use Dove::*;
    let color_dove = match c {
        'B' => (Red, B),
        'b' => (Green, B),
        'A' => (Red, A),
        'a' => (Green, A),
        'Y' => (Red, Y),
        'y' => (Green, Y),
        'M' => (Red, M),
        'm' => (Green, M),
        'T' => (Red, T),
        't' => (Green, T),
        'H' => (Red, H),
        'h' => (Green, H),
        _ => return None,
    };
    Some(color_dove)
}
