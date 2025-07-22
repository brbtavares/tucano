use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::data::{OrderBookData, TradeData, TradesHistory};
use super::{orderbook::OrderBookWidget, trades::TradesWidget};

pub struct App {
    pub should_quit: bool,
    pub orderbook_data: Option<OrderBookData>,
    pub trades_history: TradesHistory,
    orderbook_rx: mpsc::UnboundedReceiver<OrderBookData>,
    trades_rx: mpsc::UnboundedReceiver<TradeData>,
}

impl App {
    pub fn new(
        orderbook_rx: mpsc::UnboundedReceiver<OrderBookData>,
        trades_rx: mpsc::UnboundedReceiver<TradeData>,
    ) -> Self {
        Self {
            should_quit: false,
            orderbook_data: None,
            trades_history: TradesHistory::new(1000), // Keep last 1000 trades
            orderbook_rx,
            trades_rx,
        }
    }

    pub async fn update(&mut self) {
        // Process orderbook updates
        while let Ok(orderbook) = self.orderbook_rx.try_recv() {
            self.orderbook_data = Some(orderbook);
        }

        // Process trade updates
        while let Ok(trade) = self.trades_rx.try_recv() {
            self.trades_history.add_trade(trade);
        }
    }

    pub fn reset(&mut self) {
        self.orderbook_data = None;
        self.trades_history = TradesHistory::new(1000);
    }

    pub fn render(&mut self, f: &mut Frame) {
        let size = f.size();

        // Create main layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Main content
                Constraint::Length(3),  // Footer
            ])
            .split(size);

        // Render header
        self.render_header(f, main_layout[0]);

        // Create content layout
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Order book
                Constraint::Percentage(40), // Trades
            ])
            .split(main_layout[1]);

        // Render orderbook
        self.render_orderbook(f, content_layout[0]);

        // Render trades
        self.render_trades(f, content_layout[1]);

        // Render footer
        self.render_footer(f, main_layout[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = if let Some(ref orderbook) = self.orderbook_data {
            format!(
                "Binance Data Stream - {} | Last Update: {}",
                orderbook.symbol,
                orderbook.timestamp.format("%H:%M:%S%.3f")
            )
        } else {
            "Binance Data Stream - Connecting...".to_string()
        };

        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(header, area);
    }

    fn render_orderbook(&self, f: &mut Frame, area: Rect) {
        let widget = OrderBookWidget::new(self.orderbook_data.as_ref());
        widget.render(f, area);
    }

    fn render_trades(&self, f: &mut Frame, area: Rect) {
        let widget = TradesWidget::new(&self.trades_history);
        widget.render(f, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let controls = vec![
            Span::styled("Quit: ", Style::default().fg(Color::Gray)),
            Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" | Reset: ", Style::default().fg(Color::Gray)),
            Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ];

        let footer = Paragraph::new(Line::from(controls))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        f.render_widget(footer, area);
    }
}
