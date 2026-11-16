/**
 * Real piece artwork from the Lichess "companion" set
 * (upstream: lichess-org/lila public/piece/companion).
 * See frontend/public/pieces/ATTRIBUTION.txt
 */

const SET = "companion";

const FEN_TO_FILE: Record<string, string> = {
  P: "wP",
  N: "wN",
  B: "wB",
  R: "wR",
  Q: "wQ",
  K: "wK",
  p: "bP",
  n: "bN",
  b: "bB",
  r: "bR",
  q: "bQ",
  k: "bK"
};

export function pieceSrc(fenChar: string): string | null {
  const file = FEN_TO_FILE[fenChar];
  if (!file) return null;
  return `/pieces/${SET}/${file}.svg`;
}

/** Inline <img> markup for a FEN piece. */
export function pieceImg(fenChar: string, extraClass = ""): string {
  const src = pieceSrc(fenChar);
  if (!src) return "";
  return `<img class="piece-img ${extraClass}" src="${src}" alt="" draggable="false" />`;
}
