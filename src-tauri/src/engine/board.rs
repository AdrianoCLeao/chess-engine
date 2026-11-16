//! Board representation, FEN I/O, and make/unmake move.

use super::types::*;

/// Snapshot used to undo a move cheaply during search.
#[derive(Debug, Clone, Copy)]
pub struct UndoInfo {
    pub captured: Option<Piece>,
    pub castling: CastlingRights,
    pub ep_square: Option<Square>,
    pub halfmove: u32,
    pub fullmove: u32,
    pub moved_piece: Piece,
}

/// Full chess position.
///
/// Not Serde-serialized as a raw 64-array (serde's default array support is
/// capped); the UI receives occupancy via [`Board::occupancy_chars`] / FEN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// Square occupancy; index 0 = a1.
    pub squares: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub ep_square: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Default for Board {
    fn default() -> Self {
        Self::startpos()
    }
}

impl Board {
    pub fn empty() -> Self {
        Self {
            squares: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::default(),
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn startpos() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("startpos FEN must parse")
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        self.squares[sq as usize]
    }

    pub fn set_piece(&mut self, sq: Square, piece: Option<Piece>) {
        self.squares[sq as usize] = piece;
    }

    pub fn find_king(&self, color: Color) -> Option<Square> {
        self.squares.iter().enumerate().find_map(|(i, p)| {
            if let Some(piece) = p {
                if piece.color == color && piece.kind == PieceKind::King {
                    return Some(i as Square);
                }
            }
            None
        })
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("FEN must have at least 4 fields".into());
        }

        let mut board = Self::empty();
        let mut rank: i32 = 7;
        let mut file: i32 = 0;

        for ch in parts[0].chars() {
            match ch {
                '/' => {
                    rank -= 1;
                    file = 0;
                    if rank < 0 {
                        return Err("too many ranks in FEN".into());
                    }
                }
                '1'..='8' => {
                    file += (ch as u8 - b'0') as i32;
                    if file > 8 {
                        return Err("file overflow in FEN".into());
                    }
                }
                c => {
                    let piece = Piece::from_fen_char(c)
                        .ok_or_else(|| format!("invalid piece '{c}' in FEN"))?;
                    if !(0..8).contains(&file) || !(0..8).contains(&rank) {
                        return Err("square out of range in FEN".into());
                    }
                    board.set_piece(square_from(file as u8, rank as u8), Some(piece));
                    file += 1;
                }
            }
        }

        board.side_to_move = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            other => return Err(format!("invalid side to move '{other}'")),
        };

        let mut castling = CastlingRights::default();
        if parts[2] != "-" {
            for c in parts[2].chars() {
                match c {
                    'K' => castling.white_king = true,
                    'Q' => castling.white_queen = true,
                    'k' => castling.black_king = true,
                    'q' => castling.black_queen = true,
                    _ => return Err(format!("invalid castling right '{c}'")),
                }
            }
        }
        board.castling = castling;

        board.ep_square = if parts[3] == "-" {
            None
        } else {
            Some(
                square_from_name(parts[3])
                    .ok_or_else(|| format!("invalid en passant square '{}'", parts[3]))?,
            )
        };

        board.halfmove_clock = parts
            .get(4)
            .unwrap_or(&"0")
            .parse()
            .map_err(|_| "invalid halfmove clock")?;
        board.fullmove_number = parts
            .get(5)
            .unwrap_or(&"1")
            .parse()
            .map_err(|_| "invalid fullmove number")?;

        Ok(board)
    }

    pub fn to_fen(&self) -> String {
        let mut placement = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let sq = square_from(file, rank);
                match self.piece_at(sq) {
                    None => empty += 1,
                    Some(piece) => {
                        if empty > 0 {
                            placement.push(char::from(b'0' + empty));
                            empty = 0;
                        }
                        placement.push(piece.to_fen_char());
                    }
                }
            }
            if empty > 0 {
                placement.push(char::from(b'0' + empty));
            }
            if rank > 0 {
                placement.push('/');
            }
        }

        let stm = match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        };

        let mut castling = String::new();
        if self.castling.white_king {
            castling.push('K');
        }
        if self.castling.white_queen {
            castling.push('Q');
        }
        if self.castling.black_king {
            castling.push('k');
        }
        if self.castling.black_queen {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }

        let ep = self
            .ep_square
            .map(square_name)
            .unwrap_or_else(|| "-".into());

        format!(
            "{placement} {stm} {castling} {ep} {} {}",
            self.halfmove_clock, self.fullmove_number
        )
    }

    /// Apply a legal (or search-assumed legal) move and return undo information.
    pub fn make_move(&mut self, mv: Move) -> UndoInfo {
        let moved = self.piece_at(mv.from).expect("move from empty square");
        let mut captured = self.piece_at(mv.to);
        let undo = UndoInfo {
            captured,
            castling: self.castling,
            ep_square: self.ep_square,
            halfmove: self.halfmove_clock,
            fullmove: self.fullmove_number,
            moved_piece: moved,
        };

        // En passant capture removes the pawn behind the destination.
        if mv.flags.is_en_passant {
            let cap_sq = match moved.color {
                Color::White => mv.to - 8,
                Color::Black => mv.to + 8,
            };
            captured = self.piece_at(cap_sq);
            self.set_piece(cap_sq, None);
        }

        self.set_piece(mv.from, None);

        let placed = if let Some(promo) = mv.flags.promotion {
            Piece::new(moved.color, promo)
        } else {
            moved
        };
        self.set_piece(mv.to, Some(placed));

        // Castling: move the rook.
        if mv.flags.is_castle {
            match (moved.color, mv.to) {
                (Color::White, 6) => {
                    // e1-g1
                    self.set_piece(7, None);
                    self.set_piece(5, Some(Piece::new(Color::White, PieceKind::Rook)));
                }
                (Color::White, 2) => {
                    // e1-c1
                    self.set_piece(0, None);
                    self.set_piece(3, Some(Piece::new(Color::White, PieceKind::Rook)));
                }
                (Color::Black, 62) => {
                    // e8-g8
                    self.set_piece(63, None);
                    self.set_piece(61, Some(Piece::new(Color::Black, PieceKind::Rook)));
                }
                (Color::Black, 58) => {
                    // e8-c8
                    self.set_piece(56, None);
                    self.set_piece(59, Some(Piece::new(Color::Black, PieceKind::Rook)));
                }
                _ => {}
            }
        }

        // Update castling rights.
        self.update_castling_rights(mv, moved, captured);

        // En passant target.
        self.ep_square = if mv.flags.is_double_pawn {
            Some(match moved.color {
                Color::White => mv.from + 8,
                Color::Black => mv.from - 8,
            })
        } else {
            None
        };

        // Clocks.
        if moved.kind == PieceKind::Pawn || captured.is_some() || mv.flags.is_en_passant {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.side_to_move == Color::Black {
            self.fullmove_number += 1;
        }
        self.side_to_move = self.side_to_move.opposite();

        // Store the true captured piece for undo (including EP).
        let mut undo = undo;
        undo.captured = captured;
        undo
    }

    pub fn unmake_move(&mut self, mv: Move, undo: UndoInfo) {
        self.side_to_move = self.side_to_move.opposite();
        self.castling = undo.castling;
        self.ep_square = undo.ep_square;
        self.halfmove_clock = undo.halfmove;
        self.fullmove_number = undo.fullmove;

        // Undo castling rook move first.
        if mv.flags.is_castle {
            match (undo.moved_piece.color, mv.to) {
                (Color::White, 6) => {
                    self.set_piece(5, None);
                    self.set_piece(7, Some(Piece::new(Color::White, PieceKind::Rook)));
                }
                (Color::White, 2) => {
                    self.set_piece(3, None);
                    self.set_piece(0, Some(Piece::new(Color::White, PieceKind::Rook)));
                }
                (Color::Black, 62) => {
                    self.set_piece(61, None);
                    self.set_piece(63, Some(Piece::new(Color::Black, PieceKind::Rook)));
                }
                (Color::Black, 58) => {
                    self.set_piece(59, None);
                    self.set_piece(56, Some(Piece::new(Color::Black, PieceKind::Rook)));
                }
                _ => {}
            }
        }

        self.set_piece(mv.to, None);
        self.set_piece(mv.from, Some(undo.moved_piece));

        if mv.flags.is_en_passant {
            let cap_sq = match undo.moved_piece.color {
                Color::White => mv.to - 8,
                Color::Black => mv.to + 8,
            };
            self.set_piece(cap_sq, undo.captured);
        } else if let Some(cap) = undo.captured {
            self.set_piece(mv.to, Some(cap));
        }
    }

    fn update_castling_rights(&mut self, mv: Move, moved: Piece, captured: Option<Piece>) {
        // King moves remove both rights.
        if moved.kind == PieceKind::King {
            match moved.color {
                Color::White => {
                    self.castling.white_king = false;
                    self.castling.white_queen = false;
                }
                Color::Black => {
                    self.castling.black_king = false;
                    self.castling.black_queen = false;
                }
            }
        }

        // Rook moves remove that side's right.
        if moved.kind == PieceKind::Rook {
            match (moved.color, mv.from) {
                (Color::White, 0) => self.castling.white_queen = false,
                (Color::White, 7) => self.castling.white_king = false,
                (Color::Black, 56) => self.castling.black_queen = false,
                (Color::Black, 63) => self.castling.black_king = false,
                _ => {}
            }
        }

        // Capturing a rook removes opponent's right.
        if let Some(cap) = captured {
            if cap.kind == PieceKind::Rook {
                match (cap.color, mv.to) {
                    (Color::White, 0) => self.castling.white_queen = false,
                    (Color::White, 7) => self.castling.white_king = false,
                    (Color::Black, 56) => self.castling.black_queen = false,
                    (Color::Black, 63) => self.castling.black_king = false,
                    _ => {}
                }
            }
        }
    }

    /// Serialize board occupancy for the UI (64 FEN chars or '.').
    pub fn occupancy_chars(&self) -> Vec<String> {
        self.squares
            .iter()
            .map(|p| match p {
                Some(piece) => piece.to_fen_char().to_string(),
                None => ".".to_string(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_fen_roundtrip() {
        let b = Board::startpos();
        assert_eq!(
            b.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }
}
