// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use anyhow::Result;
use chrono::Datelike;
use std::path::PathBuf;
use walkdir::WalkDir;
use crate::workspace::*;

pub async fn check_disclaimers() -> Result<()> {
    let disclaimer_template = get_default_disclaimer();
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
        if path.extension().is_some_and(|ext| ext == "rs")
            && !has_disclaimer(path, &disclaimer_lines)?
        {
            missing_files.push(path.to_path_buf());
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
