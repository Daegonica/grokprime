use ratatui::{
    // backend::Backend,
    crossterm::event::{KeyCode, KeyEvent},
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

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.input.push(c);
                true
            }
            KeyCode::Backspace => {
                self.input.pop();
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
                            InputAction::SendAsMessage(_content) => todo!()

                        }
                    }
                    // self.add_message(format!("> {}", line));
                    // self.input.clear();
                }
                true
            }
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

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
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

        let input_text = Line::from(vec![
            Span::styled(" > ", Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
            Span::raw(&self.input),
        ]);

        let input_widget = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 140, 0)))
                    .title(" Input "),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(input_widget, input_area);


        if input_area.height > 0 && input_area.width > 4 {
            let cursor_x = 3 + self.input.len() as u16;
            let cursor_pos = Position {
                x: input_area.x + cursor_x.min(input_area.width.saturating_sub(1)),
                y: input_area.y + 1,
            };
            frame.set_cursor_position(cursor_pos);
        }
    }
}