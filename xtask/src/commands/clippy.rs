// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use anyhow::Result;
use tokio::process::Command as TokioCommand;

pub async fn run_clippy() -> Result<()> {
    println!("🔍 Running cargo clippy...");

    let output = TokioCommand::new("cargo")
        .args([
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
