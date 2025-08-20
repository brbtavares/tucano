// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use anyhow::Result;
use tokio::process::Command as TokioCommand;

pub async fn run_fmt() -> Result<()> {
    println!("🎨 Running cargo fmt --all...");

    let output = TokioCommand::new("cargo")
        .args(["fmt", "--all"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo fmt failed: {}", stderr);
    }

    println!("✅ Format completed successfully");
    Ok(())
}
