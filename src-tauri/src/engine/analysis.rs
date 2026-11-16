//! Move quality classification (Chess.com-style) and analysis payloads for the UI.

use super::board::Board;
use super::eval;
use super::search::{SearchLimits, Searcher};
use super::types::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Classification thresholds in centipawns of loss versus the best move.
/// Tuned to resemble Chess.com Game Review bands for middlegame positions.
fn classify_loss(loss_cp: i32, is_best: bool, sacrificed: bool, was_only_good: bool) -> MoveQuality {
    if is_best && sacrificed {
        return MoveQuality::Brilliant;
    }
    if is_best {
        return MoveQuality::Best;
    }
    if was_only_good || loss_cp <= 10 {
        return MoveQuality::Excellent;
    }
    if loss_cp <= 25 {
        return MoveQuality::Great;
    }
    if loss_cp <= 50 {
        return MoveQuality::Good;
    }
    if loss_cp <= 100 {
        return MoveQuality::Inaccuracy;
    }
    if loss_cp <= 200 {
        return MoveQuality::Mistake;
    }
    if loss_cp <= 300 {
        return MoveQuality::Miss;
    }
    MoveQuality::Blunder
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAnalysis {
    pub move_uci: String,
    pub move_san: String,
    pub quality: MoveQuality,
    pub best_move_uci: Option<String>,
    pub best_move_san: Option<String>,
    pub eval_before_cp: i32,
    pub eval_after_cp: i32,
    pub loss_cp: i32,
    pub explanation_key: String,
    pub explanation_params: ExplanationParams,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExplanationParams {
    pub played_san: String,
    pub best_san: String,
    pub loss_cp: i32,
    pub captured_piece: Option<String>,
    pub hanging_piece: Option<String>,
    pub hanging_square: Option<String>,
    pub check: bool,
    pub gave_check: bool,
    pub king_exposed: bool,
    pub castled: bool,
    pub promoted: bool,
    pub material_delta: i32,
    pub threat_piece: Option<String>,
    pub quality: String,
}

/// Analyze a single played move from `before` (side to move has not yet moved).
pub fn analyze_move(before: &Board, played: Move, analysis_depth: u32) -> MoveAnalysis {
    let played = before
        .normalize_move(played)
        .unwrap_or(played);
    let san = before.move_san(played);

    let limits = SearchLimits {
        max_depth: analysis_depth.max(2),
        max_nodes: 200_000,
        max_time: Duration::from_millis(600),
    };
    let mut searcher = Searcher::new(&limits);
    let scored = searcher.score_all_moves(before, limits.max_depth);

    let best = scored.first().copied();
    let best_score = best.map(|(_, s)| s).unwrap_or(0);
    let played_score = scored
        .iter()
        .find(|(m, _)| m.from == played.from && m.to == played.to && m.flags.promotion == played.flags.promotion)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| {
            // Move not found in list — evaluate after playing.
            let mut b = before.clone();
            b.make_move(played);
            -eval::evaluate_relative(&b)
        });

    let loss = (best_score - played_score).max(0);
    let is_best = best
        .map(|(m, _)| m.from == played.from && m.to == played.to && m.flags.promotion == played.flags.promotion)
        .unwrap_or(false);

    let material_before = material_balance(before);
    let mut after = before.clone();
    after.make_move(played);
    let material_after = material_balance(&after);
    // Material from the mover's perspective.
    let mover = before.side_to_move;
    let material_delta = (material_after - material_before) * mover.sign();

    let sacrificed = is_best && material_delta < -50;
    let quality = classify_loss(loss, is_best, sacrificed, false);

    let (key, params) = build_explanation(before, &after, played, &san, best.map(|m| m.0), loss, quality, material_delta);

    let eval_before = eval::evaluate(before);
    let eval_after = eval::evaluate(&after);

    MoveAnalysis {
        move_uci: played.to_uci(),
        move_san: san,
        quality,
        best_move_uci: best.map(|(m, _)| m.to_uci()),
        best_move_san: best.map(|(m, _)| before.move_san(m)),
        eval_before_cp: eval_before,
        eval_after_cp: eval_after,
        loss_cp: loss,
        explanation_key: key,
        explanation_params: params,
    }
}

fn material_balance(board: &Board) -> i32 {
    let mut s = 0;
    for sq in 0..64u8 {
        if let Some(p) = board.piece_at(sq) {
            if p.kind != PieceKind::King {
                s += p.kind.value_cp() * p.color.sign();
            }
        }
    }
    s
}

fn piece_name(kind: PieceKind) -> String {
    kind.to_char().to_string()
}

fn build_explanation(
    before: &Board,
    after: &Board,
    played: Move,
    san: &str,
    best: Option<Move>,
    loss: i32,
    quality: MoveQuality,
    material_delta: i32,
) -> (String, ExplanationParams) {
    let us = before.side_to_move;
    let them = us.opposite();

    let mut params = ExplanationParams {
        played_san: san.to_string(),
        best_san: best.map(|m| before.move_san(m)).unwrap_or_default(),
        loss_cp: loss,
        quality: quality.as_str().to_string(),
        material_delta,
        castled: played.flags.is_castle,
        promoted: played.flags.promotion.is_some(),
        gave_check: after.in_check(them),
        check: before.in_check(us),
        ..Default::default()
    };

    if let Some(cap) = before.piece_at(played.to) {
        params.captured_piece = Some(piece_name(cap.kind));
    } else if played.flags.is_en_passant {
        params.captured_piece = Some("p".into());
    }

    // Detect hanging piece after the move (our piece attacked and not defended well — simplified).
    if let Some(hanging) = find_hanging_piece(after, us) {
        params.hanging_piece = Some(piece_name(hanging.1.kind));
        params.hanging_square = Some(square_name(hanging.0));
        params.king_exposed = hanging.1.kind == PieceKind::King;
        // Also record a threat piece if any attacker exists.
        if let Some(attacker) = first_attacker(after, hanging.0, them) {
            params.threat_piece = Some(piece_name(attacker.kind));
        }
    }

    // King safety: did we step into check that wasn't resolved? (illegal — shouldn't happen)
    // Or leave king with more attackers nearby.
    if !params.king_exposed {
        if let Some(king) = after.find_king(us) {
            if after.is_square_attacked(king, them) {
                params.king_exposed = true;
            }
        }
    }

    let key = select_explanation_key(&params, quality);
    (key, params)
}

fn find_hanging_piece(board: &Board, us: Color) -> Option<(Square, Piece)> {
    let them = us.opposite();
    for sq in 0..64u8 {
        if let Some(p) = board.piece_at(sq) {
            if p.color != us || p.kind == PieceKind::King {
                continue;
            }
            if board.is_square_attacked(sq, them) && !board.is_square_attacked(sq, us) {
                return Some((sq, p));
            }
        }
    }
    // Also: piece attacked by lesser value (simplified MVV).
    for sq in 0..64u8 {
        if let Some(p) = board.piece_at(sq) {
            if p.color != us || p.kind == PieceKind::King {
                continue;
            }
            if board.is_square_attacked(sq, them) {
                if let Some(att) = first_attacker(board, sq, them) {
                    if att.kind.value_cp() < p.kind.value_cp() {
                        return Some((sq, p));
                    }
                }
            }
        }
    }
    None
}

fn first_attacker(board: &Board, sq: Square, by: Color) -> Option<Piece> {
    // Probe via reverse move-gen idea: check nearby piece attacks.
    for from in 0..64u8 {
        if let Some(p) = board.piece_at(from) {
            if p.color == by && board.is_square_attacked(sq, by) {
                // Verify this piece specifically attacks — approximate by temp removal check.
                let mut clone = board.clone();
                clone.set_piece(from, None);
                if !clone.is_square_attacked(sq, by) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn select_explanation_key(params: &ExplanationParams, quality: MoveQuality) -> String {
    match quality {
        MoveQuality::Brilliant => "brilliant_sacrifice".into(),
        MoveQuality::Best | MoveQuality::Excellent | MoveQuality::Great => {
            if params.castled {
                "good_castle".into()
            } else if params.gave_check {
                "good_check".into()
            } else if params.material_delta > 0 {
                "good_wins_material".into()
            } else {
                "good_best_line".into()
            }
        }
        MoveQuality::Good => "good_solid".into(),
        MoveQuality::Book => "book_move".into(),
        MoveQuality::Inaccuracy => {
            if params.hanging_piece.is_some() {
                "inaccuracy_hangs".into()
            } else {
                "inaccuracy_better_exists".into()
            }
        }
        MoveQuality::Mistake | MoveQuality::Miss => {
            if params.king_exposed {
                "mistake_king_exposed".into()
            } else if params.hanging_piece.is_some() {
                "mistake_hangs".into()
            } else if params.material_delta < 0 {
                "mistake_loses_material".into()
            } else {
                "mistake_better_exists".into()
            }
        }
        MoveQuality::Blunder => {
            if params.king_exposed || params.threat_piece.as_deref() == Some("n") {
                "blunder_king_threat".into()
            } else if params.hanging_piece.is_some() {
                "blunder_hangs".into()
            } else if params.material_delta < -200 {
                "blunder_loses_material".into()
            } else {
                "blunder_generic".into()
            }
        }
    }
}

/// Depth used for post-move review (independent of bot Elo).
pub fn review_depth_for_elo(elo: u32) -> u32 {
    EloReviewDepth::from_elo(elo).depth
}

struct EloReviewDepth {
    depth: u32,
}

impl EloReviewDepth {
    fn from_elo(elo: u32) -> Self {
        let depth = if elo < 1000 {
            3
        } else if elo < 1800 {
            4
        } else if elo < 2400 {
            5
        } else {
            6
        };
        Self { depth }
    }
}
