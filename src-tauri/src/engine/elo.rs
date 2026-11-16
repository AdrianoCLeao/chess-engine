//! Maps Chess.com-style Elo ratings (100–3000) onto search limits and move selection noise.

use super::board::Board;
use super::search::{SearchLimits, SearchResult, Searcher};
use super::types::Move;
use rand::Rng;
use std::time::Duration;

/// Clamp and describe an Elo rating used by the engine bots.
#[derive(Debug, Clone, Copy)]
pub struct EloProfile {
    pub elo: u32,
    pub depth: u32,
    pub max_nodes: u64,
    pub think_ms: u64,
    /// Probability of picking a sub-optimal move from the candidate pool.
    pub blunder_chance: f64,
    /// How far down the ordered move list we may wander (0 = always best).
    pub candidate_span: usize,
    /// Extra random centipawn noise added when scoring candidates at low Elo.
    pub noise_cp: i32,
}

impl EloProfile {
    /// Build a profile for Elo in \[100, 3000\], inspired by Chess.com bot ladders.
    pub fn from_elo(elo: u32) -> Self {
        let elo = elo.clamp(100, 3000);

        // Smooth curves: strength grows non-linearly toward master levels.
        let t = (elo as f64 - 100.0) / 2900.0; // 0..1

        let depth = if elo < 400 {
            1
        } else if elo < 800 {
            2
        } else if elo < 1200 {
            3
        } else if elo < 1600 {
            4
        } else if elo < 2000 {
            5
        } else if elo < 2400 {
            6
        } else if elo < 2700 {
            7
        } else if elo < 2900 {
            8
        } else {
            10
        };

        let max_nodes = (2_000.0 * (1.0 + t * t * 400.0)) as u64;
        let think_ms = (40.0 + t * t * 2500.0) as u64;

        let blunder_chance = if elo >= 2800 {
            0.0
        } else {
            ((1.0 - t).powi(2) * 0.55).clamp(0.0, 0.55)
        };

        let candidate_span = if elo < 600 {
            8
        } else if elo < 1000 {
            5
        } else if elo < 1400 {
            3
        } else if elo < 2000 {
            2
        } else if elo < 2600 {
            1
        } else {
            0
        };

        let noise_cp = if elo < 500 {
            180
        } else if elo < 1000 {
            100
        } else if elo < 1500 {
            50
        } else if elo < 2200 {
            20
        } else {
            0
        };

        Self {
            elo,
            depth,
            max_nodes,
            think_ms,
            blunder_chance,
            candidate_span,
            noise_cp,
        }
    }

    pub fn limits(&self) -> SearchLimits {
        SearchLimits {
            max_depth: self.depth,
            max_nodes: self.max_nodes,
            max_time: Duration::from_millis(self.think_ms),
        }
    }
}

/// Preset Elo steps shown in the UI (Chess.com-like ladder).
pub fn preset_levels() -> Vec<u32> {
    vec![
        100, 250, 400, 600, 800, 1000, 1200, 1400, 1600, 1800, 2000, 2200, 2400, 2600, 2800, 3000,
    ]
}

/// Choose a move appropriate for the given Elo.
pub fn choose_move(board: &Board, elo: u32) -> SearchResult {
    let profile = EloProfile::from_elo(elo);
    let limits = profile.limits();
    let mut searcher = Searcher::new(&limits);

    // At very low Elo, sometimes play a nearly random legal move.
    let mut rng = rand::thread_rng();
    if profile.elo <= 300 && rng.gen_bool(0.35) {
        let moves = board.legal_moves();
        let mv = moves.get(rng.gen_range(0..moves.len().max(1))).copied();
        return SearchResult {
            best_move: mv,
            score_cp: 0,
            depth: 1,
            nodes: searcher.nodes_searched(),
            pv: mv.into_iter().collect(),
        };
    }

    // Score candidates; for higher Elo use iterative deepening best move.
    if profile.candidate_span == 0 && profile.blunder_chance == 0.0 && profile.noise_cp == 0 {
        return searcher.search(board, &limits);
    }

    let depth = profile.depth.max(1);
    let mut scored = searcher.score_all_moves(board, depth);
    if scored.is_empty() {
        return searcher.search(board, &limits);
    }

    // Add evaluation noise for lower ratings.
    if profile.noise_cp > 0 {
        for entry in scored.iter_mut() {
            let noise = rng.gen_range(-profile.noise_cp..=profile.noise_cp);
            entry.1 += noise;
        }
        scored.sort_by(|a, b| b.1.cmp(&a.1));
    }

    let pick = select_with_blunders(&scored, &profile, &mut rng);
    let best_score = scored[0].1;

    SearchResult {
        best_move: Some(pick),
        score_cp: best_score,
        depth,
        nodes: searcher.nodes_searched(),
        pv: vec![pick],
    }
}

fn select_with_blunders<R: Rng>(
    scored: &[(Move, i32)],
    profile: &EloProfile,
    rng: &mut R,
) -> Move {
    let span = (profile.candidate_span + 1).min(scored.len());
    if span <= 1 || !rng.gen_bool(profile.blunder_chance.max(0.05).min(0.9)) {
        // Soft distribution over top candidates even without a "blunder".
        let weights: Vec<f64> = (0..span)
            .map(|i| {
                let loss = (scored[0].1 - scored[i].1).max(0) as f64;
                (-loss / 80.0).exp()
            })
            .collect();
        return weighted_pick(scored, &weights, rng);
    }

    // Intentional mistake: pick deeper in the list.
    let start = (span / 2).max(1);
    let idx = rng.gen_range(start..scored.len().min(span + 3).max(start + 1));
    scored[idx.min(scored.len() - 1)].0
}

fn weighted_pick<R: Rng>(scored: &[(Move, i32)], weights: &[f64], rng: &mut R) -> Move {
    let sum: f64 = weights.iter().sum();
    if sum <= 0.0 {
        return scored[0].0;
    }
    let mut roll = rng.gen::<f64>() * sum;
    for (i, w) in weights.iter().enumerate() {
        roll -= w;
        if roll <= 0.0 {
            return scored[i].0;
        }
    }
    scored[0].0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elo_is_clamped_and_monotonic_in_depth() {
        let low = EloProfile::from_elo(50);
        let mid = EloProfile::from_elo(1500);
        let high = EloProfile::from_elo(4000);
        assert_eq!(low.elo, 100);
        assert_eq!(high.elo, 3000);
        assert!(low.depth <= mid.depth);
        assert!(mid.depth <= high.depth);
        assert!(low.blunder_chance >= high.blunder_chance);
    }
}
