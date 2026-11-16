import "./styles/app.css";
import { api } from "./api/client";
import type { GameConfig, GameStateDto, PlayMode } from "./api/types";
import { DOC_PAGES, searchDocs } from "./docs/content";
import { setLocale, t } from "./i18n";
import {
  findKingSquare,
  renderBoard,
  squareCenter,
  squareToIndex
} from "./ui/board";
import {
  bindSegmented,
  bindSelects,
  bindSliders,
  bindToggles,
  customSelectHtml,
  customSliderHtml,
  readSelect,
  readSlider,
  segmentedHtml,
  toggleHtml
} from "./ui/controls";
import { icons } from "./ui/icons";
import { renderMarkdown } from "./ui/markdown";
import { pieceImg, pieceSrc } from "./ui/pieces";

type ViewId = "play" | "coach" | "analysis" | "docs" | "settings";
type ThemeMode = "light" | "dark" | "system";

const THEME_KEY = "chess-engine-theme";
const COORDS_KEY = "chess-engine-coords";
const BOT_MOVE_MS = 320;

let state: GameStateDto | null = null;
let view: ViewId = "play";
let selected: string | null = null;
let targets: string[] = [];
let flipped = false;
let busy = false;
let botThinking = false;
let theme: ThemeMode = (localStorage.getItem(THEME_KEY) as ThemeMode) || "dark";
let showCoords = localStorage.getItem(COORDS_KEY) !== "0";
let docsQuery = "";
let docsPageId = DOC_PAGES[0]?.id ?? "intro";
let pendingPromotion: { from: string; to: string } | null = null;
let draftElo = 1200;
let draftColor: "white" | "black" = "white";
let animating: { from: string; to: string; piece: string } | null = null;

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("#app missing");

setLocale("pt");
applyTheme(theme);
app.innerHTML = shellHtml();
bindShell();
void boot();

function shellHtml(): string {
  const i = t();
  return `
  <div class="app">
    <nav class="rail" aria-label="Navegação principal">
      <div class="brand" title="${i.appName}">
        <img src="/icons/icon.png" alt="" width="44" height="44" />
        <div class="brand-text">
          <strong>Chess</strong>
          <span>Engine</span>
        </div>
      </div>
      <div class="rail-nav">
        ${railBtn("play", icons.play, i.nav.play)}
        ${railBtn("coach", icons.coach, i.nav.coach)}
        ${railBtn("analysis", icons.analysis, i.nav.analysis)}
        ${railBtn("docs", icons.docs, i.nav.docs)}
      </div>
      <div class="rail-footer">
        ${railBtn("settings", icons.settings, i.nav.settings)}
      </div>
    </nav>
    <div class="workspace">
      <header class="topbar">
        <div class="topbar-copy">
          <h1 id="view-title">${i.appName}</h1>
          <p id="view-subtitle">${i.tagline}</p>
        </div>
        <div class="top-actions" id="top-actions"></div>
      </header>
      <main class="content" id="main"></main>
    </div>
  </div>`;
}

function railBtn(id: ViewId, icon: string, label: string): string {
  return `
    <button class="rail-btn" type="button" data-view="${id}" aria-current="${
      view === id ? "page" : "false"
    }">
      <span class="rail-icon">${icon}</span>
      <span class="rail-label">${label}</span>
    </button>`;
}

function bindShell() {
  app!.querySelectorAll<HTMLButtonElement>("[data-view]").forEach((btn) => {
    btn.addEventListener("click", () => {
      view = btn.dataset.view as ViewId;
      selected = null;
      targets = [];
      syncRail();
      void syncModeWithView();
      render();
    });
  });
}

function syncRail() {
  app!.querySelectorAll<HTMLButtonElement>("[data-view]").forEach((btn) => {
    btn.setAttribute("aria-current", btn.dataset.view === view ? "page" : "false");
  });
}

async function syncModeWithView() {
  if (!state) return;
  if (view !== "play" && view !== "coach" && view !== "analysis") return;
  const mode: PlayMode =
    view === "coach" ? "coach" : view === "analysis" ? "analysis" : "versus_bot";
  if (state.config.mode === mode) return;
  try {
    state = await api.setConfig({ ...state.config, mode });
  } catch (e) {
    console.error(e);
  }
}

async function boot() {
  const main = document.getElementById("main");
  if (main) main.innerHTML = `<div class="panel panel-pad"><p class="muted">${t().labels.loading}</p></div>`;
  try {
    state = await api.getState();
    draftElo = state.config.bot_elo;
    draftColor = state.config.human_color;
    if (!state.history.length) {
      state = await api.newGame({
        mode: "versus_bot",
        human_color: "white",
        bot_elo: 1200,
        lang: "pt"
      });
      draftElo = 1200;
      draftColor = "white";
    }
    await maybeBotReply(false);
  } catch (e) {
    toast(String(e));
  }
  render();
}

function render() {
  const i = t();
  const title = document.getElementById("view-title");
  const subtitle = document.getElementById("view-subtitle");
  const actions = document.getElementById("top-actions");
  const main = document.getElementById("main");
  if (!title || !subtitle || !actions || !main) return;

  const titles: Record<ViewId, [string, string]> = {
    play: [i.nav.play, i.modes.versus_bot],
    coach: [i.nav.coach, i.coach.intro],
    analysis: [i.nav.analysis, "Você joga; o bot responde. Use analisar para ver a melhor ideia."],
    docs: [i.docs.title, "Metodologia da engine, busca, Elo e classificação de lances"],
    settings: [i.settings.title, "Aparência, tabuleiro e preferências da aplicação"]
  };
  title.textContent = titles[view][0];
  subtitle.textContent = titles[view][1];

  actions.innerHTML = "";
  if (view === "play" || view === "coach" || view === "analysis") {
    actions.innerHTML = `
      <button class="btn" type="button" id="btn-flip">${icons.flip}<span>${i.actions.flipBoard}</span></button>
      <button class="btn btn-primary" type="button" id="btn-new">${icons.renew}<span>${i.actions.newGame}</span></button>
    `;
    actions.querySelector("#btn-flip")?.addEventListener("click", () => {
      flipped = !flipped;
      render();
    });
    actions.querySelector("#btn-new")?.addEventListener("click", () => void startNewGame());
  }

  if (view === "docs") {
    main.className = "content docs-layout";
    renderDocs(main);
    return;
  }
  if (view === "settings") {
    main.className = "content single";
    renderSettings(main);
    return;
  }

  main.className = "content";
  renderGame(main);
}

function boardIsFlipped(): boolean {
  return flipped !== (state?.config.human_color === "black");
}

function renderGame(main: HTMLElement) {
  const i = t();
  if (!state) {
    main.innerHTML = `<div class="panel panel-pad empty">${i.labels.loading}</div>`;
    return;
  }

  const last = state.history[state.history.length - 1];
  const checkSq = state.in_check ? findKingSquare(state.board, state.side_to_move) : null;

  main.innerHTML = `
    <section class="board-col">
      <div class="board-meta">
        <span class="chip">${i.labels.sideToMove} <strong>${
          state.side_to_move === "white" ? i.color.white : i.color.black
        }</strong></span>
        <span class="chip">${i.labels.result} <strong>${i.results[state.result]}</strong></span>
        ${state.in_check ? `<span class="chip"><strong>${i.labels.inCheck}</strong></span>` : ""}
        ${botThinking ? `<span class="chip chip-pulse"><strong>${i.labels.thinking}</strong></span>` : ""}
        <span class="chip">Elo <strong>${state.bot_profile.elo}</strong></span>
      </div>
      <div class="board-frame" id="board-shell">
        <div class="board" id="board" role="grid" aria-label="Tabuleiro de xadrez"></div>
      </div>
      <div class="board-controls panel panel-pad" id="game-controls">
        ${customSliderHtml("elo", i.labels.elo, 100, 3000, 50, draftElo, String(draftElo))}
        ${customSelectHtml(
          "color",
          i.labels.yourColor,
          [
            { value: "white", label: i.color.white },
            { value: "black", label: i.color.black }
          ],
          draftColor
        )}
        ${
          view === "analysis"
            ? `<button class="btn" type="button" id="btn-analyze">${icons.spark}<span>${i.actions.analyze}</span></button>`
            : ""
        }
      </div>
    </section>
    <aside class="side-col">
      <section class="panel panel-pad">
        <h2>${view === "coach" ? i.labels.coachPanel : i.labels.lastMove}</h2>
        ${renderAnalysisBlock(state)}
      </section>
      <section class="panel panel-pad">
        <h2>${i.labels.history}</h2>
        <div class="history">
          ${
            state.history.length
              ? state.history
                  .map(
                    (h) => `
              <button class="history-item" type="button">
                <span class="ply">${h.ply}</span>
                <span class="san">${h.san}</span>
                <span class="muted">${h.analysis?.quality_label ?? ""}</span>
              </button>`
                  )
                  .join("")
              : `<div class="empty">${i.labels.emptyHistory}</div>`
          }
        </div>
      </section>
    </aside>
  `;

  const boardEl = document.getElementById("board");
  if (boardEl) {
    renderBoard(
      boardEl,
      state.board,
      {
        selected,
        targets,
        lastFrom: last ? last.uci.slice(0, 2) : null,
        lastTo: last ? last.uci.slice(2, 4) : null,
        checkSquare: checkSq,
        flipped: boardIsFlipped(),
        showCoords,
        animating
      },
      { onSquare: (sq) => void onSquareClick(sq) }
    );
  }

  if (pendingPromotion) showPromotionUi();

  const controls = document.getElementById("game-controls");
  if (controls) {
    bindSliders(controls, (id, value) => {
      if (id === "elo") draftElo = value;
    });
    bindSelects(controls, (id, value) => {
      if (id === "color") draftColor = value as "white" | "black";
    });
  }

  document.getElementById("btn-analyze")?.addEventListener("click", () => void runAnalyze());
}

function renderAnalysisBlock(s: GameStateDto): string {
  const i = t();
  const a = s.last_analysis;
  if (!a && !s.coach_tip) {
    return `<div class="empty">${i.labels.emptyHistory}</div>`;
  }
  if (!a) {
    return `<div class="tip"><strong>${i.coach.tipTitle}:</strong> ${s.coach_tip}</div>`;
  }
  return `
    <div class="analysis-card">
      <div class="quality" data-q="${a.quality}">
        <span class="quality-dot" aria-hidden="true"></span>
        <span>${a.quality_label}</span>
      </div>
      <p>${a.explanation}</p>
      <div class="board-meta">
        ${
          a.best_move_san
            ? `<span class="chip">${i.labels.bestMove} <strong>${a.best_move_san}</strong></span>`
            : ""
        }
      </div>
      ${
        a.tip || s.coach_tip
          ? `<div class="tip"><strong>${i.coach.tipTitle}:</strong> ${a.tip || s.coach_tip}</div>`
          : ""
      }
    </div>
  `;
}

function isHumanPiece(sq: string): boolean {
  if (!state) return false;
  const piece = state.board[squareToIndex(sq)];
  if (!piece || piece === ".") return false;
  const whitePiece = piece === piece.toUpperCase();
  return (state.config.human_color === "white") === whitePiece;
}

async function onSquareClick(sq: string) {
  if (!state || busy || botThinking || state.result !== "ongoing") return;
  if (state.side_to_move !== state.config.human_color) return;

  if (!selected) {
    if (!isHumanPiece(sq)) {
      shakeBoard();
      return;
    }
    selected = sq;
    try {
      targets = await api.legalTargets(sq);
    } catch {
      targets = [];
    }
    if (!targets.length) {
      shakeBoard();
      selected = null;
    }
    render();
    return;
  }

  if (selected === sq) {
    selected = null;
    targets = [];
    render();
    return;
  }

  if (!targets.includes(sq)) {
    if (!isHumanPiece(sq)) {
      selected = null;
      targets = [];
      render();
      return;
    }
    selected = sq;
    try {
      targets = await api.legalTargets(sq);
    } catch {
      targets = [];
    }
    render();
    return;
  }

  const from = selected;
  const to = sq;
  selected = null;
  targets = [];

  const piece = state.board[squareToIndex(from)];
  const isPawn = piece === "P" || piece === "p";
  const toRank = to[1];
  if (isPawn && (toRank === "8" || toRank === "1")) {
    pendingPromotion = { from, to };
    render();
    return;
  }

  await commitMove(from, to);
}

function shakeBoard() {
  const shell = document.getElementById("board-shell");
  shell?.classList.remove("shake");
  void shell?.offsetWidth;
  shell?.classList.add("shake");
}

function showPromotionUi() {
  const shell = document.getElementById("board-shell");
  if (!shell || !pendingPromotion || !state) return;
  const white = state.config.human_color === "white";
  const choices = white ? ["Q", "R", "B", "N"] : ["q", "r", "b", "n"];
  const overlay = document.createElement("div");
  overlay.className = "promo-overlay";
  overlay.innerHTML = `
    <div class="promo-box" role="dialog" aria-label="${t().actions.promoteAs}">
      ${choices
        .map((p) => `<button type="button" data-p="${p.toLowerCase()}">${pieceImg(p)}</button>`)
        .join("")}
    </div>`;
  overlay.querySelectorAll<HTMLButtonElement>("[data-p]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const promo = btn.dataset.p!;
      const pending = pendingPromotion!;
      pendingPromotion = null;
      void commitMove(pending.from, pending.to, promo);
    });
  });
  shell.appendChild(overlay);
}

async function commitMove(from: string, to: string, promotion?: string) {
  if (!state) return;
  busy = true;
  render();
  try {
    await syncModeWithView();
    // Paint human move immediately — do NOT wait for the bot here.
    state = await api.playMove(from, to, promotion);
    draftElo = state.config.bot_elo;
    draftColor = state.config.human_color;
    busy = false;
    render();
    await maybeBotReply(true);
  } catch (e) {
    toast(t().labels.illegal);
    console.error(e);
    busy = false;
    render();
  }
}

async function maybeBotReply(animate: boolean) {
  if (!state) return;
  if (state.result !== "ongoing") return;
  if (state.side_to_move === state.config.human_color) return;

  botThinking = true;
  render();

  // Small beat so the human move is visible before the bot answers.
  await sleep(180);

  try {
    const prevLen = state.history.length;
    state = await api.engineMove();
    botThinking = false;

    if (animate && state.history.length > prevLen) {
      const botMove = state.history[state.history.length - 1];
      const from = botMove.uci.slice(0, 2);
      const to = botMove.uci.slice(2, 4);
      const piece = state.board[squareToIndex(to)];
      if (piece && piece !== ".") {
        await playBotAnimation(from, to, piece);
      }
    }
  } catch (e) {
    console.error(e);
    botThinking = false;
  }
  render();
}

async function playBotAnimation(from: string, to: string, piece: string) {
  if (!state) return;

  animating = { from, to, piece };
  render();

  const board = document.getElementById("board");
  if (!board) {
    animating = null;
    return;
  }

  const start = squareCenter(board, from, boardIsFlipped());
  const end = squareCenter(board, to, boardIsFlipped());
  const src = pieceSrc(piece);
  if (!src) {
    animating = null;
    return;
  }

  const flyer = document.createElement("img");
  flyer.className = "piece-flyer";
  flyer.src = src;
  flyer.alt = "";
  const cell = board.clientWidth / 8;
  flyer.style.width = `${cell * 0.92}px`;
  flyer.style.height = `${cell * 0.92}px`;
  flyer.style.left = `${start.x}px`;
  flyer.style.top = `${start.y}px`;
  board.appendChild(flyer);

  void flyer.offsetWidth;
  flyer.style.transition = `transform ${BOT_MOVE_MS}ms cubic-bezier(0.22, 1, 0.36, 1)`;
  flyer.style.transform = `translate(${end.x - start.x}px, ${end.y - start.y}px) translate(-50%, -50%)`;

  await sleep(BOT_MOVE_MS + 20);
  flyer.remove();
  animating = null;
}

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function startNewGame() {
  if (!state) return;
  const mode: PlayMode =
    view === "coach" ? "coach" : view === "analysis" ? "analysis" : "versus_bot";
  const main = document.getElementById("main");
  if (main) {
    const elo = readSlider(main, "elo");
    const color = readSelect(main, "color") as "white" | "black" | null;
    if (elo != null) draftElo = elo;
    if (color) draftColor = color;
  }
  const config: GameConfig = {
    mode,
    human_color: draftColor,
    bot_elo: draftElo,
    lang: "pt"
  };
  busy = true;
  render();
  try {
    state = await api.newGame(config);
    selected = null;
    targets = [];
    pendingPromotion = null;
    animating = null;
    busy = false;
    render();
    await maybeBotReply(true);
  } catch (e) {
    toast(String(e));
    busy = false;
    render();
  }
}

async function runAnalyze() {
  busy = true;
  render();
  try {
    const a = await api.analyzeCurrent();
    if (state) state = { ...state, last_analysis: a, coach_tip: a.tip ?? null };
  } catch (e) {
    toast(String(e));
  } finally {
    busy = false;
    render();
  }
}

function renderDocs(main: HTMLElement) {
  const i = t();
  const pages = searchDocs(docsQuery);
  const groups = [...new Set(pages.map((p) => p.group))];
  const current = DOC_PAGES.find((p) => p.id === docsPageId) ?? pages[0] ?? DOC_PAGES[0];

  main.innerHTML = `
    <nav class="panel docs-nav" aria-label="${i.docs.toc}">
      <input class="docs-search" id="docs-search" type="search" placeholder="${i.docs.search}" value="${escapeAttr(
        docsQuery
      )}" />
      ${groups
        .map((g) => {
          const links = pages
            .filter((p) => p.group === g)
            .map(
              (p) =>
                `<button class="docs-link" type="button" data-doc="${p.id}" aria-current="${
                  p.id === current.id
                }">${p.title}</button>`
            )
            .join("");
          return `<div class="docs-group"><h3>${g}</h3>${links}</div>`;
        })
        .join("")}
    </nav>
    <article class="panel docs-article" id="docs-article">${renderMarkdown(current.body)}</article>
  `;

  document.getElementById("docs-search")?.addEventListener("input", (e) => {
    docsQuery = (e.target as HTMLInputElement).value;
    render();
  });
  main.querySelectorAll<HTMLButtonElement>("[data-doc]").forEach((btn) => {
    btn.addEventListener("click", () => {
      docsPageId = btn.dataset.doc!;
      render();
      document.getElementById("docs-article")?.scrollTo({ top: 0, behavior: "smooth" });
    });
  });
}

function renderSettings(main: HTMLElement) {
  const i = t();
  main.innerHTML = `
    <div class="settings-page">
      <section class="settings-card panel">
        <header class="settings-card-head">
          <h2>Aparência</h2>
          <p>Escolha como a janela se apresenta no seu sistema.</p>
        </header>
        <div class="settings-card-body">
          ${segmentedHtml("theme", i.settings.theme, [
            { value: "light", label: i.actions.themeLight },
            { value: "dark", label: i.actions.themeDark },
            { value: "system", label: i.actions.themeSystem }
          ], theme)}
        </div>
      </section>

      <section class="settings-card panel">
        <header class="settings-card-head">
          <h2>Tabuleiro</h2>
          <p>Detalhes visuais durante a partida.</p>
        </header>
        <div class="settings-card-body">
          ${toggleHtml("coords", i.settings.boardCoords, showCoords)}
        </div>
      </section>

      <section class="settings-card panel">
        <header class="settings-card-head">
          <h2>Idioma</h2>
          <p>${i.settings.languageNote}</p>
        </header>
        <div class="settings-card-body">
          <div class="settings-static">
            <span class="settings-static-label">Interface</span>
            <span class="settings-static-value">Português (Brasil)</span>
          </div>
        </div>
      </section>

      <section class="settings-card panel">
        <header class="settings-card-head">
          <h2>Sobre</h2>
          <p>Engine própria com bots Elo 100–3000, revisão de lances e modo treinador.</p>
        </header>
        <div class="settings-card-body">
          <div class="settings-static">
            <span class="settings-static-label">Peças</span>
            <span class="settings-static-value">Lichess Companion</span>
          </div>
          <div class="settings-static">
            <span class="settings-static-label">Versão</span>
            <span class="settings-static-value">0.1.0</span>
          </div>
        </div>
      </section>
    </div>
  `;

  bindSegmented(main, (id, value) => {
    if (id === "theme") {
      theme = value as ThemeMode;
      localStorage.setItem(THEME_KEY, theme);
      applyTheme(theme);
      render();
    }
  });
  bindToggles(main, (id, checked) => {
    if (id === "coords") {
      showCoords = checked;
      localStorage.setItem(COORDS_KEY, showCoords ? "1" : "0");
    }
  });
}

function applyTheme(mode: ThemeMode) {
  document.documentElement.dataset.theme = mode;
  const meta = document.querySelector('meta[name="theme-color"]');
  const dark =
    mode === "dark" ||
    (mode === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  meta?.setAttribute("content", dark ? "#121511" : "#efe8da");
}

function toast(message: string) {
  const el = document.createElement("div");
  el.className = "toast";
  el.textContent = message;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), 2800);
}

function escapeAttr(s: string): string {
  return s.replace(/"/g, "&quot;");
}
