
use anyhow::Result;
use tokio::process::Command as TokioCommand;

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

async fn run_crate_tests(crate_path: &std::path::PathBuf) -> Result<bool> {
    let output = TokioCommand::new("cargo")
        .args(["test"])
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

async fn publish_crate(crate_path: &std::path::PathBuf) -> Result<()> {
    let output = TokioCommand::new("cargo")
        .args(["publish"])
        .current_dir(crate_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to publish crate: {}", stderr);
    }

    Ok(())
}
