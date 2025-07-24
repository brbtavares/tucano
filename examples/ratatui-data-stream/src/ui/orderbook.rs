use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Frame,
};

use crate::types::OrderBookData;

pub struct OrderBookWidget<'a> {
    orderbook: Option<&'a OrderBookData>,
}

impl<'a> OrderBookWidget<'a> {
    pub fn new(orderbook: Option<&'a OrderBookData>) -> Self {
        Self { orderbook }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if let Some(orderbook) = self.orderbook {
            self.render_orderbook(f, area, orderbook);
        } else {
            self.render_loading(f, area);
        }
    }

    fn render_orderbook(&self, f: &mut Frame, area: Rect, orderbook: &OrderBookData) {
        // Split area into sections
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Statistics
                Constraint::Min(10),   // Order book
            ])
            .split(area);

        // Render statistics
        self.render_stats(f, layout[0], orderbook);

        // Render order book table
        self.render_book_table(f, layout[1], orderbook);
    }

    fn render_stats(&self, f: &mut Frame, area: Rect, orderbook: &OrderBookData) {
        let best_bid = orderbook.get_best_bid();
        let best_ask = orderbook.get_best_ask();
        let spread = orderbook.get_spread();
        let mid_price = orderbook.get_mid_price();

        let stats_text = match (best_bid, best_ask, spread, mid_price) {
            (Some((bid_price, _)), Some((ask_price, _)), Some(spread), Some(mid)) => {
                format!(
                    "Best Bid: ${:.2} | Best Ask: ${:.2} | Spread: ${:.2} | Mid: ${:.2}",
                    bid_price.0, ask_price.0, spread, mid
                )
            }
            _ => "Waiting for order book data...".to_string(),
        };

        let stats = Paragraph::new(stats_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Market Data"));

        f.render_widget(stats, area);
    }

    fn render_book_table(&self, f: &mut Frame, area: Rect, orderbook: &OrderBookData) {
        // Split into bids and asks
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Render asks (left side, reversed order)
        self.render_asks_table(f, layout[0], orderbook);

        // Render bids (right side)
        self.render_bids_table(f, layout[1], orderbook);
    }

    fn render_asks_table(&self, f: &mut Frame, area: Rect, orderbook: &OrderBookData) {
        let mut rows = Vec::new();
        
        // Get asks in reverse order (highest to lowest) to display from top
        let asks: Vec<_> = orderbook.asks.iter().rev().take(15).collect();
        
        for (&price, &qty) in asks {
            let row = Row::new(vec![
                Cell::from(format!("{:.2}", price.0)).style(Style::default().fg(Color::Red)),
                Cell::from(format!("{:.4}", qty)).style(Style::default().fg(Color::White)),
                Cell::from(format!("{:.2}", price.0 * qty)).style(Style::default().fg(Color::Gray)),
            ]);
            rows.push(row);
        }

        let asks_table = Table::new(
            rows,
            [
                Constraint::Percentage(40), // Price
                Constraint::Percentage(30), // Quantity
                Constraint::Percentage(30), // Total
            ]
        )
        .header(
            Row::new(vec!["Price", "Quantity", "Total"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Asks (SELL)")
                .title_style(Style::default().fg(Color::Red))
        );

        f.render_widget(asks_table, area);
    }

    fn render_bids_table(&self, f: &mut Frame, area: Rect, orderbook: &OrderBookData) {
        let mut rows = Vec::new();
        
        // Get bids in descending order (highest to lowest)
        for (&price, &qty) in orderbook.bids.iter().rev().take(15) {
            let row = Row::new(vec![
                Cell::from(format!("{:.2}", price.0)).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.4}", qty)).style(Style::default().fg(Color::White)),
                Cell::from(format!("{:.2}", price.0 * qty)).style(Style::default().fg(Color::Gray)),
            ]);
            rows.push(row);
        }

        let bids_table = Table::new(
            rows,
            [
                Constraint::Percentage(40), // Price
                Constraint::Percentage(30), // Quantity  
                Constraint::Percentage(30), // Total
            ]
        )
        .header(
            Row::new(vec!["Price", "Quantity", "Total"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Bids (BUY)")
                .title_style(Style::default().fg(Color::Green))
        );

        f.render_widget(bids_table, area);
    }

    fn render_loading(&self, f: &mut Frame, area: Rect) {
        let loading = Paragraph::new("Loading order book data...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Order Book"));

        f.render_widget(loading, area);
    }
}
