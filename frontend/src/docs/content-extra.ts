/** Additional deep-dive documentation pages merged into the docs index. */
import type { DocPage } from "./content";

export const EXTRA_DOC_PAGES: DocPage[] = [
  {
    id: "centipawns",
    group: "Engine",
    title: "Centipeões e horizonte",
    body: `
# Centipeões e horizonte

Um **centipeão** (cp) é 1/100 do valor de um peão. Dizer que a posição está em \`+120\` significa cerca de 1,2 peões de vantagem para as brancas na avaliação estática + busca.

## Por que não usar só material?

Material ignora atividade, rei exposto e peões passados. As PST e a quiescência tentam capturar parte disso sem uma rede neural.

## Horizonte

Toda busca tem horizonte: ameaças além da profundidade podem ser “invisíveis”. A quiescência mitiga trocas no último ply, mas não resolve combinações longas. Por isso Elo alto aumenta profundidade — e mesmo assim não é Stockfish.

## Perda vs. avaliação absoluta

A classificação do lance usa **perda relativa ao melhor candidato**, não o sinal absoluto da eval. Um lance pode ser “melhor” mesmo com eval negativa (você já estava perdido e achou a defesa mais tenaz).
`
  },
  {
    id: "move-pipeline",
    group: "Engine",
    title: "Pipeline interno de um lance",
    body: `
# Pipeline interno de um lance

\`\`\`
UI play_move
  → normalize_move (flags)
  → is_legal
  → analyze_move (score_all_moves + classify + explanation_key)
  → locale.explain (pt)
  → board.make_move
  → (opcional) choose_move(elo) para o bot
  → snapshot GameStateDto
\`\`\`

Cada etapa é testável isoladamente: FEN round-trip, contagem de lances legais, mate do bobo, perfis Elo.

## Threading

A busca corre no thread do comando Tauri. Em Elo muito alto a UI mostra “pensando…”. Não há worker pool ainda — extensão natural seria \`spawn_blocking\` + eventos de progresso.
`
  },
  {
    id: "ui-tokens",
    group: "Interface",
    title: "Sistema visual e acessibilidade",
    body: `
# Sistema visual e acessibilidade

## Tokens

Cores e tipografia vivem em \`frontend/src/styles/tokens.css\`. Temas claro/escuro/sistema trocam o mesmo conjunto de variáveis — componentes não hardcodam hex de marca.

Direção visual: **floresta / nogueira / metal quente**, evitando o cluster genérico roxo-neon e o cream+terracota editorial.

## Acessibilidade

- Anel \`:focus-visible\` em controles (accessibility-1).
- \`prefers-reduced-motion\` zera animações (accessibility-2 / motion gate).
- Tabuleiro em botões com \`aria-label\` por casa.
- Documentação pesquisável; seções longas em HTML real (não canvas opaco).

## Motion com freio

Seleção de casa e toasts usam motion curto. Ações de alta frequência (cliques no tabuleiro) não levam confetes — só feedback de alvo legal e shake em beco sem saída (motion-16).
`
  },
  {
    id: "testing",
    group: "Desenvolvimento",
    title: "Testes e diagnóstico",
    body: `
# Testes e diagnóstico

\`\`\`bash
make test      # testes do crate lib
make check     # cargo check
make typecheck # tsc no frontend
\`\`\`

## O que os testes cobrem hoje

- Round-trip FEN da posição inicial
- 20 lances legais no startpos
- Sequência do mate do bobo

## Diagnóstico manual

1. Abra \`make dev\`.
2. Elo 100 vs Elo 3000 e observe profundidade/ruído no chip da UI.
3. Force um erro óbvio (entregar a dama) e leia a explicação em português.
4. Alternar tema claro/escuro em Ajustes e confirmar \`theme-color\`.
`
  },
  {
    id: "fen-uci-san",
    group: "Referência",
    title: "FEN, UCI e SAN",
    body: `
# FEN, UCI e SAN

## FEN

String canônica da posição (\`Board::to_fen\` / \`from_fen\`). A GUI também recebe um vetor de 64 caracteres (\`.\` ou peça FEN) para renderizar sem reparsear.

## UCI

\`e2e4\`, \`e7e8q\` — usado na API e no histórico interno.

## SAN

Notação legível aproximada (\`Nf3\`, \`O-O\`, \`exd5\`). A implementação atual prioriza clareza nas explicações; não é um gerador SAN com desambiguação completa PGN-grade.
`
  },
  {
    id: "security-model",
    group: "Desenvolvimento",
    title: "Modelo de segurança Tauri",
    body: `
# Modelo de segurança Tauri

A capability \`src-tauri/capabilities/default.json\` libera APIs core para a janela \`main\`. Os comandos da engine (\`play_move\`, etc.) são do próprio app.

Não há acesso a filesystem arbitrário nem shell. O projeto não precisa de rede para jogar.

CSP está \`null\` em desenvolvimento para o Vite HMR; se um dia houver empacotamento rígido, feche a CSP.
`
  }
];
