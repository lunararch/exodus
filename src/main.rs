use eframe::App;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use std::fs;
use std::path::PathBuf;

mod editor;
mod syntax;
mod config;
mod plugins;

use editor::Editor;
use syntax::SyntaxHighlighter;
use config::Config;
use plugins::PluginManager;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Exodus",
        options,
        Box::new(|cc| Box::new(Exodus::new(cc))),
    )
}

pub struct Exodus {
    editor: Editor,
    syntax_highlighter: SyntaxHighlighter,
    config: Config,
    plugin_manager: PluginManager,
    show_file_explorer: bool,
    file_explorer_width: f32,
    search_query: String,
    show_search: bool,
}

impl Exodus {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load().unwrap_or_default();
        
        // Configure egui style for minimalist appearance
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.window_rounding = egui::Rounding::ZERO;
        style.visuals.menu_rounding = egui::Rounding::ZERO;
        style.visuals.indent_has_left_vline = false;
        style.spacing.item_spacing = egui::vec2(4.0, 2.0);
        cc.egui_ctx.set_style(style);

        Self {
            editor: Editor::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
            config,
            plugin_manager: PluginManager::new(),
            show_file_explorer: true,
            file_explorer_width: 200.0,
            search_query: String::new(),
            show_search: false,
        }
    }

    fn menu_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.editor.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.editor.open_file(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        self.editor.save_current();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        self.editor.undo();
                        ui.close_menu();
                    }
                    if ui.button("Redo").clicked() {
                        self.editor.redo();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Find").clicked() {
                        self.show_search = !self.show_search;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Toggle File Explorer").clicked() {
                        self.show_file_explorer = !self.show_file_explorer;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn search_bar(&mut self, ctx: &Context) {
        if self.show_search {
            TopBottomPanel::top("search_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Find:");
                    let response = ui.text_edit_singleline(&mut self.search_query);
                    if response.changed() {
                        self.editor.highlight_search(&self.search_query);
                    }
                    if ui.button("×").clicked() {
                        self.show_search = false;
                        self.search_query.clear();
                        self.editor.clear_search_highlights();
                    }
                });
            });
        }
    }

    fn file_explorer(&mut self, ctx: &Context) {
        if self.show_file_explorer {
            SidePanel::left("file_explorer")
                .resizable(true)
                .default_width(self.file_explorer_width)
                .show(ctx, |ui| {
                    ui.heading("Files");
                    ui.separator();
                    
                    if let Ok(current_dir) = std::env::current_dir() {
                        self.show_directory_tree(ui, &current_dir, 0);
                    }
                });
        }
    }

    fn show_directory_tree(&mut self, ui: &mut egui::Ui, path: &PathBuf, depth: usize) {
        if depth > 3 { return; } // Limit recursion depth
        
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                
                if name.starts_with('.') { continue; }
                
                let indent = "  ".repeat(depth);
                
                if entry_path.is_dir() {
                    if ui.collapsing(format!("{}{} 📁", indent, name), |ui| {
                        self.show_directory_tree(ui, &entry_path, depth + 1);
                    }).header_response.clicked() {
                        // Handle directory click
                    }
                } else {
                    if ui.button(format!("{}📄 {}", indent, name)).clicked() {
                        self.editor.open_file(entry_path);
                    }
                }
            }
        }
    }
}

impl App for Exodus {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::N) {
                    self.editor.new_file();
                } else if i.key_pressed(egui::Key::O) {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.editor.open_file(path);
                    }
                } else if i.key_pressed(egui::Key::S) {
                    self.editor.save_current();
                } else if i.key_pressed(egui::Key::Z) {
                    self.editor.undo();
                } else if i.key_pressed(egui::Key::Y) {
                    self.editor.redo();
                } else if i.key_pressed(egui::Key::F) {
                    self.show_search = !self.show_search;
                }
            }
        });

        self.menu_bar(ctx);
        self.search_bar(ctx);
        self.file_explorer(ctx);

        CentralPanel::default().show(ctx, |ui| {
            self.editor.show(ui, &mut self.syntax_highlighter);
        });
    }
}