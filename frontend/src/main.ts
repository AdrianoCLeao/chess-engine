import "./style.css";

type Piece = "r" | "n" | "b" | "q" | "k" | "p" | "R" | "N" | "B" | "Q" | "K" | "P" | null;

type BoardState = Piece[][];

type Square = {
  file: string;
  rank: number;
  piece: Piece;
};

const files = ["a", "b", "c", "d", "e", "f", "g", "h"];
const ranks = [8, 7, 6, 5, 4, 3, 2, 1];

const pieceMap: Record<Exclude<Piece, null>, string> = {
  r: "♜",
  n: "♞",
  b: "♝",
  q: "♛",
  k: "♚",
  p: "♟",
  R: "♖",
  N: "♘",
  B: "♗",
  Q: "♕",
  K: "♔",
  P: "♙"
};

const initialBoard: BoardState = [
  ["r", "n", "b", "q", "k", "b", "n", "r"],
  ["p", "p", "p", "p", "p", "p", "p", "p"],
  [null, null, null, null, null, null, null, null],
  [null, null, null, null, null, null, null, null],
  [null, null, null, null, null, null, null, null],
  [null, null, null, null, null, null, null, null],
  ["P", "P", "P", "P", "P", "P", "P", "P"],
  ["R", "N", "B", "Q", "K", "B", "N", "R"]
];

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("Elemento #app não encontrado.");
}

app.innerHTML = `
  <div class="app-shell">
    <header>
      <h1>Chess Engine</h1>
      <p>Base da GUI em Tauri + Rust. Clique em qualquer casa para selecionar.</p>
    </header>
    <main>
      <section class="board-panel">
        <div class="board" aria-label="Tabuleiro de xadrez"></div>
        <div class="board-info">
          <div><strong>Selecionada:</strong> <span id="selected-square">-</span></div>
          <div><strong>Próximo passo:</strong> mover peças e conectar ao engine Rust.</div>
        </div>
      </section>
      <aside class="side-panel">
        <h2>Status</h2>
        <ul>
          <li>Renderização 8x8 pronta</li>
          <li>Peças iniciais com Unicode</li>
          <li>Pronto para conectar lógica Rust</li>
        </ul>
      </aside>
    </main>
  </div>
`;

const boardElement = app.querySelector<HTMLDivElement>(".board");
const selectedSquareEl = app.querySelector<HTMLSpanElement>("#selected-square");

if (!boardElement || !selectedSquareEl) {
  throw new Error("Componentes principais não encontrados.");
}

const selectedSquareLabel = selectedSquareEl as HTMLSpanElement;

let selectedSquare: string | null = null;

renderBoard(boardElement, initialBoard);

function renderBoard(container: HTMLDivElement, state: BoardState) {
  container.innerHTML = "";

  state.forEach((row, rowIndex) => {
    row.forEach((piece, colIndex) => {
      const square = buildSquare(rowIndex, colIndex, piece);
      container.appendChild(square);
    });
  });
}

function buildSquare(rowIndex: number, colIndex: number, piece: Piece) {
  const isDark = (rowIndex + colIndex) % 2 === 1;
  const squareId = `${files[colIndex]}${ranks[rowIndex]}`;
  const squareEl = document.createElement("button");

  squareEl.className = `square ${isDark ? "dark" : "light"}`;
  squareEl.dataset.square = squareId;
  squareEl.type = "button";
  squareEl.setAttribute("aria-label", `Casa ${squareId}`);

  if (piece) {
    const pieceEl = document.createElement("span");
    pieceEl.className = "piece";
    pieceEl.textContent = pieceMap[piece];
    squareEl.appendChild(pieceEl);
  }

  squareEl.addEventListener("click", () => {
    selectedSquare = squareId;
    highlightSquare(squareId);
    selectedSquareLabel.textContent = squareId;
  });

  return squareEl;
}

function highlightSquare(squareId: string) {
  document.querySelectorAll<HTMLButtonElement>(".square.selected").forEach((el) => {
    el.classList.remove("selected");
  });

  const target = document.querySelector<HTMLButtonElement>(`[data-square="${squareId}"]`);
  if (target) {
    target.classList.add("selected");
  }
}
