//! Coach mode: the engine plays at a chosen Elo and teaches after every learner move.

use super::analysis::{analyze_move, MoveAnalysis};
use super::board::Board;
use super::elo::{choose_move, EloProfile};
use super::types::*;
use crate::i18n::{render_explanation, Lang};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachFeedback {
    pub analysis: MoveAnalysis,
    pub explanation: String,
    pub tip: String,
    pub suggested_move_uci: Option<String>,
    pub suggested_move_san: Option<String>,
    pub elo: u32,
}

/// Build teaching feedback for a move the human just played.
pub fn feedback_for_move(
    before: &Board,
    played: Move,
    elo: u32,
    lang: Lang,
) -> CoachFeedback {
    let profile = EloProfile::from_elo(elo);
    // Coach reviews a bit deeper than the bot plays, so lessons stay useful.
    let depth = (profile.depth + 1).clamp(3, 7);
    let analysis = analyze_move(before, played, depth);
    let explanation = render_explanation(lang, &analysis.explanation_key, &analysis.explanation_params);

    let tip = match analysis.quality {
        MoveQuality::Brilliant | MoveQuality::Best | MoveQuality::Excellent | MoveQuality::Great => {
            render_explanation(lang, "coach_praise", &analysis.explanation_params)
        }
        MoveQuality::Good | MoveQuality::Book => explanation.clone(),
        _ => {
            let mut params = analysis.explanation_params.clone();
            // Ensure best move is present for the tip.
            if params.best_san.is_empty() {
                params.best_san = analysis
                    .best_move_san
                    .clone()
                    .unwrap_or_default();
            }
            render_explanation(lang, "coach_hint_best", &params)
        }
    };

    CoachFeedback {
        suggested_move_uci: analysis.best_move_uci.clone(),
        suggested_move_san: analysis.best_move_san.clone(),
        analysis,
        explanation,
        tip,
        elo,
    }
}

/// Engine reply in coach mode (plays at the selected Elo).
pub fn coach_reply_move(board: &Board, elo: u32) -> Option<Move> {
    choose_move(board, elo).best_move
}
