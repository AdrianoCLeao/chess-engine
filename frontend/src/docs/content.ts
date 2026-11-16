/**
 * In-app documentation (Portuguese UI). This is intentionally long and detailed —
 * closer to an open-source library manual than a quick-start card.
 */

import { EXTRA_DOC_PAGES } from "./content-extra";

export type DocPage = {
  id: string;
  title: string;
  group: string;
  body: string;
};

const CORE_DOC_PAGES: DocPage[] = [
  {
    id: "intro",
    group: "Começar",
    title: "Visão geral do projeto",
    body: `
# Visão geral

**Chess Engine** é uma aplicação desktop multiplataforma (Linux, macOS e Windows) construída com **Tauri 2** + **Rust** + **TypeScript**. O objetivo não é apenas “um tabuleiro bonito”: o repositório entrega uma **engine completa**, com:

1. Validação rigorosa de movimentos (incluindo roque, en passant, promoção, xeque, mate e afogamento).
2. Níveis de força alinhados a uma escala **Elo 100–3000**, no espírito dos bots do Chess.com.
3. Análise qualitativa de **cada lance** (Brilhante → Péssimo) com **explicação em português legível**.
4. Modo **Treinador**, que joga no Elo escolhido e ensina a cada movimento.
5. Documentação profunda **dentro da própria GUI**.

Não há pipeline de deploy ou “build de produção” obrigatório neste projeto — o foco é desenvolvimento local, experimentação e clareza da metodologia.

## Arquitetura em uma frase

O Rust é a fonte da verdade do xadrez (regras, busca, Elo, classificação, i18n de explicações). A GUI Tauri é uma casca nativa que consome comandos tipados e apresenta o jogo, o treinador e esta documentação.
`
  },
  {
    id: "architecture",
    group: "Começar",
    title: "Arquitetura do código",
    body: `
# Arquitetura do código

## Camadas

| Camada | Pasta | Responsabilidade |
| --- | --- | --- |
| Regras + estado | \`src-tauri/src/engine/board.rs\`, \`movegen.rs\`, \`types.rs\` | Tabuleiro, FEN, geração legal |
| Avaliação | \`engine/eval.rs\` | Material + piece-square tables |
| Busca | \`engine/search.rs\` | Alpha-beta, deepening, quiescência |
| Elo | \`engine/elo.rs\` | Limites de força e ruído |
| Análise | \`engine/analysis.rs\` | Qualidade do lance + parâmetros |
| Treinador | \`engine/coach.rs\` | Feedback pedagógico |
| Explicações | \`src-tauri/src/i18n/\` | Locales (hoje: \`pt\`) |
| API desktop | \`src-tauri/src/api.rs\` | Comandos Tauri |
| GUI | \`frontend/src/\` | Tabuleiro, modos, docs, temas |

## Fluxo de um lance humano

1. A UI chama \`play_move\` com origem, destino e promoção opcional.
2. O backend normaliza flags (roque, en passant, promoção) e valida legalidade.
3. Antes de aplicar o lance, a posição é analisada: a engine pontua candidatos e mede a **perda em centipeões** vs. o melhor.
4. Um \`explanation_key\` + \`ExplanationParams\` é escolhido; o locale renderiza o texto.
5. O lance é aplicado; se o modo for Versus/Treinador e for a vez do bot, a engine responde com \`choose_move\` no Elo configurado.

## Por que Tauri (e não um site)?

Tauri entrega janela nativa, ícone na dock/barra de tarefas, empacotamento por SO e um backend Rust no mesmo processo — ideal para uma engine CPU-bound sem “deploy web”.
`
  },
  {
    id: "rules",
    group: "Engine",
    title: "Regras e validação de movimentos",
    body: `
# Regras e validação de movimentos

## Representação

- Casas \`0..63\` com **a1 = 0** e **h8 = 63** (little-endian rank-file).
- Cada casa guarda \`Option<Piece>\` (\`Color\` + \`PieceKind\`).
- Direitos de roque, casa de en passant, relógios de meio-lance e número do lance completo seguem a semântica FEN.

## Geração

1. **Pseudo-legais**: peões (passo simples/duplo, capturas, EP, promoções), saltadores, deslizadores e roques com casas vazias e não atacadas no caminho do rei.
2. **Filtro legal**: o lance é aplicado numa cópia; se o rei da cor que moveu permanece atacado, o lance é descartado.

## Ataques

\`is_square_attacked\` considera peões na direção correta, cavalos, rei e raios de bispo/torre/dama. É a base de xeque, roque e do filtro legal.

## Resultados

- Sem lances legais + rei em xeque → **mate** (vitória do oponente).
- Sem lances legais + rei seguro → **afogamento** (empate).
- \`halfmove_clock >= 100\` → empate por regra dos 50 lances (implementação pragmática).

## Testes de fumaça

O módulo de movimento inclui asserts clássicos (20 lances no início; mate do bobo). Rode \`make test\` para exercitá-los.
`
  },
  {
    id: "eval-search",
    group: "Engine",
    title: "Avaliação e busca",
    body: `
# Avaliação e busca

## Avaliação estática

A função \`evaluate\` devolve centipeões **do ponto de vista das brancas**:

- Material padrão (P=100, N=320, B=330, R=500, Q=900).
- **Piece-square tables** (PST) por tipo de peça.
- Bônus de par de bispos (±30).

A busca trabalha com score **relativo à cor que move** (\`evaluate_relative\`).

## Alpha-beta

\`Searcher\` implementa:

- **Deepening iterativo** até \`max_depth\`.
- Corte por **nós** e **tempo**.
- Ordenação de lances (capturas MVV-ish, promoções, roque, hint do lance anterior).
- **Quiescência** em capturas para reduzir o horizonte em trocas.

Mate é representado por um score grande (\`MATE_SCORE\`).

## Limites

Os limites concretos não são “infinitos”: mesmo no Elo 3000 há teto de profundidade/nós/tempo para manter a UI responsiva em desenvolvimento. Aumentar esses tetos é o principal botão para força bruta adicional.
`
  },
  {
    id: "elo",
    group: "Engine",
    title: "Sistema Elo 100–3000",
    body: `
# Sistema Elo 100–3000

A escala foi desenhada para **lembrar a escada de bots do Chess.com**: iniciantes erram muito e calculam pouco; níveis de mestre calculam fundo e quase não “escapam” do melhor lance.

## O que o Elo controla

| Parâmetro | Efeito |
| --- | --- |
| \`depth\` | Profundidade máxima da busca |
| \`max_nodes\` / \`think_ms\` | Orçamento de CPU/tempo |
| \`noise_cp\` | Ruído aleatório somado aos scores dos candidatos |
| \`candidate_span\` | Quão longe na lista ordenada o bot pode escolher |
| \`blunder_chance\` | Probabilidade de escolher deliberadamente um lance pior |

## Faixas aproximadas

- **100–400**: profundidade 1, muito ruído, às vezes lance quase aleatório.
- **600–1000**: começa a preferir capturas e desenvolver, ainda com erros frequentes.
- **1200–1600**: cálculo curto coerente; imprecisões clássicas.
- **1800–2200**: poucos blunders graves; profundidade média.
- **2400–2800**: busca profunda, pouca permissão para cair na lista.
- **3000**: perfil “melhor esforço” da engine neste repositório.

## Presets da UI

A interface oferece degraus (\`100, 250, 400, …, 3000\`), mas qualquer inteiro em \`[100, 3000]\` é aceito via \`EloProfile::from_elo\`.

## Importante

Elo aqui é uma **metáfora de força controlada**, não uma medição estatística de rating real em pool humano. Serve para treinar e comparar comportamento, não para emitir certificados de rating.
`
  },
  {
    id: "classification",
    group: "Engine",
    title: "Classificação de lances",
    body: `
# Classificação de lances (estilo Chess.com)

Depois de cada lance (humano ou bot), a engine:

1. Pontua os lances legais na posição **anterior**.
2. Mede \`loss_cp = best_score - played_score\` (nunca negativo).
3. Observa sinais táticos/estruturais (material, peça pendurada, rei exposto, roque, xeque, sacrifício correto).
4. Emite um rótulo qualitativo + chave de explicação.

## Faixas de perda (centipeões)

| Rótulo | Critério típico |
| --- | --- |
| Brilhante | Melhor lance **e** sacrifício de material |
| Melhor | Top da engine |
| Excelente | Perda ≤ ~10 |
| Ótimo | Perda ≤ ~25 |
| Bom | Perda ≤ ~50 |
| Impreciso | Perda ≤ ~100 |
| Erro | Perda ≤ ~200 |
| Lance perdido | Perda ≤ ~300 |
| Péssimo | Perda > ~300 |

Os limiares são pragmáticos e documentados em \`analysis.rs\`. Em aberturas muito teóricas, um futuro livro de aberturas poderia marcar \`Book\`.

## Avaliação mostrada

A UI exibe avaliação absoluta das brancas (positivo = vantagem branca), antes/depois do lance, além da perda relativa ao melhor.
`
  },
  {
    id: "explanations",
    group: "Engine",
    title: "Explicações e i18n",
    body: `
# Explicações e internacionalização

## Contrato estável

A análise **não** concatena frases em português dentro do classificador. Ela produz:

- \`explanation_key\` — identificador estável (\`blunder_king_threat\`, \`good_castle\`, …)
- \`ExplanationParams\` — SAN jogado, melhor SAN, perda, peças envolvidas, flags booleanas

O módulo \`i18n\` escolhe um \`Locale\` e renderiza texto natural.

## Português hoje

\`i18n/pt.rs\` gera frases no formato pedido, por exemplo:

> “Esse movimento foi péssimo pois ele expôs o seu rei ao cavalo inimigo…”

## Como adicionar um idioma

1. Criar \`src-tauri/src/i18n/en.rs\` (ou outro) implementando \`Locale\`.
2. Registrar em \`locale_for\`.
3. Estender o enum \`Lang\` e o parser.
4. No frontend, adicionar um catálogo em \`frontend/src/i18n/\` e um seletor.

A GUI já trata idioma como configuração (\`config.lang\`), mesmo que só \`pt\` esteja completo.
`
  },
  {
    id: "coach-mode",
    group: "Modos",
    title: "Modo treinador",
    body: `
# Modo treinador

No modo **Treinador**:

1. Você escolhe o Elo do coach.
2. Joga normalmente no tabuleiro.
3. Após **cada** lance seu, a engine publica:
   - classificação + explicação;
   - uma **dica** (elogio se o lance foi forte; sugestão do melhor lance se foi fraco);
4. O coach responde jogando com o perfil Elo selecionado — ele **não** joga em força máxima “escondida”; ele ensina no nível combinado.

## Diferença para “Contra o bot”

No Versus Bot a análise ainda ocorre (histórico completo), mas o painel enfatiza menos a pedagogia. No Treinador, o painel lateral é dedicado às dicas.

## Boas práticas de treino

- Comece 200–400 Elo abaixo do seu nível percebido para focar em padrões, não em sobrevivência.
- Leia a explicação **antes** de olhar só o melhor lance.
- Use a documentação de classificação para entender por que “impreciso” ≠ “péssimo”.
`
  },
  {
    id: "gui",
    group: "Interface",
    title: "Interface desktop",
    body: `
# Interface desktop

## Stack

- **Tauri 2** janela nativa com ícones \`.icns\` / \`.ico\` / PNG.
- Frontend **Vite + TypeScript** (sem framework obrigatório) para UI leve e controlada.
- Temas **claro** e **escuro** via tokens CSS (\`data-theme\`), com opção de seguir o sistema.
- Tipografia: **Fraunces** (display) + **IBM Plex Sans/Mono** (UI/código).

## Superfícies

- **Jogar** — versus bot com seletor de Elo e cor.
- **Treinador** — mesmo motor com painel pedagógico.
- **Análise** — peça livremente e peça sugestões.
- **Documentação** — este manual completo com busca.
- **Ajustes** — tema, idioma (preparado), preferências de tabuleiro.

## Detalhes de craft aplicados

- Foco visível (\`:focus-visible\`) em todos os controles.
- \`prefers-reduced-motion\` reduz animações.
- Scrollbars discretas alinhadas à superfície.
- Coordenadas do tabuleiro e alvos legais sem poluir o estado de seleção.
- Microcopy em português com substantivos estáveis (“lance”, “Elo”, “treinador”).
`
  },
  {
    id: "makefile",
    group: "Desenvolvimento",
    title: "Makefile e comandos",
    body: `
# Makefile e comandos

Na raiz do repositório:

| Comando | Função |
| --- | --- |
| \`make dev\` | Sobe a app Tauri em modo desenvolvimento |
| \`make frontend\` | Só o Vite (útil para UI sem Rust) |
| \`make check\` | \`cargo check\` no backend |
| \`make test\` | Testes Rust da engine |
| \`make icons\` | Regenera ícones a partir de \`icon.png\` |
| \`make help\` | Lista alvos |

## Pré-requisitos

- Rust estável (\`rustup\` / Homebrew)
- Node.js + npm
- Dependências Tauri do seu SO ([guia oficial](https://v2.tauri.app/start/prerequisites/))

## Sem deploy

Não há alvos de publicação em lojas ou CI de release neste Makefile de propósito: o projeto prioriza o ciclo local.
`
  },
  {
    id: "api-commands",
    group: "Desenvolvimento",
    title: "Comandos Tauri (API)",
    body: `
# Comandos Tauri (API)

| Comando | Entrada | Saída |
| --- | --- | --- |
| \`get_state\` | — | \`GameStateDto\` |
| \`new_game\` | \`GameConfig\` | estado inicial (bot pode mover se humanas são pretas) |
| \`legal_targets\` | casa origem | casas destino legais |
| \`play_move\` | from/to/promo | estado após humano (+ bot se aplicável) |
| \`engine_move\` | — | força resposta / lance em análise |
| \`analyze_current\` | — | melhor lance sugerido + texto |
| \`list_elo_levels\` | — | presets |
| \`elo_profile\` | elo | profundidade/tempo/ruído |
| \`methodology_summary\` | — | resumo curto espelhando esta doc |

Todos os DTOs são serializados com Serde; o frontend tipa-os em \`frontend/src/api/types.ts\`.
`
  },
  {
    id: "limitations",
    group: "Desenvolvimento",
    title: "Limitações conhecidas e extensões",
    body: `
# Limitações conhecidas e extensões naturais

## Limitações atuais

- Sem tabela de transposição / bitboards — a busca é clara e didática, não Stockfish.
- Sem livro de aberturas dedicado (rótulo \`Book\` está preparado, pouco usado).
- Empates por repetição tripla / material insuficiente não estão todos modelados com rigor de torneio.
- Explicações cobrem os padrões mais comuns; posições quietas caem em templates genéricos com SAN + perda.
- Ícone \`.ico\` gerado de forma pragmática; para lojas oficiais, use \`cargo tauri icon\`.

## Extensões sugeridas

1. TT + killer moves + null move para Elo alto.
2. Livro de aberturas JSON + marcação \`Book\`.
3. Locales \`en\` / \`es\` espelhando as chaves.
4. Exportação PGN da partida analisada.
5. Puzzle mode reutilizando o classificador.
`
  },
  {
    id: "glossary",
    group: "Referência",
    title: "Glossário",
    body: `
# Glossário

- **Centipeão (cp)** — 1/100 de peão; unidade da avaliação.
- **Elo (neste app)** — parâmetro de força do bot/treinador, não rating FIDE oficial.
- **PST** — piece-square table; valor posicional por casa.
- **Quiescência** — extensão da busca só em capturas para estabilizar a eval.
- **SAN** — notação algébrica curta (\`Nf3\`, \`O-O\`).
- **UCI** — notação de coordenadas (\`e2e4\`, \`e7e8q\`).
- **Perda (loss_cp)** — quanto o lance ficou abaixo do melhor da engine.
`
  }
];

export const DOC_PAGES: DocPage[] = [...CORE_DOC_PAGES, ...EXTRA_DOC_PAGES];

export function searchDocs(query: string): DocPage[] {
  const q = query.trim().toLowerCase();
  if (!q) return DOC_PAGES;
  return DOC_PAGES.filter(
    (p) =>
      p.title.toLowerCase().includes(q) ||
      p.group.toLowerCase().includes(q) ||
      p.body.toLowerCase().includes(q)
  );
}
