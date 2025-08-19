use anyhow::Result;
use ratatui::widgets::ListState;

use crate::commands::*;
use crate::workspace::WorkspaceInfo;

pub struct App {
    pub workspace: WorkspaceInfo,
    pub current_screen: Screen,
    pub menu_state: ListState,
    pub crate_state: ListState,
    //pub selected_crate: Option<usize>,
    pub status_message: String,
    pub loading: bool,
    pub current_tab: Tab,
}

#[derive(Clone, PartialEq)]
pub enum Screen {
    MainMenu,
    CodeQuality,
    SizeComparison,
    DisclaimerCheck,
    Release,
    CrateDetails(String),
}

#[derive(Clone, PartialEq)]
pub enum Tab {
    Overview,
    Tasks,
    Crates,
    Settings,
}

impl App {
    pub fn new(workspace: WorkspaceInfo) -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));

        let mut crate_state = ListState::default();
        if !workspace.crates.is_empty() {
            crate_state.select(Some(0));
        }

        Self {
            workspace,
            current_screen: Screen::MainMenu,
            menu_state,
            crate_state,
            //selected_crate: None,
            status_message: "Ready - Use ↑↓ to navigate, Enter to select, 'q' to quit".to_string(),
            loading: false,
            current_tab: Tab::Overview,
        }
    }

    pub fn next(&mut self) {
        match self.current_screen {
            Screen::MainMenu => {
                let i = match self.menu_state.selected() {
                    Some(i) => (i + 1) % 5, // 5 menu items
                    None => 0,
                };
                self.menu_state.select(Some(i));
            }
            Screen::SizeComparison | Screen::Release => {
                if !self.workspace.crates.is_empty() {
                    let i = match self.crate_state.selected() {
                        Some(i) => (i + 1) % self.workspace.crates.len(),
                        None => 0,
                    };
                    self.crate_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    pub fn previous(&mut self) {
        match self.current_screen {
            Screen::MainMenu => {
                let i = match self.menu_state.selected() {
                    Some(i) => (i + 4) % 5, // 5 menu items
                    None => 0,
                };
                self.menu_state.select(Some(i));
            }
            Screen::SizeComparison | Screen::Release => {
                if !self.workspace.crates.is_empty() {
                    let i = match self.crate_state.selected() {
                        Some(i) => {
                            (i + self.workspace.crates.len() - 1) % self.workspace.crates.len()
                        }
                        None => 0,
                    };
                    self.crate_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Overview => Tab::Tasks,
            Tab::Tasks => Tab::Crates,
            Tab::Crates => Tab::Settings,
            Tab::Settings => Tab::Overview,
        };
    }

    pub async fn select(&mut self) -> Result<()> {
        match &self.current_screen {
            Screen::MainMenu => {
                if let Some(i) = self.menu_state.selected() {
                    self.current_screen = match i {
                        0 => Screen::CodeQuality,
                        1 => Screen::SizeComparison,
                        2 => Screen::DisclaimerCheck,
                        3 => Screen::Release,
                        4 => return Ok(()), // Exit will be handled by main loop
                        _ => Screen::MainMenu,
                    };

                    // Initialize data for new screen
                    match self.current_screen {
                        Screen::SizeComparison => {
                            self.status_message = "Loading crate sizes...".to_string();
                        }
                        Screen::DisclaimerCheck => {
                            self.status_message = "Ready to check disclaimers".to_string();
                        }
                        Screen::Release => {
                            self.status_message = "Select a crate to release".to_string();
                        }
                        _ => {}
                    }
                }
            }
            Screen::CodeQuality => {
                self.run_code_quality_task().await?;
            }
            Screen::SizeComparison => {
                if let Some(i) = self.crate_state.selected() {
                    if let Some(crate_info) = self.workspace.crates.get(i) {
                        self.current_screen = Screen::CrateDetails(crate_info.name.clone());
                    }
                }
            }
            Screen::DisclaimerCheck => {
                self.run_disclaimer_check().await?;
            }
            Screen::Release => {
                if let Some(i) = self.crate_state.selected() {
                    if let Some(crate_info) = self.workspace.crates.get(i) {
                        let name = crate_info.name.clone();
                        self.run_release(&name).await?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn back(&mut self) {
        self.current_screen = match &self.current_screen {
            Screen::MainMenu => Screen::MainMenu,
            _ => {
                self.status_message = "Back to main menu".to_string();
                Screen::MainMenu
            }
        };
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.loading = true;
        self.status_message = "Refreshing workspace data...".to_string();

        match &self.current_screen {
            Screen::SizeComparison => {
                self.refresh_sizes().await?;
            }
            _ => {
                self.workspace = WorkspaceInfo::load().await?;
            }
        }

        self.loading = false;
        self.status_message = "Refreshed successfully".to_string();
        Ok(())
    }

    async fn refresh_sizes(&mut self) -> Result<()> {
        self.workspace.calculate_local_sizes().await?;
        self.workspace.fetch_published_sizes().await?;
        self.status_message = "Crate sizes updated".to_string();
        Ok(())
    }

    async fn run_code_quality_task(&mut self) -> Result<()> {
        self.loading = true;
        self.status_message = "Running fmt and clippy...".to_string();

        // Run fmt
        if let Err(e) = run_fmt().await {
            self.status_message = format!("fmt failed: {}", e);
            self.loading = false;
            return Ok(());
        }

        // Run clippy
        if let Err(e) = run_clippy().await {
            self.status_message = format!("clippy failed: {}", e);
            self.loading = false;
            return Ok(());
        }

        self.loading = false;
        self.status_message = "✅ Code quality checks completed successfully".to_string();
        Ok(())
    }

    async fn run_disclaimer_check(&mut self) -> Result<()> {
        self.loading = true;
        self.status_message = "Checking disclaimers...".to_string();

        if let Err(e) = check_disclaimers().await {
            self.status_message = format!("Disclaimer check failed: {}", e);
        } else {
            self.status_message = "✅ Disclaimer check completed".to_string();
        }

        self.loading = false;
        Ok(())
    }

    async fn run_release(&mut self, crate_name: &str) -> Result<()> {
        self.loading = true;
        self.status_message = format!("Releasing {}...", crate_name);

        if let Err(e) = release_crates(Some(crate_name.to_string()), true).await {
            self.status_message = format!("Release failed: {}", e);
        } else {
            self.status_message = format!("✅ {} released successfully", crate_name);
        }

        self.loading = false;
        Ok(())
    }

    /*pub fn get_selected_crate(&self) -> Option<&crate::workspace::CrateInfo> {
        self.crate_state
            .selected()
            .and_then(|i| self.workspace.crates.get(i))
    }*/
}
