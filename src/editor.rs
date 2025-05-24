use egui::{ScrollArea, TextEdit, Ui, Color32};
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

    pub fn find_next(&mut self, query: &str) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if query.is_empty() || tab.search_highlights.is_empty() {
                return;
            }

            // Find the next match after the current cursor position
            let current_pos = tab.cursor_pos;
            let mut next_match = None;

            // Look for the first match after current position
            for &(start, end) in &tab.search_highlights {
                if start > current_pos {
                    next_match = Some((start, end));
                    break;
                }
            }

            // If no match found after cursor, wrap to the first match
            if next_match.is_none() && !tab.search_highlights.is_empty() {
                next_match = Some(tab.search_highlights[0]);
            }

            // Update cursor position to the found match
            if let Some((start, _end)) = next_match {
                tab.cursor_pos = start;
            }
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

        // Handle search actions first
        let mut clear_search = false;
        let mut find_next = false;
        let mut find_previous = false;

        // Editor content
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            let old_content = tab.content.clone();
            let has_highlights = !tab.search_highlights.is_empty();
            
            // Show search info if we have highlights
            if has_highlights {
                ui.horizontal(|ui| {
                    ui.label(format!("🔍 {} matches found", tab.search_highlights.len()));
                    if ui.small_button("Clear Search").clicked() {
                        clear_search = true;
                    }
                    if ui.small_button("Find Next").clicked() {
                        find_next = true;
                    }
                    if ui.small_button("Find Previous").clicked() {
                        find_previous = true;
                    }
                });
                ui.separator();
            }
            
            // Create highlighted job outside of closures to avoid borrow issues
            let highlighted_job = if has_highlights {
                Some(Self::create_highlighted_job_static(tab))
            } else {
                None
            };
            
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(job) = highlighted_job {
                        // Show highlighted content - use a more direct approach
                        ui.add(egui::Label::new(job).wrap(false)); // Remove the .selectable(false) call
                        
                        // Hide the actual editor widget but still process it for changes
                        let mut frame = egui::Frame::none();
                        frame.fill = Color32::TRANSPARENT;
                        frame.show(ui, |ui| {
                            ui.set_min_size(egui::vec2(0.0, 0.0)); // Make it take no space
                            
                            let text_edit = TextEdit::multiline(&mut tab.content)
                                .desired_width(0.0)
                                .desired_rows(0)
                                .frame(false)
                                .interactive(true);

                            let response = ui.add_sized(egui::vec2(0.0, 0.0), text_edit);
                            
                            if response.changed() && old_content != tab.content {
                                tab.undo_stack.push(old_content);
                                tab.redo_stack.clear();
                                tab.modified = true;
                                
                                // Rehighlight on content change
                                if !tab.search_highlights.is_empty() {
                                    // Try to preserve search matches after edit
                                    // Would need query string to properly reapply
                                }
                            }
                        });
                    } else {
                        // Normal editor when no search
                        let text_edit = TextEdit::multiline(&mut tab.content)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(50);

                        let response = ui.add(text_edit);

                        if response.changed() && old_content != tab.content {
                            tab.undo_stack.push(old_content);
                            tab.redo_stack.clear();
                            tab.modified = true;
                        }
                    }
                });
        }

        // Handle search actions after borrowing is done
        if clear_search {
            self.clear_search_highlights();
        }
        if find_next {
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                if !tab.search_highlights.is_empty() {
                    let current_pos = tab.cursor_pos;
                    let mut next_match = None;
                    
                    for &(start, _) in &tab.search_highlights {
                        if start > current_pos {
                            next_match = Some(start);
                            break;
                        }
                    }
                    
                    if next_match.is_none() {
                        next_match = Some(tab.search_highlights[0].0);
                    }
                    
                    if let Some(pos) = next_match {
                        tab.cursor_pos = pos;
                    }
                }
            }
        }
        if find_previous {
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                if !tab.search_highlights.is_empty() {
                    let current_pos = tab.cursor_pos;
                    let mut prev_match = None;
                    
                    for &(start, _) in tab.search_highlights.iter().rev() {
                        if start < current_pos {
                            prev_match = Some(start);
                            break;
                        }
                    }
                    
                    if prev_match.is_none() {
                        prev_match = Some(tab.search_highlights.last().unwrap().0);
                    }
                    
                    if let Some(pos) = prev_match {
                        tab.cursor_pos = pos;
                    }
                }
            }
        }
    }

    fn create_highlighted_job(&self, tab: &EditorTab) -> egui::text::LayoutJob {
        Self::create_highlighted_job_static(tab)
    }

    fn create_highlighted_job_static(tab: &EditorTab) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        let mut last_end = 0;
        
        // Sort highlights by position
        let mut highlights = tab.search_highlights.clone();
        highlights.sort_by_key(|&(start, _)| start);
        
        // Find current highlight index
        let current_highlight_idx = highlights.iter().position(|(start, _)| *start >= tab.cursor_pos);
        
        for (i, (start, end)) in highlights.iter().enumerate() {
            let start = *start;
            let end = *end;
            
            // Add normal text before highlight
            if last_end < start && start < tab.content.len() {
                job.append(
                    &tab.content[last_end..start],
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color: Color32::GRAY, // Light gray for non-highlighted text
                        ..Default::default()
                    },
                );
            }
            
            // Add highlighted text
            if start < tab.content.len() && end <= tab.content.len() {
                let is_current = current_highlight_idx == Some(i);
                let (bg_color, text_color) = if is_current { 
                    // Current match: blue background with white text
                    (Color32::from_rgb(30, 144, 255), Color32::WHITE)
                } else { 
                    // Other matches: yellow background with black text
                    (Color32::from_rgb(255, 255, 0), Color32::BLACK)
                };
                
                job.append(
                    &tab.content[start..end],
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color: text_color,
                        background: bg_color,
                        ..Default::default()
                    },
                );
            }
            
            last_end = end;
        }
        
        // Add remaining text
        if last_end < tab.content.len() {
            job.append(
                &tab.content[last_end..],
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::monospace(14.0),
                    color: Color32::GRAY, // Light gray for non-highlighted text
                    ..Default::default()
                },
            );
        }
        
        job
    }

    fn move_to_next_match(&mut self, tab: &mut EditorTab) {
        if tab.search_highlights.is_empty() {
            return;
        }
        
        let current_pos = tab.cursor_pos;
        let mut next_match = None;
        
        for &(start, _) in &tab.search_highlights {
            if start > current_pos {
                next_match = Some(start);
                break;
            }
        }
        
        if next_match.is_none() {
            next_match = Some(tab.search_highlights[0].0);
        }
        
        if let Some(pos) = next_match {
            tab.cursor_pos = pos;
        }
    }

    fn move_to_previous_match(&mut self, tab: &mut EditorTab) {
        if tab.search_highlights.is_empty() {
            return;
        }
        
        let current_pos = tab.cursor_pos;
        let mut prev_match = None;
        
        for &(start, _) in tab.search_highlights.iter().rev() {
            if start < current_pos {
                prev_match = Some(start);
                break;
            }
        }
        
        if prev_match.is_none() {
            prev_match = Some(tab.search_highlights.last().unwrap().0);
        }
        
        if let Some(pos) = prev_match {
            tab.cursor_pos = pos;
        }
    }

    fn show_highlighted_content(&self, _ui: &mut egui::Ui, _tab: &EditorTab) {
        // This method is no longer used but kept for compatibility
    }

    fn close_tab(&mut self, index: usize) {
        if self.tabs.len() > 1 {
            self.tabs.remove(index);
            if self.active_tab >= index && self.active_tab > 0 {
                self.active_tab -= 1;
            }
        }
    }

    pub fn get_search_match_count(&self) -> usize {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            tab.search_highlights.len()
        } else {
            0
        }
    }

    pub fn open_folder(&mut self, folder_path: PathBuf) {
        self.workspace_folder = Some(folder_path);
    }

    pub fn get_workspace_folder(&self) -> Option<&PathBuf> {
        self.workspace_folder.as_ref()
    }
}