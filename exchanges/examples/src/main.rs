

fn main() {
    println!("toucan-examples: available examples (each as a separate bin):");
    println!(" - example_1_live_login        (login + subscribe; mock or live)");
    println!(" - example_2_get_history_trades (trade history; mock or live)");
    println!(" - mock_minimal                 (minimal order send + events)");
    println!("Usage: cargo run -p toucan-examples --bin <name> [--features real_dll]");
}
