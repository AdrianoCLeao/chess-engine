//! Core chess types shared across the engine.
//!
//! All board coordinates use a 0–63 square index with a1 = 0 and h8 = 63
//! (little-endian rank-file mapping). Files are a–h (0–7) and ranks are 1–8 (0–7).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Side to move / piece colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline]
    pub fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    #[inline]
    pub fn sign(self) -> i32 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

/// Piece kinds, ordered for compact indexing into tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'p' => Some(PieceKind::Pawn),
            'n' => Some(PieceKind::Knight),
            'b' => Some(PieceKind::Bishop),
            'r' => Some(PieceKind::Rook),
            'q' => Some(PieceKind::Queen),
            'k' => Some(PieceKind::King),
            _ => None,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        }
    }

    /// Midgame material value in centipawns.
    pub fn value_cp(self) -> i32 {
        match self {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 320,
            PieceKind::Bishop => 330,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 20_000,
        }
    }
}

/// A coloured piece on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    pub fn new(color: Color, kind: PieceKind) -> Self {
        Self { color, kind }
    }

    /// FEN / algebraic character (uppercase = white).
    pub fn to_fen_char(self) -> char {
        let c = self.kind.to_char();
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    pub fn from_fen_char(c: char) -> Option<Self> {
        let kind = PieceKind::from_char(c)?;
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        Some(Self { color, kind })
    }
}

/// Square index 0..64 (a1 = 0).
pub type Square = u8;

pub fn square_file(sq: Square) -> u8 {
    sq % 8
}

pub fn square_rank(sq: Square) -> u8 {
    sq / 8
}

pub fn square_from(file: u8, rank: u8) -> Square {
    rank * 8 + file
}

pub fn square_name(sq: Square) -> String {
    let file = (b'a' + square_file(sq)) as char;
    let rank = (b'1' + square_rank(sq)) as char;
    format!("{file}{rank}")
}

pub fn square_from_name(name: &str) -> Option<Square> {
    let bytes = name.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let file = bytes[0].to_ascii_lowercase();
    let rank = bytes[1];
    if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
        return None;
    }
    Some(square_from(file - b'a', rank - b'1'))
}

/// Special move flags packed into a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MoveFlags {
    pub promotion: Option<PieceKind>,
    pub is_castle: bool,
    pub is_en_passant: bool,
    pub is_double_pawn: bool,
}

/// A pseudo-legal or legal chess move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub flags: MoveFlags,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            flags: MoveFlags::default(),
        }
    }

    pub fn with_promotion(from: Square, to: Square, kind: PieceKind) -> Self {
        Self {
            from,
            to,
            flags: MoveFlags {
                promotion: Some(kind),
                ..Default::default()
            },
        }
    }

    /// UCI-style string such as `e2e4` or `e7e8q`.
    pub fn to_uci(self) -> String {
        let mut s = format!("{}{}", square_name(self.from), square_name(self.to));
        if let Some(promo) = self.flags.promotion {
            s.push(promo.to_char());
        }
        s
    }

    pub fn from_uci(uci: &str) -> Option<Self> {
        if uci.len() < 4 {
            return None;
        }
        let from = square_from_name(&uci[0..2])?;
        let to = square_from_name(&uci[2..4])?;
        let promotion = if uci.len() >= 5 {
            PieceKind::from_char(uci.chars().nth(4)?)
        } else {
            None
        };
        Some(Self {
            from,
            to,
            flags: MoveFlags {
                promotion,
                ..Default::default()
            },
        })
    }
}

/// Terminal game outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameResult {
    Ongoing,
    WhiteWins,
    BlackWins,
    Draw,
}

/// Chess.com-style qualitative move labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveQuality {
    Brilliant,
    Great,
    Best,
    Excellent,
    Good,
    Book,
    Inaccuracy,
    Mistake,
    Miss,
    Blunder,
}

impl MoveQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            MoveQuality::Brilliant => "brilliant",
            MoveQuality::Great => "great",
            MoveQuality::Best => "best",
            MoveQuality::Excellent => "excellent",
            MoveQuality::Good => "good",
            MoveQuality::Book => "book",
            MoveQuality::Inaccuracy => "inaccuracy",
            MoveQuality::Mistake => "mistake",
            MoveQuality::Miss => "miss",
            MoveQuality::Blunder => "blunder",
        }
    }
}

/// Castling rights bit flags: white kingside/queenside, black kingside/queenside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CastlingRights {
    pub white_king: bool,
    pub white_queen: bool,
    pub black_king: bool,
    pub black_queen: bool,
}

impl CastlingRights {
    pub fn start() -> Self {
        Self {
            white_king: true,
            white_queen: true,
            black_king: true,
            black_queen: true,
        }
    }
}
