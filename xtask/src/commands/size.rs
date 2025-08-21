
use anyhow::Result;

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
    println!("┌─────────────────────────────┬──────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│ Crate Name                  │ Local Size   │ Published    │ % Diff       │ Bytes Diff   │");
    println!("├─────────────────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤");

    for crate_info in &workspace.crates {
        let percent_str = match crate_info.percent_diff() {
            Some(p) => format!("{:+.2}%", p),
            None => "n/a".to_string(),
        };
        let bytes_diff = match (crate_info.local_size, crate_info.published_size) {
            (Some(local), Some(published)) => format!("{:+} B", (local as i64 - published as i64)),
            _ => "n/a".to_string(),
        };
        println!(
            "│ {:<27} │ {:>12} │ {:>12} │ {:>12} │ {:>12} │",
            truncate_string(&crate_info.name, 27),
            crate_info.local_size_bytes(),
            crate_info.published_size_bytes(),
            percent_str,
            bytes_diff
        );
    }

    println!("└─────────────────────────────┴──────────────┴──────────────┴──────────────┴──────────────┘");

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
