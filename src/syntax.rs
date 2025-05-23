use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::easy::HighlightLines;
use egui::Color32;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();
        
        Self {
            syntax_set,
            theme,
        }
    }    pub fn highlight_line(&mut self, line: &str, language: &str) -> Vec<(String, Color32)> {
        let syntax = self.syntax_set.find_syntax_by_extension(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let ranges = highlighter.highlight_line(line, &self.syntax_set).unwrap_or_default();
        
        ranges.into_iter().map(|(style, text)| {
            let color = Color32::from_rgb(
                (style.foreground.r as f32 * 255.0) as u8,
                (style.foreground.g as f32 * 255.0) as u8,
                (style.foreground.b as f32 * 255.0) as u8,
            );
            (text.to_string(), color)
        }).collect()
    }
}