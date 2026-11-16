//! Tauri command surface exposed to the desktop UI.

use crate::engine::analysis::{analyze_move, review_depth_for_elo};
use crate::engine::board::Board;
use crate::engine::coach::{self, CoachFeedback};
use crate::engine::elo::{choose_move, preset_levels, EloProfile};
use crate::engine::types::*;
use crate::i18n::{render_explanation, Lang};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayMode {
    VersusBot,
    Coach,
    Analysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub mode: PlayMode,
    pub human_color: Color,
    pub bot_elo: u32,
    pub lang: String,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            mode: PlayMode::VersusBot,
            human_color: Color::White,
            bot_elo: 1200,
            lang: "pt".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub ply: u32,
    pub uci: String,
    pub san: String,
    pub analysis: Option<AnalyzedMoveDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedMoveDto {
    pub quality: String,
    pub quality_label: String,
    pub explanation: String,
    pub loss_cp: i32,
    pub eval_before_cp: i32,
    pub eval_after_cp: i32,
    pub best_move_uci: Option<String>,
    pub best_move_san: Option<String>,
    pub tip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateDto {
    pub fen: String,
    pub board: Vec<String>,
    pub side_to_move: Color,
    pub legal_moves: Vec<String>,
    pub result: GameResult,
    pub in_check: bool,
    pub config: GameConfig,
    pub history: Vec<HistoryEntry>,
    pub last_analysis: Option<AnalyzedMoveDto>,
    pub coach_tip: Option<String>,
    pub bot_profile: BotProfileDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotProfileDto {
    pub elo: u32,
    pub depth: u32,
    pub think_ms: u64,
    pub blunder_chance: f64,
}

#[derive(Debug)]
pub struct AppState {
    pub board: Board,
    pub config: GameConfig,
    pub history: Vec<HistoryEntry>,
    pub last_analysis: Option<AnalyzedMoveDto>,
    pub coach_tip: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            board: Board::startpos(),
            config: GameConfig::default(),
            history: vec![],
            last_analysis: None,
            coach_tip: None,
        }
    }
}

pub type SharedState = Mutex<AppState>;

fn lang_of(config: &GameConfig) -> Lang {
    Lang::parse(&config.lang)
}

fn quality_label(lang: Lang, quality: &str) -> String {
    crate::i18n::locale_for(lang).quality_label(quality)
}

fn to_dto(analysis: &crate::engine::analysis::MoveAnalysis, lang: Lang, tip: Option<String>) -> AnalyzedMoveDto {
    let explanation = render_explanation(lang, &analysis.explanation_key, &analysis.explanation_params);
    AnalyzedMoveDto {
        quality: analysis.quality.as_str().to_string(),
        quality_label: quality_label(lang, analysis.quality.as_str()),
        explanation,
        loss_cp: analysis.loss_cp,
        eval_before_cp: analysis.eval_before_cp,
        eval_after_cp: analysis.eval_after_cp,
        best_move_uci: analysis.best_move_uci.clone(),
        best_move_san: analysis.best_move_san.clone(),
        tip,
    }
}

fn snapshot(state: &AppState) -> GameStateDto {
    let profile = EloProfile::from_elo(state.config.bot_elo);
    let legal = state
        .board
        .legal_moves()
        .into_iter()
        .map(|m| m.to_uci())
        .collect();
    GameStateDto {
        fen: state.board.to_fen(),
        board: state.board.occupancy_chars(),
        side_to_move: state.board.side_to_move,
        legal_moves: legal,
        result: state.board.game_result(),
        in_check: state.board.in_check(state.board.side_to_move),
        config: state.config.clone(),
        history: state.history.clone(),
        last_analysis: state.last_analysis.clone(),
        coach_tip: state.coach_tip.clone(),
        bot_profile: BotProfileDto {
            elo: profile.elo,
            depth: profile.depth,
            think_ms: profile.think_ms,
            blunder_chance: profile.blunder_chance,
        },
    }
}

fn analyze_and_store(state: &mut AppState, before: &Board, mv: Move) -> AnalyzedMoveDto {
    let lang = lang_of(&state.config);
    let depth = review_depth_for_elo(state.config.bot_elo);

    if state.config.mode == PlayMode::Coach {
        let fb: CoachFeedback = coach::feedback_for_move(before, mv, state.config.bot_elo, lang);
        let dto = to_dto(&fb.analysis, lang, Some(fb.tip.clone()));
        state.coach_tip = Some(fb.tip);
        state.last_analysis = Some(dto.clone());
        dto
    } else {
        let analysis = analyze_move(before, mv, depth);
        let dto = to_dto(&analysis, lang, None);
        state.coach_tip = None;
        state.last_analysis = Some(dto.clone());
        dto
    }
}

fn push_history(state: &mut AppState, san: String, uci: String, analysis: Option<AnalyzedMoveDto>) {
    let ply = state.history.len() as u32 + 1;
    state.history.push(HistoryEntry {
        ply,
        uci,
        san,
        analysis,
    });
}

/// Play one bot move when it is not the human's turn.
/// Used in every mode — the player never moves the opponent's pieces.
fn play_engine_if_needed(state: &mut AppState) {
    if state.board.game_result() != GameResult::Ongoing {
        return;
    }
    if state.board.side_to_move == state.config.human_color {
        return;
    }

    let before = state.board.clone();
    let Some(mv) = choose_move(&before, state.config.bot_elo).best_move else {
        return;
    };
    let mv = before.normalize_move(mv).unwrap_or(mv);
    let san = before.move_san(mv);
    let uci = mv.to_uci();
    let dto = analyze_and_store(state, &before, mv);
    state.board.make_move(mv);
    push_history(state, san, uci, Some(dto));
}

#[tauri::command]
pub fn get_state(state: State<'_, SharedState>) -> Result<GameStateDto, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(snapshot(&guard))
}

#[tauri::command]
pub fn set_config(config: GameConfig, state: State<'_, SharedState>) -> Result<GameStateDto, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let mut cfg = config;
    cfg.bot_elo = cfg.bot_elo.clamp(100, 3000);
    guard.config = cfg;
    if matches!(guard.config.mode, PlayMode::Coach) && guard.coach_tip.is_none() {
        guard.coach_tip =
            Some("Modo treinador ativo: a cada lance você receberá uma explicação e uma dica.".into());
    }
    Ok(snapshot(&guard))
}

#[tauri::command]
pub fn new_game(config: GameConfig, state: State<'_, SharedState>) -> Result<GameStateDto, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let mut cfg = config;
    cfg.bot_elo = cfg.bot_elo.clamp(100, 3000);
    let coach_tip = if matches!(cfg.mode, PlayMode::Coach) {
        Some("Modo treinador ativo: a cada lance você receberá uma explicação e uma dica.".into())
    } else {
        None
    };
    *guard = AppState {
        board: Board::startpos(),
        config: cfg,
        history: vec![],
        last_analysis: None,
        coach_tip,
    };
    // Bot opening move (human as black) is requested by the UI via `engine_move`
    // so the frontend can animate without freezing on `new_game`.
    Ok(snapshot(&guard))
}

#[tauri::command]
pub fn legal_targets(from: String, state: State<'_, SharedState>) -> Result<Vec<String>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let from_sq = square_from_name(&from).ok_or_else(|| format!("casa inválida: {from}"))?;
    let targets = guard
        .board
        .legal_moves()
        .into_iter()
        .filter(|m| m.from == from_sq)
        .map(|m| square_name(m.to))
        .collect();
    Ok(targets)
}

#[derive(Debug, Deserialize)]
pub struct PlayMoveRequest {
    pub from: String,
    pub to: String,
    pub promotion: Option<String>,
}

#[tauri::command]
pub fn play_move(req: PlayMoveRequest, state: State<'_, SharedState>) -> Result<GameStateDto, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;

    if guard.board.game_result() != GameResult::Ongoing {
        return Err("a partida já terminou".into());
    }
    // Every mode is human-vs-bot: only the configured human color may move here.
    if guard.board.side_to_move != guard.config.human_color {
        return Err("não é a sua vez".into());
    }

    let from = square_from_name(&req.from).ok_or("casa de origem inválida")?;
    let to = square_from_name(&req.to).ok_or("casa de destino inválida")?;
    let mut mv = Move::new(from, to);
    if let Some(p) = req.promotion {
        mv.flags.promotion = PieceKind::from_char(p.chars().next().unwrap_or('q'));
    }
    let before = guard.board.clone();
    let mv = before
        .normalize_move(mv)
        .ok_or("movimento inválido")?;
    if !before.is_legal(mv) {
        return Err("movimento ilegal".into());
    }

    let san = before.move_san(mv);
    let uci = mv.to_uci();
    let dto = analyze_and_store(&mut guard, &before, mv);
    guard.board.make_move(mv);
    push_history(&mut guard, san, uci, Some(dto));

    // Bot reply is intentionally NOT applied here — the UI calls `engine_move`
    // afterward so the human move paints first and the bot can animate.
    Ok(snapshot(&guard))
}

#[tauri::command]
pub fn engine_move(state: State<'_, SharedState>) -> Result<GameStateDto, String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    if guard.board.game_result() != GameResult::Ongoing {
        return Err("a partida já terminou".into());
    }
    if guard.board.side_to_move == guard.config.human_color {
        return Err("é a vez do jogador".into());
    }
    play_engine_if_needed(&mut guard);
    Ok(snapshot(&guard))
}

#[tauri::command]
pub fn analyze_current(state: State<'_, SharedState>) -> Result<AnalyzedMoveDto, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    let lang = lang_of(&guard.config);
    let depth = review_depth_for_elo(guard.config.bot_elo);
    let limits_depth = depth;
    // Analyze best move suggestion from current position (no move played yet).
    let ghost = guard.board.clone();
    let result = choose_move(&ghost, 3000);
    let Some(best) = result.best_move else {
        return Err("sem lances legais".into());
    };
    let analysis = analyze_move(&ghost, best, limits_depth);
    Ok(to_dto(&analysis, lang, Some(render_explanation(
        lang,
        "coach_hint_best",
        &analysis.explanation_params,
    ))))
}

#[tauri::command]
pub fn list_elo_levels() -> Vec<u32> {
    preset_levels()
}

#[tauri::command]
pub fn elo_profile(elo: u32) -> BotProfileDto {
    let p = EloProfile::from_elo(elo);
    BotProfileDto {
        elo: p.elo,
        depth: p.depth,
        think_ms: p.think_ms,
        blunder_chance: p.blunder_chance,
    }
}

#[tauri::command]
pub fn methodology_summary() -> MethodologyDto {
    MethodologyDto {
        title: "Metodologia da Engine".into(),
        sections: methodology_sections(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodologyDto {
    pub title: String,
    pub sections: Vec<DocSectionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSectionDto {
    pub id: String,
    pub title: String,
    pub body: String,
}

fn methodology_sections() -> Vec<DocSectionDto> {
    // Short summaries; the full docs live in the frontend documentation view.
    vec![
        DocSectionDto {
            id: "rules".into(),
            title: "Validação de movimentos".into(),
            body: "A engine gera lances pseudo-legais e filtra os que deixariam o rei em xeque. \
                   Suporta roque, en passant, promoção, xeque, mate e afogamento."
                .into(),
        },
        DocSectionDto {
            id: "search".into(),
            title: "Busca".into(),
            body: "Alpha-beta com deepening iterativo e quiescência em capturas. \
                   A avaliação combina material e piece-square tables."
                .into(),
        },
        DocSectionDto {
            id: "elo".into(),
            title: "Níveis Elo (100–3000)".into(),
            body: "Cada Elo mapeia profundidade, nós, tempo, ruído e chance de erro deliberado, \
                   inspirado na escada de bots do Chess.com."
                .into(),
        },
        DocSectionDto {
            id: "review".into(),
            title: "Classificação de lances".into(),
            body: "Comparamos o lance jogado ao melhor da engine e rotulamos por perda em centipeões: \
                   brilhante, melhor, excelente, ótimo, bom, impreciso, erro, lance perdido, péssimo."
                .into(),
        },
        DocSectionDto {
            id: "explain".into(),
            title: "Explicações".into(),
            body: "Chaves estáveis + parâmetros estruturados são renderizados por locales (hoje: pt). \
                   Novos idiomas só precisam de um novo Locale."
                .into(),
        },
    ]
}
