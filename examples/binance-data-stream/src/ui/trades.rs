use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Frame,
};

use crate::types::TradesHistory;

pub struct TradesWidget<'a> {
    trades_history: &'a TradesHistory,
}

impl<'a> TradesWidget<'a> {
    pub fn new(trades_history: &'a TradesHistory) -> Self {
        Self { trades_history }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if self.trades_history.trades.is_empty() {
            self.render_loading(f, area);
        } else {
            self.render_trades(f, area);
        }
    }

    fn render_trades(&self, f: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Statistics
                Constraint::Min(10),   // Trades table
            ])
            .split(area);

        // Render statistics
        self.render_trade_stats(f, layout[0]);

        // Render trades table
        self.render_trades_table(f, layout[1]);
    }

    fn render_trade_stats(&self, f: &mut Frame, area: Rect) {
        let volume_1m = self.trades_history.get_volume_in_period(60);
        let avg_price_1m = self.trades_history.get_avg_price_in_period(60);
        let total_trades = self.trades_history.trades.len();

        let stats_text = match avg_price_1m {
            Some(avg_price) => {
                format!(
                    "Trades: {} | 1m Volume: {:.4} BTC | 1m Avg: ${:.2}",
                    total_trades, volume_1m, avg_price
                )
            }
            None => format!("Trades: {} | 1m Volume: {:.4} BTC", total_trades, volume_1m),
        };

        let stats = Paragraph::new(stats_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Trade Statistics"));

        f.render_widget(stats, area);
    }

    fn render_trades_table(&self, f: &mut Frame, area: Rect) {
        let mut rows = Vec::new();

        // Get recent trades (latest first)
        let recent_trades = self.trades_history.get_recent_trades(20);
        
        for trade in recent_trades.iter().rev() {
            let side_color = if trade.is_buyer_maker {
                Color::Red  // Sell
            } else {
                Color::Green // Buy
            };

            let row = Row::new(vec![
                Cell::from(trade.timestamp.format("%H:%M:%S").to_string())
                    .style(Style::default().fg(Color::Gray)),
                Cell::from(trade.side_str())
                    .style(Style::default().fg(side_color).add_modifier(Modifier::BOLD)),
                Cell::from(format!("{:.2}", trade.price))
                    .style(Style::default().fg(Color::White)),
                Cell::from(format!("{:.4}", trade.quantity))
                    .style(Style::default().fg(Color::White)),
                Cell::from(format!("{:.2}", trade.notional_value()))
                    .style(Style::default().fg(Color::Gray)),
            ]);
            rows.push(row);
        }

        let trades_table = Table::new(
            rows,
            [
                Constraint::Percentage(20), // Time
                Constraint::Percentage(15), // Side
                Constraint::Percentage(25), // Price
                Constraint::Percentage(20), // Quantity
                Constraint::Percentage(20), // Total
            ]
        )
        .header(
            Row::new(vec!["Time", "Side", "Price", "Quantity", "Total"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Trades")
                .title_style(Style::default().fg(Color::Cyan))
        );

        f.render_widget(trades_table, area);
    }

    fn render_loading(&self, f: &mut Frame, area: Rect) {
        let loading = Paragraph::new("Waiting for trade data...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Trades"));

        f.render_widget(loading, area);
    }
}
