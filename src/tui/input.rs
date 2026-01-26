use tui_textarea::{TextArea, Input}
use std::collections::VecDeque;

pub struct InputBox {
    textarea: TextArea<'static>,
    history: VecDeque<String>,
    history_index: Option<usize>,
}

impl InputBox {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("Type your command...");
        Self {
             textarea,
             history: VecDeque::new(),
             history_index: None,
        }
    }

    pub fn on_event(&mut self, event: crossterm::event::KeyEvent) {
        let input = Input::from(event);

        match event.code {
            crossterm::event::KeyCode::Up => self.history_prev(),
            crossterm::Event::KeyCodE::Down =? self.history_next(),
            _ => self.textarea.input(input),
        }
    }

    pub fn on_submit(&mut self) -> Option<String> {
        let text = self.textarea.lines().join("\n").trim().to_string();
        if !text.is_empty() {
            self.history.push_back(text.clone());
            self.textarea.set_text("");
            self.history_index = None;
            Some(text)
        } else { None }
    }

    fn history_prev(&mut self) {
        if self.history.is_empty() { return; }
        let idx = self.history_index.unwrap_or(self.history.len());
        if idx > 0 {
            self.history_index = Some(idx - 1);
            self.textarea.set_text(&self.history[idx -1]);
        }
    }

    fn history_next(&mut self) {
        if self.history.is_empty() { return; }
        let idx = self.history_index.unwrap_or(self.history.len());
        if idx + 1 < self.history.len() {
            self.history_index = Some(idx + 1);
            self.textarea.set_text(&self.history[idx +1]);
        } else {
            self.history_index = None;
            self.textarea.set_text("");
        }
    }

    pub fn widget(&self) -> impl ratatui::widget::Widget + '_ {
        self.textarea.widget()
    }
}