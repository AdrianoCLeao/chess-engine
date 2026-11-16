export type Color = "white" | "black";
export type PlayMode = "versus_bot" | "coach" | "analysis";
export type GameResult = "ongoing" | "white_wins" | "black_wins" | "draw";

export type GameConfig = {
  mode: PlayMode;
  human_color: Color;
  bot_elo: number;
  lang: string;
};

export type AnalyzedMoveDto = {
  quality: string;
  quality_label: string;
  explanation: string;
  loss_cp: number;
  eval_before_cp: number;
  eval_after_cp: number;
  best_move_uci?: string | null;
  best_move_san?: string | null;
  tip?: string | null;
};

export type HistoryEntry = {
  ply: number;
  uci: string;
  san: string;
  analysis?: AnalyzedMoveDto | null;
};

export type BotProfileDto = {
  elo: number;
  depth: number;
  think_ms: number;
  blunder_chance: number;
};

export type GameStateDto = {
  fen: string;
  board: string[];
  side_to_move: Color;
  legal_moves: string[];
  result: GameResult;
  in_check: boolean;
  config: GameConfig;
  history: HistoryEntry[];
  last_analysis?: AnalyzedMoveDto | null;
  coach_tip?: string | null;
  bot_profile: BotProfileDto;
};

export type DocSectionDto = {
  id: string;
  title: string;
  body: string;
};

export type MethodologyDto = {
  title: string;
  sections: DocSectionDto[];
};
