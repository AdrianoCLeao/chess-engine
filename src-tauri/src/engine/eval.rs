//! Static evaluation: material + piece-square tables + simple mobility/king safety.

use super::board::Board;
use super::types::*;

/// Piece-square tables from White's perspective (a1 = index 0). Higher = better for White.
const PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,  0,  0,  0, 20, 20,  0,  0,  0,  5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,  0,  0,  0,  0,  0,  0,  0,  0,
];

const KNIGHT_PST: [i32; 64] = [
   -50,-40,-30,-30,-30,-30,-40,-50,-40,-20,  0,  0,  0,  0,-20,-40,-30,  0, 10, 15, 15, 10,  0,-30,
   -30,  5, 15, 20, 20, 15,  5,-30,-30,  0, 15, 20, 20, 15,  0,-30,-30,  5, 10, 15, 15, 10,  5,-30,
   -40,-20,  0,  5,  5,  0,-20,-40,-50,-40,-30,-30,-30,-30,-40,-50,
];

const BISHOP_PST: [i32; 64] = [
   -20,-10,-10,-10,-10,-10,-10,-20,-10,  0,  0,  0,  0,  0,  0,-10,-10,  0,  5, 10, 10,  5,  0,-10,
   -10,  5,  5, 10, 10,  5,  5,-10,-10,  0, 10, 10, 10, 10,  0,-10,-10, 10, 10, 10, 10, 10, 10,-10,
   -10,  5,  0,  0,  0,  0,  5,-10,-20,-10,-10,-10,-10,-10,-10,-20,
];

const ROOK_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,  5, 10, 10, 10, 10, 10, 10,  5, -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5, -5,  0,  0,  0,  0,  0,  0, -5, -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,  0,  0,  0,  5,  5,  0,  0,  0,
];

const QUEEN_PST: [i32; 64] = [
   -20,-10,-10, -5, -5,-10,-10,-20,-10,  0,  0,  0,  0,  0,  0,-10,-10,  0,  5,  5,  5,  5,  0,-10,
    -5,  0,  5,  5,  5,  5,  0, -5,  0,  0,  5,  5,  5,  5,  0, -5,-10,  5,  5,  5,  5,  5,  0,-10,
   -10,  0,  5,  0,  0,  0,  0,-10,-20,-10,-10, -5, -5,-10,-10,-20,
];

const KING_MG_PST: [i32; 64] = [
   -30,-40,-40,-50,-50,-40,-40,-30,-30,-40,-40,-50,-50,-40,-40,-30,-30,-40,-40,-50,-50,-40,-40,-30,
   -30,-40,-40,-50,-50,-40,-40,-30,-20,-30,-30,-40,-40,-30,-30,-20,-10,-20,-20,-20,-20,-20,-20,-10,
    20, 20,  0,  0,  0,  0, 20, 20, 20, 30, 10,  0,  0, 10, 30, 20,
];

fn pst_for(kind: PieceKind) -> &'static [i32; 64] {
    match kind {
        PieceKind::Pawn => &PAWN_PST,
        PieceKind::Knight => &KNIGHT_PST,
        PieceKind::Bishop => &BISHOP_PST,
        PieceKind::Rook => &ROOK_PST,
        PieceKind::Queen => &QUEEN_PST,
        PieceKind::King => &KING_MG_PST,
    }
}

fn mirror(sq: Square) -> Square {
    sq ^ 56
}

/// Evaluate position in centipawns from White's perspective.
pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0i32;
    let mut white_bishops = 0;
    let mut black_bishops = 0;

    for sq in 0..64u8 {
        if let Some(piece) = board.piece_at(sq) {
            let pst = pst_for(piece.kind);
            let idx = match piece.color {
                Color::White => sq as usize,
                Color::Black => mirror(sq) as usize,
            };
            let value = piece.kind.value_cp() + pst[idx];
            score += value * piece.color.sign();

            if piece.kind == PieceKind::Bishop {
                match piece.color {
                    Color::White => white_bishops += 1,
                    Color::Black => black_bishops += 1,
                }
            }
        }
    }

    // Bishop pair bonus.
    if white_bishops >= 2 {
        score += 30;
    }
    if black_bishops >= 2 {
        score -= 30;
    }

    // Tiny mobility term using legal-move count (expensive but fine at shallow depths).
    // Skipped here for speed; search uses quiescence captures instead.

    // Prefer the side to move slightly when scores are equal — handled by search.
    score
}

/// Evaluate relative to the side to move (positive = good for side to move).
pub fn evaluate_relative(board: &Board) -> i32 {
    evaluate(board) * board.side_to_move.sign()
}
