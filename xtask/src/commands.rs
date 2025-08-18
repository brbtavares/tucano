use anyhow::Result;
use chrono::Datelike;
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;
use walkdir::WalkDir;

pub async fn run_fmt() -> Result<()> {
    println!("🎨 Running cargo fmt --all...");

    let output = TokioCommand::new("cargo")
        .args(&["fmt", "--all"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo fmt failed: {}", stderr);
    }

    println!("✅ Format completed successfully");
    Ok(())
}

pub async fn run_clippy() -> Result<()> {
    println!("🔍 Running cargo clippy...");

    let output = TokioCommand::new("cargo")
        .args(&[
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("{}", stdout);

    if !output.status.success() {
        println!("⚠️  Clippy found issues:");
        println!("{}", stderr);
    } else {
        println!("✅ Clippy checks passed");
    }

    Ok(())
}

pub async fn check_disclaimers() -> Result<()> {
    let disclaimer_template = crate::workspace::get_default_disclaimer();
    println!("📝 Checking disclaimers with template:");
    println!("{}\n", disclaimer_template);

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()?;

    let mut issues_found = 0;

    for package in metadata.workspace_packages() {
        if package.name == "xtask" {
            continue;
        }

        let crate_path = package.manifest_path.parent().unwrap().as_std_path();
        let missing_files = find_files_missing_disclaimer(crate_path, &disclaimer_template)?;

        if !missing_files.is_empty() {
            issues_found += missing_files.len();
            println!(
                "📦 Crate '{}' - {} files missing disclaimer:",
                package.name,
                missing_files.len()
            );
            for file in &missing_files {
                println!("  - {}", file.display());
            }
            println!();
        }
    }

    if issues_found > 0 {
        println!("❌ Found {} files missing disclaimer", issues_found);
        println!("Run with --fix to add disclaimers automatically");
    } else {
        println!("✅ All files have proper disclaimers");
    }

    Ok(())
}

pub async fn add_disclaimers(fix: bool) -> Result<()> {
    let disclaimer_template = crate::workspace::get_default_disclaimer();

    if !fix {
        println!("🔍 DRY RUN - Use --fix to apply changes");
    } else {
        println!("📝 Adding disclaimers to files...");
    }

    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()?;

    let mut files_modified = 0;

    for package in metadata.workspace_packages() {
        if package.name == "xtask" {
            continue;
        }

        let crate_path = package.manifest_path.parent().unwrap().as_std_path();
        let missing_files = find_files_missing_disclaimer(crate_path, &disclaimer_template)?;

        for file_path in missing_files {
            if fix {
                add_disclaimer_to_file(&file_path, &disclaimer_template)?;
                files_modified += 1;
                println!("✅ Added disclaimer to {}", file_path.display());
            } else {
                println!("Would add disclaimer to {}", file_path.display());
                files_modified += 1;
            }
        }
    }

    if fix {
        println!("🎉 Added disclaimers to {} files", files_modified);
    } else {
        println!("Would modify {} files (use --fix to apply)", files_modified);
    }

    Ok(())
}

pub async fn show_size_comparison() -> Result<()> {
    println!("📊 Calculating crate sizes...");

    let mut workspace = crate::workspace::WorkspaceInfo::load().await?;

    // Calculate local sizes
    workspace.calculate_local_sizes().await?;

    // Fetch published sizes (this might take a while)
    println!("🌐 Fetching published sizes from crates.io...");
    workspace.fetch_published_sizes().await?;

    // Display comparison table
    println!("\n📦 Crate Size Comparison:");
    println!("┌─────────────────────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ Crate Name                  │ Local Size   │ Published    │ Difference   │");
    println!("├─────────────────────────────┼──────────────┼──────────────┼──────────────┤");

    for crate_info in &workspace.crates {
        let diff_str = match crate_info.size_diff() {
            Some(diff) if diff > 0 => format!("+{:.2} MiB", diff as f64 / (1024.0 * 1024.0)),
            Some(diff) if diff < 0 => format!("-{:.2} MiB", (-diff) as f64 / (1024.0 * 1024.0)),
            Some(_) => "same".to_string(),
            None => "unknown".to_string(),
        };

        println!(
            "│ {:<27} │ {:>12} │ {:>12} │ {:>12} │",
            truncate_string(&crate_info.name, 27),
            crate_info.local_size_mb(),
            crate_info.published_size_mb(),
            diff_str
        );
    }

    println!("└─────────────────────────────┴──────────────┴──────────────┴──────────────┘");

    Ok(())
}

pub async fn release_crates(crate_name: Option<String>, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("🔍 DRY RUN - No actual publishing will occur");
    }

    let workspace = crate::workspace::WorkspaceInfo::load().await?;

    let crates_to_release = if let Some(name) = crate_name {
        workspace
            .crates
            .into_iter()
            .filter(|c| c.name == name)
            .collect::<Vec<_>>()
    } else {
        workspace.crates
    };

    if crates_to_release.is_empty() {
        println!("❌ No crates found to release");
        return Ok(());
    }

    println!("🚀 Release plan for {} crate(s):", crates_to_release.len());

    for crate_info in &crates_to_release {
        println!("\n📦 Crate: {}", crate_info.name);
        println!("   Version: {}", crate_info.version);
        println!("   Path: {}", crate_info.path.display());

        // Pre-release checks
        println!("   Running pre-release checks...");

        // Check if tests pass
        let test_result = run_crate_tests(&crate_info.path).await?;
        if !test_result {
            println!("   ❌ Tests failed for {}", crate_info.name);
            continue;
        }

        // Check if already published
        if is_version_published(&crate_info.name, &crate_info.version).await? {
            println!("   ⚠️  Version {} already published", crate_info.version);
            continue;
        }

        // Publish
        if !dry_run {
            println!("   📤 Publishing to crates.io...");
            publish_crate(&crate_info.path).await?;
            println!("   ✅ Successfully published {}", crate_info.name);
        } else {
            println!("   📤 Would publish to crates.io (dry run)");
        }
    }

    println!("\n🎉 Release process completed!");
    Ok(())
}

// Helper functions

fn find_files_missing_disclaimer(
    crate_path: &std::path::Path,
    disclaimer_template: &str,
) -> Result<Vec<PathBuf>> {
    let mut missing_files = Vec::new();
    let disclaimer_lines: Vec<&str> = disclaimer_template.lines().collect();

    for entry in WalkDir::new(crate_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip target directory and other build artifacts
        if path
            .components()
            .any(|c| matches!(c.as_os_str().to_string_lossy().as_ref(), "target" | ".git"))
        {
            continue;
        }

        // Only check .rs files
        if path.extension().map_or(false, |ext| ext == "rs") {
            if !has_disclaimer(path, &disclaimer_lines)? {
                missing_files.push(path.to_path_buf());
            }
        }
    }

    Ok(missing_files)
}

fn has_disclaimer(file_path: &std::path::Path, disclaimer_lines: &[&str]) -> Result<bool> {
    let content = std::fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if disclaimer_lines.len() > lines.len() {
        return Ok(false);
    }

    for (i, template_line) in disclaimer_lines.iter().enumerate() {
        let actual_line = lines[i]
            .trim_start_matches("//")
            .trim_start_matches("/*")
            .trim();
        let template_line = template_line.trim();

        // Handle dynamic year in template
        if template_line.contains("{year}") {
            let current_year = chrono::Utc::now().year_ce().1.to_string();
            let expected = template_line.replace("{year}", &current_year);
            if actual_line != expected.trim() {
                return Ok(false);
            }
        } else if actual_line != template_line {
            return Ok(false);
        }
    }

    Ok(true)
}

fn add_disclaimer_to_file(file_path: &PathBuf, disclaimer_template: &str) -> Result<()> {
    let content = std::fs::read_to_string(file_path)?;

    // Format disclaimer with proper comment syntax
    let formatted_disclaimer = disclaimer_template
        .lines()
        .map(|line| format!("// {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    let new_content = format!("{}\n\n{}", formatted_disclaimer, content);
    std::fs::write(file_path, new_content)?;

    Ok(())
}

async fn run_crate_tests(crate_path: &PathBuf) -> Result<bool> {
    let output = TokioCommand::new("cargo")
        .args(&["test"])
        .current_dir(crate_path)
        .output()
        .await?;

    Ok(output.status.success())
}

async fn is_version_published(crate_name: &str, version: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("https://crates.io/api/v1/crates/{}/{}", crate_name, version);

    let response = client
        .get(&url)
        .header("User-Agent", "xtask-workspace-manager")
        .send()
        .await?;

    Ok(response.status().is_success())
}

async fn publish_crate(crate_path: &PathBuf) -> Result<()> {
    let output = TokioCommand::new("cargo")
        .args(&["publish"])
        .current_dir(crate_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to publish crate: {}", stderr);
    }

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
