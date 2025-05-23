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
                    if ui.button("New File").clicked() {
                        self.editor.new_file();
                        ui.close_menu();
                    }
                    if ui.button("Open File").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.editor.open_file(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Open Folder").clicked(){
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.editor.open_folder(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Save File").clicked() {
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
                    
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .max_width(f32::INFINITY)
                        .show(ui, |ui| {
                            ui.set_min_width(200.0);
                            
                            if let Some(workspace_folder) = self.editor.get_workspace_folder().cloned() {
                                if let Some(folder_name) = workspace_folder.file_name().and_then(|n| n.to_str()) {
                                    ui.label(format!("📁 {}", folder_name));
                                    ui.separator();
                                }
                                self.show_directory_tree(ui, &workspace_folder, 0);
                            } else {
                                ui.label("No folder opened");
                                ui.separator();
                                if ui.button("Open Folder").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.editor.open_folder(path);
                                    }
                                }
                            }
                        });
                });
        }
    }

    fn show_directory_tree(&mut self, ui: &mut egui::Ui, path: &PathBuf, depth: usize) {
        if depth > 5 { return; }
        
        if let Ok(entries) = fs::read_dir(path) {
            let mut entries: Vec<_> = entries.flatten().collect();
            
            entries.sort_by(|a, b| {
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();
                
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.file_name().cmp(&b.file_name()),
                }
            });
            
            for entry in entries {
                let entry_path = entry.path();
                let name = entry_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");
                
                if name.starts_with('.') || 
                   name == "target" || 
                   name == "node_modules" || 
                   name == "__pycache__" ||
                   name == ".git" {
                    continue;
                }
                
                ui.horizontal(|ui| {
                    ui.add_space(depth as f32 * 16.0);
                    
                    if entry_path.is_dir() {
                        let dir_path = entry_path.clone();
                        let response = ui.collapsing(format!("📁 {}", name), |ui| {
                            self.show_directory_tree(ui, &dir_path, depth + 1);
                        });
                        
                        if response.header_response.double_clicked() {
                            // Double-click to expand/collapse
                        }
                    } else {
                        let icon = match entry_path.extension().and_then(|e| e.to_str()) {
                            Some("rs") => "🦀",
                            Some("py") => "🐍",
                            Some("js") | Some("ts") => "📜",
                            Some("html") => "🌐",
                            Some("css") => "🎨",
                            Some("json") => "📋",
                            Some("md") => "📝",
                            Some("toml") | Some("yaml") | Some("yml") => "⚙️",
                            Some("txt") => "📄",
                            _ => "📄",
                        };
                        
                        let file_path = entry_path.clone();
                        let button_text = format!("{} {}", icon, name);
                        if ui.add(egui::Button::new(button_text).wrap(false)).clicked() {
                            self.editor.open_file(file_path);
                        }
                    }
                });
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