//! Alpha-beta search with iterative deepening and quiescence.

use super::board::Board;
use super::eval;
use super::types::*;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

const MATE_SCORE: i32 = 100_000;
const INF: i32 = 1_000_000;

#[derive(Debug, Clone)]
pub struct SearchLimits {
    pub max_depth: u32,
    pub max_nodes: u64,
    pub max_time: Duration,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            max_depth: 4,
            max_nodes: 500_000,
            max_time: Duration::from_millis(800),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score_cp: i32,
    pub depth: u32,
    pub nodes: u64,
    pub pv: Vec<Move>,
}

#[derive(Debug)]
pub struct Searcher {
    nodes: u64,
    deadline: Instant,
    max_nodes: u64,
    aborted: bool,
}

impl Searcher {
    pub fn new(limits: &SearchLimits) -> Self {
        Self {
            nodes: 0,
            deadline: Instant::now() + limits.max_time,
            max_nodes: limits.max_nodes,
            aborted: false,
        }
    }

    pub fn nodes_searched(&self) -> u64 {
        self.nodes
    }

    pub fn search(&mut self, board: &Board, limits: &SearchLimits) -> SearchResult {
        let mut best: Option<Move> = None;
        let mut best_score = 0;
        let mut best_depth = 0;
        let mut pv = Vec::new();

        let moves = board.legal_moves();
        if moves.is_empty() {
            let score = if board.in_check(board.side_to_move) {
                -MATE_SCORE
            } else {
                0
            };
            return SearchResult {
                best_move: None,
                score_cp: score,
                depth: 0,
                nodes: 0,
                pv: vec![],
            };
        }

        // Iterative deepening.
        for depth in 1..=limits.max_depth {
            if self.should_stop() {
                break;
            }
            let mut alpha = -INF;
            let beta = INF;
            let mut local_best = None;
            let mut local_score = -INF;
            let mut ordered = moves.clone();
            order_moves(board, &mut ordered, best);

            for mv in ordered {
                if self.should_stop() {
                    break;
                }
                let mut child = board.clone();
                let undo = child.make_move(mv);
                let score = -self.alphabeta(&mut child, depth - 1, -beta, -alpha);
                child.unmake_move(mv, undo);

                if score > local_score {
                    local_score = score;
                    local_best = Some(mv);
                }
                if score > alpha {
                    alpha = score;
                }
            }

            if !self.aborted {
                if let Some(mv) = local_best {
                    best = Some(mv);
                    best_score = local_score;
                    best_depth = depth;
                    pv = vec![mv];
                }
            }
        }

        SearchResult {
            best_move: best.or_else(|| moves.first().copied()),
            score_cp: best_score,
            depth: best_depth,
            nodes: self.nodes,
            pv,
        }
    }

    /// Root multi-PV helper: score every legal move at a fixed depth.
    pub fn score_all_moves(&mut self, board: &Board, depth: u32) -> Vec<(Move, i32)> {
        let mut scored = Vec::new();
        let mut moves = board.legal_moves();
        order_moves(board, &mut moves, None);
        for mv in moves {
            if self.should_stop() {
                break;
            }
            let mut child = board.clone();
            let undo = child.make_move(mv);
            let score = -self.alphabeta(&mut child, depth.saturating_sub(1), -INF, INF);
            child.unmake_move(mv, undo);
            scored.push((mv, score));
        }
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored
    }

    fn alphabeta(&mut self, board: &mut Board, depth: u32, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.should_stop() {
            return eval::evaluate_relative(board);
        }

        if depth == 0 {
            return self.quiesce(board, alpha, beta);
        }

        let mut moves = board.legal_moves();
        if moves.is_empty() {
            return if board.in_check(board.side_to_move) {
                -MATE_SCORE + self.nodes as i32 % 100 // prefer shorter mates slightly via depth
            } else {
                0
            };
        }

        order_moves(board, &mut moves, None);
        let mut best = -INF;
        for mv in moves {
            let undo = board.make_move(mv);
            let score = -self.alphabeta(board, depth - 1, -beta, -alpha);
            board.unmake_move(mv, undo);

            if score > best {
                best = score;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }
        best
    }

    fn quiesce(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        let stand_pat = eval::evaluate_relative(board);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut captures: Vec<Move> = board
            .legal_moves()
            .into_iter()
            .filter(|mv| board.piece_at(mv.to).is_some() || mv.flags.is_en_passant)
            .collect();
        order_moves(board, &mut captures, None);

        for mv in captures {
            if self.should_stop() {
                break;
            }
            let undo = board.make_move(mv);
            let score = -self.quiesce(board, -beta, -alpha);
            board.unmake_move(mv, undo);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    fn should_stop(&mut self) -> bool {
        if self.nodes >= self.max_nodes || Instant::now() >= self.deadline {
            self.aborted = true;
            return true;
        }
        false
    }
}

fn order_moves(board: &Board, moves: &mut [Move], hint: Option<Move>) {
    moves.sort_by(|a, b| {
        if hint == Some(*a) {
            return Ordering::Less;
        }
        if hint == Some(*b) {
            return Ordering::Greater;
        }
        move_score(board, *b).cmp(&move_score(board, *a))
    });
}

fn move_score(board: &Board, mv: Move) -> i32 {
    let mut s = 0;
    if let Some(victim) = board.piece_at(mv.to) {
        let attacker = board.piece_at(mv.from).map(|p| p.kind.value_cp()).unwrap_or(0);
        s += 10_000 + victim.kind.value_cp() * 10 - attacker / 10;
    }
    if mv.flags.is_en_passant {
        s += 10_000;
    }
    if mv.flags.promotion.is_some() {
        s += 9_000;
    }
    if mv.flags.is_castle {
        s += 500;
    }
    s
}

/// Absolute score from White's perspective.
pub fn white_score(board: &Board, relative: i32) -> i32 {
    relative * board.side_to_move.sign()
}
