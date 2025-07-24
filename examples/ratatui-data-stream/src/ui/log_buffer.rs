use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    text::Line,
    Frame,
};

use crate::types::LogBuffer;

pub struct LogBufferWidget<'a> {
    log_buffer: &'a LogBuffer,
}

impl<'a> LogBufferWidget<'a> {
    pub fn new(log_buffer: &'a LogBuffer) -> Self {
        Self { log_buffer }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let logs = self.log_buffer.get_all();
        let lines: Vec<Line> = logs.iter().map(|msg| Line::from(msg.clone())).collect();
        let log_widget = Paragraph::new(lines)
            .block(Block::default().title("Logs").borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(log_widget, area);
    }

    #[allow(dead_code)]
    pub fn push_log(&self, msg: String) {
        self.log_buffer.push(msg);
    }
}

    
