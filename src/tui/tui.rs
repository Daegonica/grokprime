use ratatui::{
    // backend::Backend,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::prelude::*;

#[derive(Debug)]
pub struct ShadowApp {
    pub messages: VecDeque<String>,
    pub input: String,
    pub scroll: u16,
    pub max_history: usize,
    pub user_input: Option<UserInput>,
    pub pending_messages: Arc<Mutex<Vec<String>>>,
    pub is_waiting: bool,
    pub input_scroll: usize,
    pub input_max_lines: u16,
}

impl Default for ShadowApp {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            input: String::new(),
            scroll: 0,
            max_history: 1000,
            user_input: None,
            pending_messages: Arc::new(Mutex::new(Vec::new())),
            is_waiting: false,
            input_scroll: 0,
            input_max_lines: 20,
        }
    }
}

impl ShadowApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_message_buffer(&self) -> Arc<Mutex<Vec<String>>> {
        Arc::clone(&self.pending_messages)
    }

    pub fn flush_pending_messages(&mut self) {
        let messages_to_add: Vec<String> = if let Ok(mut pending) = self.pending_messages.lock() {
            pending.drain(..).collect()
        } else {
            Vec::new()
        };

        for msg in messages_to_add {
            self.add_message(msg);
        }
    }
    
    pub fn add_message(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.messages.push_back(msg);

        while self.messages.len() > self.max_history {
            self.messages.pop_front();
        }

        self.scroll_to_bottom();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = u16::MAX;
    }

    fn scroll_input_to_bottom(&mut self) {
        let wrapped = self.wrap_input_text(100);
        self.input_scroll = wrapped.len().saturating_sub(self.input_max_lines as usize);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {

            // Input Text control
            KeyCode::Char(c) => {
                self.input.push(c);
                self.scroll_input_to_bottom();
                true
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.scroll_input_to_bottom();
                true
            }
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    let line = self.input.trim().to_string();
                    if let Some(ref user_input) = self.user_input {
                        match user_input.process_input(&line) {
                            InputAction::DoNothing => {
                                // self.add_message(format!("> {}", line));
                                self.input.clear();
                            }
                            InputAction::ContinueNoSend(msg) => {
                                self.add_message(format!("> {}", msg));
                                self.input.clear();
                            }
                            InputAction::Quit => todo!(),
                            InputAction::SendAsMessage(_content) => todo!(),
                            InputAction::PostTweet(_content) => todo!(),
                            InputAction::DraftTweet(_content) => todo!(),

                        }
                    }
                }
                true
            }

            // Input Scroll control
            KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input_scroll = self.input_scroll.saturating_sub(1);
                true
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let wrapped = self.wrap_input_text(100);
                let max_scroll = wrapped.len().saturating_sub(self.input_max_lines as usize);
                self.input_scroll = (self.input_scroll + 1).min(max_scroll);
                true
            }

            // History Scroll control
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
                true
            }
            KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1);
                true
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
                true
            }
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_add(10);
                true
            }
            _ => false,
        }
    }

    fn calculate_input_height(&self, width: u16) -> u16 {
        if self.is_waiting{
            return 3;
        }

        let available_width = width.saturating_sub(6) as usize;
        if available_width == 0{
            return 3;
        }

        let lines_needed = if self.input.is_empty() {
            1
        } else {
            (self.input.len() / available_width) + 1
        };

        (lines_needed.min(self.input_max_lines as usize) as u16) + 2
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let input_height = self.calculate_input_height(frame.area().width);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(input_height),
            ])
            .split(frame.area());

        let message_area = chunks[0];

        let mut lines: Vec<Line> = Vec::new();
        for msg in &self.messages {
            let content = if msg.starts_with('>') {
                Line::from(Span::styled(
                    msg.clone(),
                    Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(msg.clone())
            };
            lines.push(content);
        }

        let text = Text::from(lines.clone());

        let content_height = lines.len() as u16;
        let visible_height = message_area.height.saturating_sub(2);

        let max_scroll = content_height.saturating_sub(visible_height);

        self.scroll = self.scroll.min(max_scroll);

        if self.scroll == u16::MAX || self.scroll > max_scroll {
            self.scroll = max_scroll;
        }

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Shadow ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 140, 0)))
                    .title_style(Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)),
            )
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));

        frame.render_widget(paragraph, message_area);

        let content_len = content_height as usize;
        let viewport_len = visible_height as usize;

        let mut scrollbar_state = ScrollbarState::default()
            .content_length(content_len)
            .viewport_content_length(viewport_len)
            .position(self.scroll as usize);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        frame.render_stateful_widget(scrollbar, message_area, &mut scrollbar_state);

        let max_scroll = content_len.saturating_sub(viewport_len);
        self.scroll = (self.scroll as usize).min(max_scroll) as u16;

        let input_area = chunks[1];

        let input_text = if self.is_waiting {
            Text::from(vec![
                Line::from(vec![
                    Span::styled(" > ", Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
                    Span::styled("Shadow is thinking...", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                ])
            ])
        } else {
            let available_width = input_area.width.saturating_sub(6) as usize;

            let wrapped_lines = self.wrap_input_text(available_width);
            let total_lines = wrapped_lines.len();

            let max_visible = (input_area.height.saturating_sub(2)) as usize;
            let scroll_offset = self.input_scroll.min(total_lines.saturating_sub(max_visible));

            let visible_lines: Vec<Line> = wrapped_lines
                .iter()
                .skip(scroll_offset)
                .take(max_visible)
                .enumerate()
                .map(|(idx, line)| {
                    if idx == 0 {
                        Line::from(vec![
                            Span::styled(" > ", Style::default().fg(Color::Rgb(255, 140, 0))),
                            Span::raw(line.to_string()),
                        ])
                    } else {
                        Line::from(format!("   {}", line))
                    }
                })
                .collect();

            Text::from(visible_lines)
        };

        let input_widget = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 140, 0)))
                    .title(" Input "),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(input_widget, input_area);


        if input_area.height > 2 && input_area.width > 6 && !self.is_waiting {
            let width = input_area.width.saturating_sub(6) as usize;
            let wrapped = self.wrap_input_text(width);

            let mut chars_counted = 0;
            let mut cursor_line = 0;
            let mut cursor_col_in_line = 0;

            for (line_idx, line) in wrapped.iter().enumerate() {
                let line_len = line.len();
                if chars_counted + line_len >= self.input.len() {
                    cursor_line = line_idx;
                    cursor_col_in_line = self.input.len() - chars_counted;
                    break;
                }
                chars_counted += line_len;
            }

            if cursor_line >= self.input_scroll {
                let visible_line = cursor_line - self.input_scroll;
                let max_visible = input_area.height.saturating_sub(2) as usize;

                if visible_line < max_visible {
                    let cursor_pos = Position {
                        x: input_area.x + 3 + cursor_col_in_line as u16,
                        y: input_area.y + 1 + visible_line as u16,
                    };
                    frame.set_cursor_position(cursor_pos);
                }
            }
        }
    }

    fn wrap_input_text(&self, width: usize) -> Vec<String> {
        if self.input.is_empty() {
            return vec![String::new()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in self.input.split_inclusive(|c: char| c.is_whitespace()) {
            if word.contains('\n') {
                let parts: Vec<&str> = word.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        lines.push(current_line.clone());
                        current_line.clear();
                    }
                    if !part.is_empty() {
                        let test_len = current_line.len() + part.len();
                        if test_len > width && !current_line.is_empty() {
                            lines.push(current_line.clone());
                            current_line = part.to_string();
                        } else {
                            current_line.push_str(part);
                        }
                    }
                }
                continue;
            }

            let test_len = current_line.len() + word.len();

            if test_len > width && !current_line.is_empty() {
                lines.push(current_line.trim_end().to_string());
                current_line = word.to_string();
            } else {
                current_line.push_str(word);
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        }
    }
}