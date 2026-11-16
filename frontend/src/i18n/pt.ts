/** Portuguese UI copy. Keep keys stable when adding languages. */
export const pt = {
  appName: "Chess Engine",
  tagline: "Jogue, aprenda e revise cada lance",

  nav: {
    play: "Jogar",
    coach: "Treinador",
    analysis: "Análise",
    docs: "Documentação",
    settings: "Ajustes"
  },

  modes: {
    versus_bot: "Partida contra o bot",
    coach: "Treino com explicações a cada lance",
    analysis: "Tabuleiro livre para explorar ideias"
  },

  color: {
    white: "Brancas",
    black: "Pretas"
  },

  actions: {
    newGame: "Nova partida",
    flipBoard: "Girar",
    askEngine: "Pedir lance",
    analyze: "Analisar",
    themeLight: "Claro",
    themeDark: "Escuro",
    themeSystem: "Sistema",
    promoteAs: "Promover peão"
  },

  labels: {
    elo: "Nível do bot",
    depth: "Profundidade",
    think: "Tempo",
    blunderChance: "Chance de erro",
    sideToMove: "Vez",
    result: "Resultado",
    evaluation: "Avaliação",
    history: "Lances",
    coachPanel: "Treinador",
    lastMove: "Último lance",
    bestMove: "Melhor ideia",
    loss: "Impacto",
    inCheck: "Xeque",
    emptyHistory: "Faça o primeiro lance para começar a revisão.",
    loading: "Abrindo a engine…",
    thinking: "Pensando…",
    illegal: "Esse lance não é permitido nesta posição.",
    yourMove: "Seu lance",
    engineMove: "Lance da engine",
    yourColor: "Sua cor",
    appearance: "Aparência"
  },

  results: {
    ongoing: "Em andamento",
    white_wins: "Brancas venceram",
    black_wins: "Pretas venceram",
    draw: "Empate"
  },

  settings: {
    title: "Ajustes",
    theme: "Tema",
    language: "Idioma",
    languageNote: "Por enquanto só português. A estrutura já aceita novos idiomas.",
    boardCoords: "Mostrar coordenadas no tabuleiro",
    animations: "Animações suaves"
  },

  coach: {
    intro: "O treinador joga no Elo escolhido e explica cada um dos seus lances em linguagem clara.",
    tipTitle: "Dica"
  },

  docs: {
    title: "Documentação",
    search: "Buscar…",
    toc: "Sumário"
  }
} as const;
