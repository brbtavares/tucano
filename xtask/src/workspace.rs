// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.

use anyhow::Result;
use cargo_metadata::MetadataCommand;
use chrono::Datelike;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub crates: Vec<CrateInfo>,
    //pub root_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub local_size: Option<u64>,     // em bytes
    pub published_size: Option<u64>, // em bytes
    pub dependencies: Vec<String>,
    //pub has_disclaimer: Option<bool>,
    //pub fmt_status: Option<bool>,
    //pub clippy_issues: Option<u32>,
}

impl WorkspaceInfo {
    pub async fn load() -> Result<Self> {
        let metadata = MetadataCommand::new()
            .manifest_path("./Cargo.toml")
            .exec()?;

        let mut crates = Vec::new();

        for package in metadata.workspace_packages() {
            // Skip xtask itself
            if package.name == "xtask" {
                continue;
            }

            let crate_info = CrateInfo {
                name: package.name.clone(),
                version: package.version.to_string(),
                path: package
                    .manifest_path
                    .parent()
                    .unwrap_or_else(|| &package.manifest_path)
                    .as_std_path()
                    .to_path_buf(),
                local_size: None,
                published_size: None,
                dependencies: package
                    .dependencies
                    .iter()
                    .map(|dep| dep.name.clone())
                    .collect(),
                //has_disclaimer: None,
                //fmt_status: None,
                //clippy_issues: None,
            };

            crates.push(crate_info);
        }

        Ok(WorkspaceInfo {
            crates,
            //root_path: metadata.workspace_root.as_std_path().to_path_buf(),
        })
    }

    pub async fn calculate_local_sizes(&mut self) -> Result<()> {
        for crate_info in &mut self.crates {
            crate_info.local_size = Some(calculate_crate_size(&crate_info.path)?);
        }
        Ok(())
    }

    pub async fn fetch_published_sizes(&mut self) -> Result<()> {
        let client = reqwest::Client::new();

        for crate_info in &mut self.crates {
            if let Ok(size) = fetch_crates_io_size(&client, &crate_info.name).await {
                crate_info.published_size = Some(size);
            }
        }
        Ok(())
    }

    /*pub async fn check_disclaimers(&mut self, disclaimer_template: &str) -> Result<()> {
        for crate_info in &mut self.crates {
            crate_info.has_disclaimer = Some(check_crate_disclaimer(
                &crate_info.path,
                disclaimer_template,
            )?);
        }
        Ok(())
    }*/
}

impl CrateInfo {
    pub fn local_size_bytes(&self) -> String {
        match self.local_size {
            Some(size) => format!("{} B", size),
            None => "calculating...".to_string(),
        }
    }

    pub fn published_size_bytes(&self) -> String {
        match self.published_size {
            Some(size) => format!("{} B", size),
            None => "unknown".to_string(),
        }
    }

    pub fn percent_diff(&self) -> Option<f64> {
        match (self.local_size, self.published_size) {
            (Some(local), Some(published)) if published > 0 => {
                Some(((local as f64 - published as f64) / published as f64) * 100.0)
            }
            _ => None,
        }
    }
}

fn calculate_crate_size(crate_path: &PathBuf) -> Result<u64> {
    use walkdir::WalkDir;

    let mut total_size = 0u64;

    for entry in WalkDir::new(crate_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip target, node_modules, .git etc
        if path.components().any(|c| {
            matches!(
                c.as_os_str().to_string_lossy().as_ref(),
                "target" | "node_modules" | ".git" | ".svn"
            )
        }) {
            continue;
        }

        // Only count source files
        if let Some(ext) = path.extension() {
            if matches!(ext.to_string_lossy().as_ref(), "rs" | "toml" | "md" | "txt") {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }
    }

    Ok(total_size)
}

async fn fetch_crates_io_size(client: &reqwest::Client, crate_name: &str) -> Result<u64> {
    // Isso é uma estimativa - crates.io não fornece tamanho diretamente
    // Você pode fazer download do .crate file e verificar o tamanho
    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);

    let response = client
        .get(&url)
        .header("User-Agent", "xtask-workspace-manager")
        .send()
        .await?;

    if response.status().is_success() {
        // Por enquanto retornar um placeholder
        // Implementação completa requer download do arquivo .crate
        Ok(1024 * 50) // ~50KB placeholder
    } else {
        anyhow::bail!("Failed to fetch crate info from crates.io");
    }
}

pub fn get_default_disclaimer() -> String {
    "Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.".to_string()
}
