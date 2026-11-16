//! Pseudo-legal and legal move generation, plus attack detection.

use super::board::Board;
use super::types::*;

const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];

const KING_DELTAS: [(i8, i8); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

const BISHOP_DIRS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_DIRS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

fn try_offset(sq: Square, df: i8, dr: i8) -> Option<Square> {
    let file = square_file(sq) as i8 + df;
    let rank = square_rank(sq) as i8 + dr;
    if (0..8).contains(&file) && (0..8).contains(&rank) {
        Some(square_from(file as u8, rank as u8))
    } else {
        None
    }
}

impl Board {
    /// True if `sq` is attacked by any piece of `by`.
    pub fn is_square_attacked(&self, sq: Square, by: Color) -> bool {
        // White pawns attack up-left/up-right; a square is attacked by a white pawn
        // if there is a white pawn one rank below on an adjacent file.
        let pawn_from: [(i8, i8); 2] = match by {
            Color::White => [(-1, -1), (1, -1)],
            Color::Black => [(-1, 1), (1, 1)],
        };
        for (df, dr) in pawn_from {
            if let Some(from) = try_offset(sq, df, dr) {
                if let Some(p) = self.piece_at(from) {
                    if p.color == by && p.kind == PieceKind::Pawn {
                        return true;
                    }
                }
            }
        }

        for (df, dr) in KNIGHT_DELTAS {
            if let Some(from) = try_offset(sq, df, dr) {
                if let Some(p) = self.piece_at(from) {
                    if p.color == by && p.kind == PieceKind::Knight {
                        return true;
                    }
                }
            }
        }

        for (df, dr) in KING_DELTAS {
            if let Some(from) = try_offset(sq, df, dr) {
                if let Some(p) = self.piece_at(from) {
                    if p.color == by && p.kind == PieceKind::King {
                        return true;
                    }
                }
            }
        }

        for (df, dr) in BISHOP_DIRS {
            if self.ray_attacks(sq, df, dr, by, &[PieceKind::Bishop, PieceKind::Queen]) {
                return true;
            }
        }
        for (df, dr) in ROOK_DIRS {
            if self.ray_attacks(sq, df, dr, by, &[PieceKind::Rook, PieceKind::Queen]) {
                return true;
            }
        }

        false
    }

    fn ray_attacks(
        &self,
        sq: Square,
        df: i8,
        dr: i8,
        by: Color,
        kinds: &[PieceKind],
    ) -> bool {
        let mut file = square_file(sq) as i8 + df;
        let mut rank = square_rank(sq) as i8 + dr;
        while (0..8).contains(&file) && (0..8).contains(&rank) {
            let target = square_from(file as u8, rank as u8);
            if let Some(p) = self.piece_at(target) {
                return p.color == by && kinds.contains(&p.kind);
            }
            file += df;
            rank += dr;
        }
        false
    }

    pub fn in_check(&self, color: Color) -> bool {
        match self.find_king(color) {
            Some(king) => self.is_square_attacked(king, color.opposite()),
            None => false,
        }
    }

    /// Generate all legal moves for the side to move.
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        self.generate_pseudo_legal(&mut moves);
        moves
            .into_iter()
            .filter(|mv| self.is_legal(*mv))
            .collect()
    }

    pub fn is_legal(&self, mv: Move) -> bool {
        let mut copy = self.clone();
        // Ensure flags are filled for validation from UCI-ish moves.
        let mv = copy.normalize_move(mv).unwrap_or(mv);
        if !copy.is_pseudo_legal(mv) {
            return false;
        }
        let us = copy.side_to_move;
        let _ = copy.make_move(mv);
        !copy.in_check(us)
    }

    /// Fill castling / EP / double-pawn flags when the caller only provided from/to/promo.
    pub fn normalize_move(&self, mut mv: Move) -> Option<Move> {
        let piece = self.piece_at(mv.from)?;
        if piece.color != self.side_to_move {
            return None;
        }

        // Detect castling by king moving two files.
        if piece.kind == PieceKind::King {
            let from_f = square_file(mv.from) as i8;
            let to_f = square_file(mv.to) as i8;
            if (from_f - to_f).abs() == 2 {
                mv.flags.is_castle = true;
            }
        }

        // En passant: pawn moves diagonally to empty square matching ep target.
        if piece.kind == PieceKind::Pawn {
            if self.ep_square == Some(mv.to) && self.piece_at(mv.to).is_none() {
                let df = square_file(mv.to) as i8 - square_file(mv.from) as i8;
                if df.abs() == 1 {
                    mv.flags.is_en_passant = true;
                }
            }
            let from_r = square_rank(mv.from);
            let to_r = square_rank(mv.to);
            if (from_r as i8 - to_r as i8).abs() == 2 {
                mv.flags.is_double_pawn = true;
            }
            // Default promotion to queen if reaching last rank without specifying.
            let last = match piece.color {
                Color::White => 7,
                Color::Black => 0,
            };
            if to_r == last && mv.flags.promotion.is_none() {
                mv.flags.promotion = Some(PieceKind::Queen);
            }
        }

        Some(mv)
    }

    fn is_pseudo_legal(&self, mv: Move) -> bool {
        let mut list = Vec::new();
        self.generate_pseudo_legal(&mut list);
        list.iter().any(|m| {
            m.from == mv.from
                && m.to == mv.to
                && m.flags.promotion == mv.flags.promotion
                && m.flags.is_castle == mv.flags.is_castle
                && m.flags.is_en_passant == mv.flags.is_en_passant
        })
    }

    fn generate_pseudo_legal(&self, out: &mut Vec<Move>) {
        let us = self.side_to_move;
        for sq in 0..64u8 {
            if let Some(piece) = self.piece_at(sq) {
                if piece.color != us {
                    continue;
                }
                match piece.kind {
                    PieceKind::Pawn => self.gen_pawn(sq, piece.color, out),
                    PieceKind::Knight => self.gen_leaper(sq, us, &KNIGHT_DELTAS, out),
                    PieceKind::Bishop => self.gen_slider(sq, us, &BISHOP_DIRS, out),
                    PieceKind::Rook => self.gen_slider(sq, us, &ROOK_DIRS, out),
                    PieceKind::Queen => {
                        self.gen_slider(sq, us, &BISHOP_DIRS, out);
                        self.gen_slider(sq, us, &ROOK_DIRS, out);
                    }
                    PieceKind::King => {
                        self.gen_leaper(sq, us, &KING_DELTAS, out);
                        self.gen_castling(sq, us, out);
                    }
                }
            }
        }
    }

    fn gen_pawn(&self, sq: Square, color: Color, out: &mut Vec<Move>) {
        let forward: i8 = match color {
            Color::White => 1,
            Color::Black => -1,
        };
        let start_rank = match color {
            Color::White => 1,
            Color::Black => 6,
        };
        let promo_rank = match color {
            Color::White => 7,
            Color::Black => 0,
        };

        // Single push
        if let Some(to) = try_offset(sq, 0, forward) {
            if self.piece_at(to).is_none() {
                self.push_pawn_moves(sq, to, square_rank(to) == promo_rank, out);
                // Double push
                if square_rank(sq) == start_rank {
                    if let Some(to2) = try_offset(sq, 0, forward * 2) {
                        if self.piece_at(to2).is_none() {
                            let mut mv = Move::new(sq, to2);
                            mv.flags.is_double_pawn = true;
                            out.push(mv);
                        }
                    }
                }
            }
        }

        // Captures
        for df in [-1i8, 1] {
            if let Some(to) = try_offset(sq, df, forward) {
                if let Some(victim) = self.piece_at(to) {
                    if victim.color != color {
                        self.push_pawn_moves(sq, to, square_rank(to) == promo_rank, out);
                    }
                } else if self.ep_square == Some(to) {
                    let mut mv = Move::new(sq, to);
                    mv.flags.is_en_passant = true;
                    out.push(mv);
                }
            }
        }
    }

    fn push_pawn_moves(&self, from: Square, to: Square, promote: bool, out: &mut Vec<Move>) {
        if promote {
            for kind in [
                PieceKind::Queen,
                PieceKind::Rook,
                PieceKind::Bishop,
                PieceKind::Knight,
            ] {
                out.push(Move::with_promotion(from, to, kind));
            }
        } else {
            out.push(Move::new(from, to));
        }
    }

    fn gen_leaper(&self, sq: Square, us: Color, deltas: &[(i8, i8)], out: &mut Vec<Move>) {
        for &(df, dr) in deltas {
            if let Some(to) = try_offset(sq, df, dr) {
                match self.piece_at(to) {
                    None => out.push(Move::new(sq, to)),
                    Some(p) if p.color != us => out.push(Move::new(sq, to)),
                    _ => {}
                }
            }
        }
    }

    fn gen_slider(&self, sq: Square, us: Color, dirs: &[(i8, i8)], out: &mut Vec<Move>) {
        for &(df, dr) in dirs {
            let mut file = square_file(sq) as i8 + df;
            let mut rank = square_rank(sq) as i8 + dr;
            while (0..8).contains(&file) && (0..8).contains(&rank) {
                let to = square_from(file as u8, rank as u8);
                match self.piece_at(to) {
                    None => out.push(Move::new(sq, to)),
                    Some(p) => {
                        if p.color != us {
                            out.push(Move::new(sq, to));
                        }
                        break;
                    }
                }
                file += df;
                rank += dr;
            }
        }
    }

    fn gen_castling(&self, sq: Square, us: Color, out: &mut Vec<Move>) {
        // King must be on original square.
        let (king_sq, k_right, q_right, k_to, q_to, k_pass, q_pass, q_path) = match us {
            Color::White => (
                4u8,
                self.castling.white_king,
                self.castling.white_queen,
                6u8,
                2u8,
                [5u8, 6u8],
                [3u8, 2u8],
                [1u8, 2u8, 3u8],
            ),
            Color::Black => (
                60u8,
                self.castling.black_king,
                self.castling.black_queen,
                62u8,
                58u8,
                [61u8, 62u8],
                [59u8, 58u8],
                [57u8, 58u8, 59u8],
            ),
        };

        if sq != king_sq {
            return;
        }
        if self.in_check(us) {
            return;
        }
        let enemy = us.opposite();

        if k_right {
            if k_pass.iter().all(|&s| self.piece_at(s).is_none())
                && k_pass.iter().all(|&s| !self.is_square_attacked(s, enemy))
            {
                let mut mv = Move::new(sq, k_to);
                mv.flags.is_castle = true;
                out.push(mv);
            }
        }

        if q_right {
            if q_path.iter().all(|&s| self.piece_at(s).is_none())
                && q_pass.iter().all(|&s| !self.is_square_attacked(s, enemy))
            {
                let mut mv = Move::new(sq, q_to);
                mv.flags.is_castle = true;
                out.push(mv);
            }
        }
    }

    pub fn game_result(&self) -> GameResult {
        let moves = self.legal_moves();
        if !moves.is_empty() {
            if self.halfmove_clock >= 100 {
                return GameResult::Draw;
            }
            return GameResult::Ongoing;
        }
        if self.in_check(self.side_to_move) {
            match self.side_to_move {
                Color::White => GameResult::BlackWins,
                Color::Black => GameResult::WhiteWins,
            }
        } else {
            GameResult::Draw
        }
    }

    /// SAN-ish short algebraic for UI (not fully disambiguated, good enough for analysis text).
    pub fn move_san(&self, mv: Move) -> String {
        let mv = self.normalize_move(mv).unwrap_or(mv);
        let piece = match self.piece_at(mv.from) {
            Some(p) => p,
            None => return mv.to_uci(),
        };

        if mv.flags.is_castle {
            return if square_file(mv.to) == 6 {
                "O-O".into()
            } else {
                "O-O-O".into()
            };
        }

        let mut s = String::new();
        if piece.kind != PieceKind::Pawn {
            s.push(piece.kind.to_char().to_ascii_uppercase());
        } else if self.piece_at(mv.to).is_some() || mv.flags.is_en_passant {
            s.push((b'a' + square_file(mv.from)) as char);
        }

        let capture = self.piece_at(mv.to).is_some() || mv.flags.is_en_passant;
        if capture {
            s.push('x');
        }
        s.push_str(&square_name(mv.to));

        if let Some(promo) = mv.flags.promotion {
            s.push('=');
            s.push(promo.to_char().to_ascii_uppercase());
        }

        let mut after = self.clone();
        let _ = after.make_move(mv);
        if after.in_check(after.side_to_move) {
            if after.legal_moves().is_empty() {
                s.push('#');
            } else {
                s.push('+');
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_has_20_moves() {
        let b = Board::startpos();
        assert_eq!(b.legal_moves().len(), 20);
    }

    #[test]
    fn scholars_mate_is_checkmate() {
        // Fool's mate: 1.f3 e5 2.g4 Qh4#
        let mut b = Board::startpos();
        for uci in ["f2f3", "e7e5", "g2g4", "d8h4"] {
            let mv = b.normalize_move(Move::from_uci(uci).unwrap()).unwrap();
            assert!(b.is_legal(mv), "{uci} should be legal");
            b.make_move(mv);
        }
        assert_eq!(b.game_result(), GameResult::BlackWins);
    }
}
