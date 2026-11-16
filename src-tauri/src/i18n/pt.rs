//! Portuguese (Brazil) explanations — written for a human player, never for an engine log.

use super::Locale;
use crate::engine::analysis::ExplanationParams;
use crate::i18n::Lang;

pub struct Portuguese;

impl Locale for Portuguese {
    fn lang(&self) -> Lang {
        Lang::Pt
    }

    fn piece_name(&self, fen_char: &str) -> String {
        match fen_char.to_ascii_lowercase().as_str() {
            "p" => "peão".into(),
            "n" => "cavalo".into(),
            "b" => "bispo".into(),
            "r" => "torre".into(),
            "q" => "dama".into(),
            "k" => "rei".into(),
            _ => "peça".into(),
        }
    }

    fn quality_label(&self, quality: &str) -> String {
        match quality {
            "brilliant" => "Brilhante".into(),
            "great" => "Ótimo".into(),
            "best" => "Melhor".into(),
            "excellent" => "Excelente".into(),
            "good" => "Bom".into(),
            "book" => "De abertura".into(),
            "inaccuracy" => "Impreciso".into(),
            "mistake" => "Erro".into(),
            "miss" => "Lance perdido".into(),
            "blunder" => "Péssimo".into(),
            _ => quality.to_string(),
        }
    }

    fn explain(&self, key: &str, params: &ExplanationParams) -> String {
        let piece = |opt: &Option<String>| {
            opt.as_ref()
                .map(|c| self.piece_name(c))
                .unwrap_or_else(|| "peça".into())
        };
        let article = |opt: &Option<String>| -> &'static str {
            match opt
                .as_ref()
                .map(|c| c.to_ascii_lowercase())
                .as_deref()
            {
                Some("p") | Some("b") | Some("r") | Some("q") => "a",
                _ => "o",
            }
        };
        let threat = piece(&params.threat_piece);
        let hanging = piece(&params.hanging_piece);
        let captured = piece(&params.captured_piece);
        let best = if params.best_san.is_empty() {
            "outro lance".to_string()
        } else {
            params.best_san.clone()
        };
        let sq = params
            .hanging_square
            .clone()
            .unwrap_or_else(|| "uma casa importante".into());
        let how_bad = loss_in_words(params.loss_cp);

        match key {
            "brilliant_sacrifice" => format!(
                "Esse movimento foi brilhante! Com {} você entregou material de propósito — e estava certo: era o caminho mais forte da posição.",
                params.played_san
            ),
            "good_castle" => {
                "Esse movimento foi ótimo pois ao rocar você colocou o rei em segurança e ligou as torres para o meio-jogo.".into()
            }
            "good_check" => format!(
                "Esse movimento foi forte pois {} dá xeque e obriga o adversário a responder, ganhando tempo para desenvolver o resto das peças.",
                params.played_san
            ),
            "good_wins_material" => format!(
                "Esse movimento foi excelente pois com {} você ganhou material — saiu na frente na troca de peças.",
                params.played_san
            ),
            "good_best_line" => format!(
                "Esse movimento foi o melhor da posição. {} mantém o plano certo e não deixa o adversário respirar.",
                params.played_san
            ),
            "good_solid" => format!(
                "Esse movimento foi bom e sólido. Mantém a posição sob controle, embora {} fosse um pouquinho mais preciso.",
                best
            ),
            "book_move" => format!(
                "Esse movimento é clássico de abertura: {} aparece com frequência em partidas bem jogadas no início do jogo.",
                params.played_san
            ),
            "inaccuracy_hangs" => format!(
                "Esse movimento foi impreciso pois depois dele {} seu {} em {} fica vulnerável — o adversário pode atacá-lo com vantagem.",
                article(&params.hanging_piece),
                hanging,
                sq
            ),
            "inaccuracy_better_exists" => format!(
                "Esse movimento foi impreciso porque {} {}. Em vez disso, {} mantinha melhor a posição.",
                how_bad, params.played_san, best
            ),
            "mistake_king_exposed" => format!(
                "Esse movimento foi um erro pois ele expôs o seu rei ao {} inimigo, abrindo caminho para ameaças perigosas.",
                threat
            ),
            "mistake_hangs" => format!(
                "Esse movimento foi um erro pois deixou {} {} desprotegido em {}. O adversário pode capturá-lo de graça.",
                article(&params.hanging_piece),
                hanging,
                sq
            ),
            "mistake_loses_material" => format!(
                "Esse movimento foi um erro pois você perde material sem boa compensação. O caminho certo era {}.",
                best
            ),
            "mistake_better_exists" => format!(
                "Esse movimento foi um erro: {} {}. O lance {} era bem mais forte e mantinha a vantagem.",
                how_bad, params.played_san, best
            ),
            "blunder_king_threat" => format!(
                "Esse movimento foi péssimo pois ele expôs o seu rei ao {} inimigo, permitindo ameaças graves ou até mate.",
                threat
            ),
            "blunder_hangs" => format!(
                "Esse movimento foi péssimo pois entregou {} {} em {} sem nenhuma compensação.",
                article(&params.hanging_piece),
                hanging,
                sq
            ),
            "blunder_loses_material" => format!(
                "Esse movimento foi péssimo pois você perde uma peça importante. O lance correto era {}.",
                best
            ),
            "blunder_generic" => format!(
                "Esse movimento foi péssimo porque {} {}. {} era bem superior e salvava a posição.",
                how_bad, params.played_san, best
            ),
            "coach_hint_best" => format!(
                "Dica do treinador: pense em {}. É o lance mais forte agora e resolve o problema da posição.",
                best
            ),
            "coach_praise" => format!(
                "Muito bem! {} foi um lance {}. Continue nesse ritmo.",
                params.played_san,
                self.quality_label(&params.quality).to_lowercase()
            ),
            "coach_warn" => format!(
                "Cuidado: {} não foi o melhor caminho. Olhe de novo a posição — {} merecia mais atenção.",
                params.played_san, best
            ),
            _ => {
                if params.captured_piece.is_some() {
                    format!(
                        "Com {} você mexeu na {} adversária. Ainda assim, {} era o caminho mais preciso.",
                        params.played_san, captured, best
                    )
                } else if params.loss_cp >= 100 {
                    format!(
                        "Esse movimento foi fraco porque {} {}. Vale a pena comparar com {}, que mantinha a posição bem melhor.",
                        how_bad, params.played_san, best
                    )
                } else {
                    format!(
                        "Esse movimento até funciona, mas {} era mais preciso e mantinha melhor o controle da posição.",
                        best
                    )
                }
            }
        }
    }
}

/// Translate centipawn loss into player language — never mention "centipeões".
fn loss_in_words(loss_cp: i32) -> &'static str {
    if loss_cp <= 30 {
        "fica só um pouco abaixo do ideal"
    } else if loss_cp <= 80 {
        "abre mão de uma ideia melhor"
    } else if loss_cp <= 150 {
        "piora bastante a sua posição"
    } else if loss_cp <= 300 {
        "entrega uma vantagem clara ao adversário"
    } else {
        "estraga a posição de forma grave"
    }
}
