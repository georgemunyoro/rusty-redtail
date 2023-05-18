use std::ops::Not;

/// Represents the color of a piece
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Color::White => "White",
                Color::Black => "Black",
            }
        )
    }
}

impl From<Color> for usize {
    fn from(v: Color) -> Self {
        match v {
            Color::White => 0,
            Color::Black => 1,
        }
    }
}

impl From<char> for Color {
    fn from(v: char) -> Self {
        match v {
            'w' => Color::White,
            'b' => Color::Black,
            _ => panic!("Invalid color"),
        }
    }
}
