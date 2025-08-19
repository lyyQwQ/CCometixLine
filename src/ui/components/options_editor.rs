use crate::config::{SegmentConfig, SegmentId};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub struct OptionsEditorComponent {
    pub is_open: bool,
    selected_option: usize,
    current_segment_id: Option<SegmentId>,
    current_options: Vec<(String, serde_json::Value)>,
}

impl Default for OptionsEditorComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionsEditorComponent {
    pub fn new() -> Self {
        Self {
            is_open: false,
            selected_option: 0,
            current_segment_id: None,
            current_options: Vec::new(),
        }
    }

    pub fn open(&mut self, segment: &SegmentConfig) {
        self.is_open = true;
        self.selected_option = 0;
        self.current_segment_id = Some(segment.id);

        // Convert HashMap to sorted Vec for consistent ordering
        self.current_options = segment
            .options
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        self.current_options.sort_by_key(|(k, _)| k.clone());
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.current_options.clear();
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.current_options.is_empty() {
            return;
        }

        let new_selection = (self.selected_option as i32 + delta)
            .max(0)
            .min((self.current_options.len() - 1) as i32) as usize;
        self.selected_option = new_selection;
    }

    pub fn toggle_current(&mut self) -> Option<(String, serde_json::Value)> {
        if let Some((key, value)) = self.current_options.get_mut(self.selected_option) {
            // Toggle boolean values
            if let Some(bool_val) = value.as_bool() {
                *value = serde_json::json!(!bool_val);
                return Some((key.clone(), value.clone()));
            }
        }
        None
    }

    pub fn get_updated_options(&self) -> HashMap<String, serde_json::Value> {
        self.current_options.iter().cloned().collect()
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.is_open {
            return;
        }

        // Calculate popup area (50% width, 60% height)
        let popup_area = centered_rect(50, 60, area);

        // Clear the popup area
        f.render_widget(Clear, popup_area);

        // Get segment name for title
        let segment_name = self
            .current_segment_id
            .map(|id| match id {
                SegmentId::Model => "Model",
                SegmentId::Directory => "Directory",
                SegmentId::Git => "Git",
                SegmentId::Usage => "Usage",
                SegmentId::Update => "Update",
                SegmentId::Cost => "Cost",
                SegmentId::BurnRate => "BurnRate",
            })
            .unwrap_or("Unknown");

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("{} Options", segment_name))
            .border_style(Style::default().fg(Color::Cyan));

        let inner = popup_block.inner(popup_area);
        f.render_widget(popup_block, popup_area);

        // Split into content and help areas
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Options list
                Constraint::Length(3), // Help text
            ])
            .split(inner);

        // Render options list
        if self.current_options.is_empty() {
            let no_options = Paragraph::new("No configurable options for this segment")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(no_options, chunks[0]);
        } else {
            let items: Vec<ListItem> = self
                .current_options
                .iter()
                .enumerate()
                .map(|(i, (key, value))| {
                    let is_selected = i == self.selected_option;

                    // Format the option display
                    let formatted_key = key.replace('_', " ");
                    let value_str = value.to_string();
                    let value_display = if let Some(bool_val) = value.as_bool() {
                        if bool_val {
                            "[✓]"
                        } else {
                            "[ ]"
                        }
                    } else {
                        value_str.as_str()
                    };

                    let line = if is_selected {
                        format!("▶ {} {}", formatted_key, value_display)
                    } else {
                        format!("  {} {}", formatted_key, value_display)
                    };

                    if is_selected {
                        ListItem::new(line).style(Style::default().fg(Color::Cyan))
                    } else {
                        ListItem::new(line)
                    }
                })
                .collect();

            let list = List::new(items);
            f.render_widget(list, chunks[0]);
        }

        // Render help text
        let help_text = "↑/↓: Navigate  Space/Enter: Toggle  Esc: Close";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(help, chunks[1]);
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
