import { invoke } from "@tauri-apps/api/core";
import type {
  AnalyzedMoveDto,
  BotProfileDto,
  GameConfig,
  GameStateDto,
  MethodologyDto
} from "./types";

/** Thin wrappers around Tauri commands. Falls back with clear errors outside Tauri. */
async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Falha ao chamar ${cmd}: ${message}`);
  }
}

export const api = {
  getState: () => call<GameStateDto>("get_state"),
  newGame: (config: GameConfig) => call<GameStateDto>("new_game", { config }),
  setConfig: (config: GameConfig) => call<GameStateDto>("set_config", { config }),
  legalTargets: (from: string) => call<string[]>("legal_targets", { from }),
  playMove: (from: string, to: string, promotion?: string) =>
    call<GameStateDto>("play_move", {
      req: { from, to, promotion: promotion ?? null }
    }),
  engineMove: () => call<GameStateDto>("engine_move"),
  analyzeCurrent: () => call<AnalyzedMoveDto>("analyze_current"),
  listEloLevels: () => call<number[]>("list_elo_levels"),
  eloProfile: (elo: number) => call<BotProfileDto>("elo_profile", { elo }),
  methodologySummary: () => call<MethodologyDto>("methodology_summary")
};
