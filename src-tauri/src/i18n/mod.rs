//! Internationalization for human-readable move explanations.
//!
//! The engine emits stable `explanation_key` values plus structured parameters.
//! Locales render those into natural language. Today only Portuguese (`pt`) is
//! shipped; adding a language means implementing [`Locale`] and registering it
//! in [`locale_for`].

pub mod pt;

use crate::engine::analysis::ExplanationParams;
use serde::{Deserialize, Serialize};

/// Supported UI / explanation locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Lang {
    #[default]
    Pt,
    // Future: En, Es, …
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Pt => "pt",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "pt" | "pt-br" | "pt_br" | "portuguese" => Lang::Pt,
            _ => Lang::Pt, // fallback until more locales exist
        }
    }
}

/// Locale-specific string rendering.
pub trait Locale: Send + Sync {
    fn lang(&self) -> Lang;
    fn piece_name(&self, fen_char: &str) -> String;
    fn quality_label(&self, quality: &str) -> String;
    fn explain(&self, key: &str, params: &ExplanationParams) -> String;
}

pub fn locale_for(lang: Lang) -> Box<dyn Locale> {
    match lang {
        Lang::Pt => Box::new(pt::Portuguese),
    }
}

pub fn render_explanation(lang: Lang, key: &str, params: &ExplanationParams) -> String {
    locale_for(lang).explain(key, params)
}
