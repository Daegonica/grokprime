//! # Daegonica Module: tui::widgets
//!
//! **Purpose:** Reusable rendering widgets and helper functions for the TUI
//!
//! **Context:**
//! - Pure functions that take data and produce UI elements
//! - No state, no side effects - just rendering logic
//! - Used by app.rs to draw the interface
//!
//! **Responsibilities:**
//! - Render scrollable message sections
//! - Format text with proper styling
//! - Calculate widget dimensions
//! - Handle text wrapping
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2026-01-20
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
/// # render_message_section
///
/// **Purpose:**
/// Standalone function to render a scrollable message section with borders and scrollbar.
///
/// **Parameters:**
/// - `frame`: The ratatui frame to render into
/// - `area`: The rectangular area to render the message section
/// - `lines`: Vector of formatted lines to display
/// - `title`: Title to display in the border
/// - `scroll`: Mutable reference to scroll position (updated if out of bounds)
///
/// **Returns:**
/// `bool` - true if scroll is at the actual bottom after clamping, false otherwise
///
/// **Details:**
/// - Automatically bounds scroll position to valid range
/// - Renders scrollbar with up/down arrows and position indicator
/// - Applies text wrapping and orange border styling
pub fn render_message_section(
    frame: &mut Frame,
    area: Rect,
    lines: Vec<Line>,
    title: &String,
    scroll: &mut u16,
) -> bool {

    let visible_height = area.height.saturating_sub(2);
    let content_width = area.width.saturating_sub(2) as usize; // Account for borders
    
    // Calculate actual wrapped line count
    let mut wrapped_line_count = 0u16;
    for line in &lines {
        let line_width = line.width();
        if line_width == 0 {
            wrapped_line_count += 1; // Empty lines still take 1 line
        } else {
            // Calculate how many visual lines this Line will wrap into
            let visual_lines = (line_width + content_width - 1) / content_width.max(1);
            wrapped_line_count += visual_lines as u16;
        }
    }
    
    let content_height = wrapped_line_count;
    let content_len = content_height as usize;
    let viewport_len = visible_height as usize;

    // Set scroll within bounds
    let max_scroll = content_height.saturating_sub(visible_height);
    if *scroll == u16::MAX || *scroll > max_scroll {
        *scroll = max_scroll;
    }
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");


    let mut scrollbar_state = ScrollbarState::new(content_len)
        .viewport_content_length(viewport_len)
        .position(*scroll as usize);

    // Add all messages to 1 'text' for display
    let text = Text::from(lines.clone());
    // Set border and title styles
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title(title.as_str())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(255, 140, 0)))
                .title_style(Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD)),
        )
        .wrap(Wrap { trim: true })
        .scroll((*scroll, 0));

    // Render message area
    frame.render_widget(paragraph, area);
    // Add scrollbar to message area
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    
    // Return whether we're at the actual bottom
    *scroll >= max_scroll
}

