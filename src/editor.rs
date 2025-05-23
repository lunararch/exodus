use egui::{ScrollArea, TextEdit, Ui};
use std::path::PathBuf;
use std::fs;

pub struct Editor {
    tabs: Vec<EditorTab>,
    active_tab: usize,
    next_id: usize,
    workspace_folder: Option<PathBuf>,
}

pub struct EditorTab {
    id: usize,
    title: String,
    content: String,
    file_path: Option<PathBuf>,
    modified: bool,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    cursor_pos: usize,
    search_highlights: Vec<(usize, usize)>,
}

impl Editor {    pub fn new() -> Self {
        let mut editor = Self {
            tabs: Vec::new(),
            active_tab: 0,
            next_id: 0,
            workspace_folder: None,
        };
        editor.new_file();
        editor
    }

    pub fn new_file(&mut self) {
        let tab = EditorTab {
            id: self.next_id,
            title: format!("Untitled {}", self.next_id + 1),
            content: String::new(),
            file_path: None,
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            cursor_pos: 0,
            search_highlights: Vec::new(),
        };
        
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        self.next_id += 1;
    }

    pub fn open_file(&mut self, path: PathBuf) {
        if let Ok(content) = fs::read_to_string(&path) {
            let title = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();

            let tab = EditorTab {
                id: self.next_id,
                title,
                content,
                file_path: Some(path),
                modified: false,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                cursor_pos: 0,
                search_highlights: Vec::new(),
            };

            self.tabs.push(tab);
            self.active_tab = self.tabs.len() - 1;
            self.next_id += 1;
        }
    }

    pub fn close_file(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn save_current(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(path) = &tab.file_path {
                if fs::write(path, &tab.content).is_ok() {
                    tab.modified = false;
                }
            } else {
                // Save as dialog
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    if fs::write(&path, &tab.content).is_ok() {
                        tab.file_path = Some(path.clone());
                        tab.title = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Untitled")
                            .to_string();
                        tab.modified = false;
                    }
                }
            }
        }
    }

    pub fn undo(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(previous) = tab.undo_stack.pop() {
                tab.redo_stack.push(tab.content.clone());
                tab.content = previous;
                tab.modified = true;
            }
        }
    }

    pub fn redo(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if let Some(next) = tab.redo_stack.pop() {
                tab.undo_stack.push(tab.content.clone());
                tab.content = next;
                tab.modified = true;
            }
        }
    }

    pub fn highlight_search(&mut self, query: &str) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.search_highlights.clear();
            if !query.is_empty() {
                let content_lower = tab.content.to_lowercase();
                let query_lower = query.to_lowercase();
                let mut start = 0;
                
                while let Some(pos) = content_lower[start..].find(&query_lower) {
                    let actual_pos = start + pos;
                    tab.search_highlights.push((actual_pos, actual_pos + query.len()));
                    start = actual_pos + 1;
                }
            }
        }
    }

    pub fn clear_search_highlights(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.search_highlights.clear();
        }
    }

    pub fn show(&mut self, ui: &mut Ui, _syntax_highlighter: &mut crate::syntax::SyntaxHighlighter) {
        // Tab bar
        if self.tabs.len() > 1 {
            ui.horizontal(|ui| {
                for (i, tab) in self.tabs.iter().enumerate() {
                    let text = if tab.modified {
                        format!("● {}", tab.title)
                    } else {
                        tab.title.clone()
                    };
                    
                    if ui.selectable_label(i == self.active_tab, text).clicked() {
                        self.active_tab = i;
                    }
                    
                    if ui.small_button("×").clicked() {
                        self.close_tab(i);
                        break;
                    }
                }
            });
            ui.separator();
        }

        // Editor content
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let old_content = tab.content.clone();
                    
                    let response = ui.add(
                        TextEdit::multiline(&mut tab.content)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(50)
                    );

                    if response.changed() && old_content != tab.content {
                        tab.undo_stack.push(old_content);
                        tab.redo_stack.clear();
                        tab.modified = true;
                        
                        // Limit undo stack size
                        if tab.undo_stack.len() > 100 {
                            tab.undo_stack.remove(0);
                        }
                    }
                });
        }
    }

    fn close_tab(&mut self, index: usize) {
        if self.tabs.len() > 1 {
            self.tabs.remove(index);
            if self.active_tab >= index && self.active_tab > 0 {
                self.active_tab -= 1;
            }
        }
    }    pub fn open_folder(&mut self, folder_path: PathBuf) {
        self.workspace_folder = Some(folder_path);
    }

    pub fn get_workspace_folder(&self) -> Option<&PathBuf> {
        self.workspace_folder.as_ref()
    }
}