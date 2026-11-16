import { pieceImg } from "./pieces";

const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"];

export type BoardHandlers = {
  onSquare: (square: string) => void;
};

export function squareName(file: number, rank: number): string {
  return `${FILES[file]}${rank + 1}`;
}

export function renderBoard(
  root: HTMLElement,
  occupancy: string[],
  opts: {
    selected?: string | null;
    targets?: string[];
    lastFrom?: string | null;
    lastTo?: string | null;
    checkSquare?: string | null;
    flipped?: boolean;
    showCoords?: boolean;
    animating?: { from: string; to: string; piece: string } | null;
  },
  handlers: BoardHandlers
) {
  root.innerHTML = "";
  root.classList.toggle("is-flipped", !!opts.flipped);

  const ranks = opts.flipped ? [0, 1, 2, 3, 4, 5, 6, 7] : [7, 6, 5, 4, 3, 2, 1, 0];
  const files = opts.flipped ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];

  for (const rank of ranks) {
    for (const file of files) {
      const index = rank * 8 + file;
      const sq = squareName(file, rank);
      let fen = occupancy[index] && occupancy[index] !== "." ? occupancy[index] : null;

      // Hide destination piece while animating the flyer onto it
      if (opts.animating && opts.animating.to === sq) {
        fen = null;
      }
      // Hide source during animation (piece is flying)
      if (opts.animating && opts.animating.from === sq) {
        fen = null;
      }

      const isDark = (file + rank) % 2 === 0;
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `sq ${isDark ? "sq-dark" : "sq-light"}`;
      btn.dataset.square = sq;
      btn.setAttribute("aria-label", fen ? `Casa ${sq}, ${fen}` : `Casa ${sq}`);

      if (opts.selected === sq) btn.classList.add("is-selected");
      if (opts.lastFrom === sq || opts.lastTo === sq) btn.classList.add("is-last");
      if (opts.checkSquare === sq) btn.classList.add("is-check");
      if (opts.targets?.includes(sq)) {
        btn.classList.add(occupancy[index] && occupancy[index] !== "." ? "is-capture" : "is-target");
      }

      if (opts.showCoords !== false) {
        const showFile = opts.flipped ? rank === 7 : rank === 0;
        const showRank = opts.flipped ? file === 7 : file === 0;
        if (showFile) {
          const c = document.createElement("span");
          c.className = "sq-coord sq-coord-file";
          c.textContent = FILES[file];
          btn.appendChild(c);
        }
        if (showRank) {
          const c = document.createElement("span");
          c.className = "sq-coord sq-coord-rank";
          c.textContent = String(rank + 1);
          btn.appendChild(c);
        }
      }

      if (fen) {
        const holder = document.createElement("span");
        holder.className = "sq-piece";
        holder.innerHTML = pieceImg(fen);
        btn.appendChild(holder);
      }

      btn.addEventListener("click", () => handlers.onSquare(sq));
      root.appendChild(btn);
    }
  }
}

export function findKingSquare(occupancy: string[], color: "white" | "black"): string | null {
  const needle = color === "white" ? "K" : "k";
  for (let i = 0; i < 64; i++) {
    if (occupancy[i] === needle) {
      return squareName(i % 8, Math.floor(i / 8));
    }
  }
  return null;
}

export function squareToIndex(sq: string): number {
  const file = sq.charCodeAt(0) - "a".charCodeAt(0);
  const rank = Number(sq[1]) - 1;
  return rank * 8 + file;
}

/** Pixel center of a square relative to the board element. */
export function squareCenter(boardEl: HTMLElement, sq: string, flipped: boolean): { x: number; y: number } {
  const file = sq.charCodeAt(0) - "a".charCodeAt(0);
  const rank = Number(sq[1]) - 1;
  const col = flipped ? 7 - file : file;
  const row = flipped ? rank : 7 - rank;
  const cell = boardEl.clientWidth / 8;
  return {
    x: col * cell + cell / 2,
    y: row * cell + cell / 2
  };
}
