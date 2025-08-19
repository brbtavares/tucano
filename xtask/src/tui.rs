// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use crate::app::*;

fn render_main_menu(f: &mut Frame, area: Rect, app: &mut App) {
    let menu_items = vec![
        ListItem::new(vec![Line::from(vec![
            Span::styled("🔧 ", Style::default().fg(Color::Blue)),
            Span::raw("Code Quality"),
            Span::styled(" (fmt + clippy)", Style::default().fg(Color::Gray)),
        ])]),
        ListItem::new(vec![Line::from(vec![
            Span::styled("📊 ", Style::default().fg(Color::Green)),
            Span::raw("Size Comparison"),
            Span::styled(" (local vs published)", Style::default().fg(Color::Gray)),
        ])]),
        ListItem::new(vec![Line::from(vec![
            Span::styled("📝 ", Style::default().fg(Color::Cyan)),
            Span::raw("Disclaimer Check"),
            Span::styled(" (copyright headers)", Style::default().fg(Color::Gray)),
        ])]),
        ListItem::new(vec![Line::from(vec![
            Span::styled("🚀 ", Style::default().fg(Color::Magenta)),
            Span::raw("Release Management"),
            Span::styled(" (publish to crates.io)", Style::default().fg(Color::Gray)),
        ])]),
        ListItem::new(vec![Line::from(vec![
            Span::styled("❌ ", Style::default().fg(Color::Red)),
            Span::raw("Exit"),
        ])]),
    ];
    let menu = List::new(menu_items)
        .block(
            Block::default()
                .title("📋 Main Menu - Select a task")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");
    f.render_stateful_widget(menu, area, &mut app.menu_state);
}

fn render_code_quality(f: &mut Frame, area: Rect, app: &App) {
    let content = if app.loading {
        vec![
            Line::from("🔄 Running code quality checks..."),
            Line::from(""),
            Line::from("• cargo fmt --all"),
            Line::from("• cargo clippy --all-targets --all-features"),
            Line::from(""),
            Line::from("Please wait..."),
        ]
    } else {
        vec![
            Line::from("🔧 Code Quality Tools"),
            Line::from(""),
            Line::from("This will run:"),
            Line::from("• cargo fmt --all - Format all Rust code"),
            Line::from("• cargo clippy --all-targets --all-features - Run linter"),
            Line::from(""),
            Line::from("Press Enter to run checks"),
            Line::from("Press Esc to go back"),
        ]
    };
    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title("🔧 Code Quality")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_size_comparison(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    let items: Vec<ListItem> = app
        .workspace
        .crates
        .iter()
        .map(|c| {
            let local_size = c.local_size_mb();
            let status_icon = if c.local_size.is_some() { "✅" } else { "⏳" };
            ListItem::new(vec![Line::from(vec![
                Span::styled(
                    format!("{} ", status_icon),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    &c.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" ({})", local_size),
                    Style::default().fg(Color::Gray),
                ),
            ])])
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .title("📦 Crates")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");
    f.render_stateful_widget(list, chunks[0], &mut app.crate_state);
    if let Some(selected_crate) = app
        .workspace
        .crates
        .get(app.crate_state.selected().unwrap_or(0))
    {
        let details = vec![
            Line::from(vec![
                Span::styled("Crate: ", Style::default().fg(Color::Yellow)),
                Span::raw(&selected_crate.name),
            ]),
            Line::from(vec![
                Span::styled("Version: ", Style::default().fg(Color::Yellow)),
                Span::raw(&selected_crate.version),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("📁 Local Size: ", Style::default().fg(Color::Cyan)),
                Span::raw(selected_crate.local_size_mb()),
            ]),
            Line::from(vec![
                Span::styled("🌐 Published Size: ", Style::default().fg(Color::Magenta)),
                Span::raw(selected_crate.published_size_mb()),
            ]),
            Line::from(""),
        ];
        let mut content = details;
        if let Some(diff) = selected_crate.size_diff() {
            let (color, symbol) = if diff > 0 {
                (Color::Red, "+")
            } else if diff < 0 {
                (Color::Green, "")
            } else {
                (Color::Gray, "")
            };
            content.push(Line::from(vec![
                Span::styled("📊 Difference: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}{:.2} MiB", symbol, diff as f64 / (1024.0 * 1024.0)),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        content.push(Line::from(""));
        content.push(Line::from("Press 'r' to refresh sizes"));
        content.push(Line::from("Press Enter for details"));
        let details_panel = Paragraph::new(content)
            .block(
                Block::default()
                    .title("📊 Size Details")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(details_panel, chunks[1]);
    } else {
        let empty_panel = Paragraph::new("Select a crate to see details").block(
            Block::default()
                .title("📊 Details")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        );
        f.render_widget(empty_panel, chunks[1]);
    }
}

fn render_disclaimer_check(f: &mut Frame, area: Rect, app: &App) {
    let content = if app.loading {
        vec![
            Line::from("🔄 Checking disclaimers..."),
            Line::from(""),
            Line::from("Scanning all .rs files for copyright headers"),
            Line::from("This may take a moment..."),
        ]
    } else {
        vec![
            Line::from("📝 Disclaimer Management"),
            Line::from(""),
            Line::from("Features:"),
            Line::from("• Check all .rs files for disclaimer headers"),
            Line::from("• Identify files missing copyright notices"),
            Line::from("• Add disclaimer template to missing files"),
            Line::from(""),
            Line::from("Default template:"),
            Line::from("// Copyright (c) 2024 Your Name"),
            Line::from("// Licensed under the MIT License"),
            Line::from(""),
            Line::from("Press Enter to check all files"),
            Line::from("Press Esc to go back"),
        ]
    };
    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title("📝 Disclaimer Check")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_release(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let items: Vec<ListItem> = app
        .workspace
        .crates
        .iter()
        .map(|c| {
            ListItem::new(vec![Line::from(vec![
                Span::styled("📦 ", Style::default().fg(Color::Magenta)),
                Span::styled(
                    &c.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" v{}", c.version), Style::default().fg(Color::Gray)),
            ])])
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .title("🚀 Select Crate to Release")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");
    f.render_stateful_widget(list, chunks[0], &mut app.crate_state);
    let content = if app.loading {
        vec![
            Line::from("🔄 Processing release..."),
            Line::from(""),
            Line::from("Running pre-release checks:"),
            Line::from("• Running tests"),
            Line::from("• Checking if version exists"),
            Line::from("• Publishing to crates.io"),
        ]
    } else {
        vec![
            Line::from("🚀 Release Management"),
            Line::from(""),
            Line::from("Pre-release checks:"),
            Line::from("✅ Run cargo test"),
            Line::from("✅ Check version not published"),
            Line::from("✅ Validate Cargo.toml"),
            Line::from(""),
            Line::from("Release process:"),
            Line::from("📤 cargo publish"),
            Line::from("🏷️  Git tag (optional)"),
            Line::from(""),
            Line::from("Press Enter to release selected crate"),
            Line::from("Press Esc to go back"),
        ]
    };
    let info_panel = Paragraph::new(content)
        .block(
            Block::default()
                .title("ℹ️  Release Info")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(info_panel, chunks[1]);
}

fn render_crate_details(f: &mut Frame, area: Rect, app: &App, crate_name: &str) {
    let selected_crate = app.workspace.crates.iter().find(|c| c.name == crate_name);
    let content = if let Some(crate_info) = selected_crate {
        vec![
            Line::from(vec![
                Span::styled("📦 Crate: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    &crate_info.name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("🏷️  Version: ", Style::default().fg(Color::Yellow)),
                Span::raw(&crate_info.version),
            ]),
            Line::from(vec![
                Span::styled("📁 Path: ", Style::default().fg(Color::Yellow)),
                Span::raw(crate_info.path.to_string_lossy()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("📊 Local Size: ", Style::default().fg(Color::Cyan)),
                Span::raw(crate_info.local_size_mb()),
            ]),
            Line::from(vec![
                Span::styled("🌐 Published Size: ", Style::default().fg(Color::Magenta)),
                Span::raw(crate_info.published_size_mb()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🔗 Dependencies (", Style::default().fg(Color::Green)),
                Span::styled(
                    crate_info.dependencies.len().to_string(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("):", Style::default().fg(Color::Green)),
            ]),
        ]
    } else {
        vec![Line::from(vec![
            Span::styled("❌ Crate not found: ", Style::default().fg(Color::Red)),
            Span::raw(crate_name),
        ])]
    };
    let mut full_content = content;
    if let Some(crate_info) = selected_crate {
        for (_i, dep) in crate_info.dependencies.iter().enumerate().take(10) {
            full_content.push(Line::from(vec![Span::raw("  • "), Span::raw(dep)]));
        }
        if crate_info.dependencies.len() > 10 {
            full_content.push(Line::from(vec![Span::styled(
                format!("  ... and {} more", crate_info.dependencies.len() - 10),
                Style::default().fg(Color::Gray),
            )]));
        }
        full_content.push(Line::from(""));
        full_content.push(Line::from("Press Esc to go back"));
    }
    let details = Paragraph::new(full_content)
        .block(
            Block::default()
                .title(format!("🔍 Crate Details: {}", crate_name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(details, area);
}

// --- INÍCIO: Funções de renderização modernas do tui_new.rs ---
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Header com título e contagem de crates
    render_header(f, chunks[0], app);

    // Main content
    match &app.current_screen {
        Screen::MainMenu => render_main_menu(f, chunks[1], app),
        Screen::CodeQuality => render_code_quality(f, chunks[1], app),
        Screen::SizeComparison => render_size_comparison(f, chunks[1], app),
        Screen::DisclaimerCheck => render_disclaimer_check(f, chunks[1], app),
        Screen::Release => render_release(f, chunks[1], app),
        Screen::CrateDetails(name) => render_crate_details(f, chunks[1], app, name),
    }

    // Status bar
    render_status_bar(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        "🦀 Rust Workspace Manager - {} crates",
        app.workspace.crates.len()
    );
    let header = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let mut status_style = Style::default().fg(Color::Green);
    let mut loading_indicator = "";
    if app.loading {
        status_style = Style::default().fg(Color::Yellow);
        loading_indicator = " ⟳";
    }
    let controls = "Controls: ↑↓/jk=navigate | Enter=select | Esc=back | r=refresh | q=quit";
    let status_text = format!("{}{} | {}", app.status_message, loading_indicator, controls);
    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, area);
}

// (Cole aqui as demais funções de renderização modernas do tui_new.rs, adaptando para usar o App e enums locais)
// ...
