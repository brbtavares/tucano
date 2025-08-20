// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.

use cargo_metadata::{MetadataCommand, Package};
use std::fs;
use std::io::Write;
use chrono::{Local, Datelike, Timelike};

pub fn run_inventory() -> anyhow::Result<()> {
    let metadata = MetadataCommand::new().exec()?;
    let mut report = String::new();
    let workspace_root = metadata.workspace_root.as_std_path();
    std::fs::create_dir_all("xtask/reports")?;
    let workspace_crates: std::collections::HashSet<_> = metadata.packages.iter()
        .filter(|p| p.manifest_path.starts_with(&metadata.workspace_root))
        .map(|p| p.name.as_str())
        .collect();

    // Mapa de dependentes: crate -> Vec<dependente>
    let mut dependents: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for package in &metadata.packages {
        if !package.manifest_path.starts_with(&metadata.workspace_root) {
            continue;
        }
        for dep in &package.dependencies {
            if workspace_crates.contains(dep.name.as_str()) {
                dependents.entry(dep.name.as_str()).or_default().push(package.name.as_str());
            }
        }
    }

    for package in &metadata.packages {
        if !package.manifest_path.starts_with(&metadata.workspace_root) {
            continue;
        }
        report.push_str(&format!("-------------------------\n# Crate: {}\n-------------------------\n", package.name));
        report.push_str("Dependencies:\n");
        for dep in &package.dependencies {
            if workspace_crates.contains(dep.name.as_str()) {
                report.push_str(&format!("  - {} ({})\n", dep.name, dep.req));
            }
        }
        // Dependents
        report.push_str("Dependents:\n");
        if let Some(deps) = dependents.get(package.name.as_str()) {
            for dep in deps {
                report.push_str(&format!("  - {}\n", dep));
            }
        }
        report.push_str("Public API:\n");
        if let Some(src_path) = find_lib_rs(&package) {
            let src = fs::read_to_string(&src_path)?;
            let mut last_header: Option<String> = None;
            let mut in_pub_fn = false;
            let mut fn_lines = Vec::new();
            let mut in_enum = false;
            let mut enum_has_variant = false;
            for line in src.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("pub use") || trimmed.starts_with("pub mod") {
                    continue;
                }
                // Captura função pública multi-linha
                if in_pub_fn {
                    fn_lines.push(trimmed);
                    if trimmed.ends_with('{') || trimmed.ends_with(';') {
                        let signature = fn_lines.join(" ");
                        let signature = signature.trim_end_matches('{').trim_end_matches(';').trim();
                        if last_header.is_some() {
                            report.push_str(&format!("    {}\n", signature));
                        } else {
                            report.push_str(&format!("  pub {}\n", signature.strip_prefix("pub ").unwrap_or(signature)));
                        }
                        in_pub_fn = false;
                        fn_lines.clear();
                    }
                    continue;
                }
                if trimmed.starts_with("pub fn ") {
                    in_pub_fn = true;
                    fn_lines.clear();
                    fn_lines.push(trimmed);
                    continue;
                }
                // Detecta início de enum
                if let Some(header) = trimmed.strip_prefix("pub ") {
                    let header = header.trim_end_matches(';').trim_end_matches('{').trim();
                    if header.starts_with("enum ") {
                        last_header = Some(header.to_string());
                        in_enum = true;
                        enum_has_variant = false;
                        continue;
                    }
                    if header.starts_with("trait ") || header.starts_with("struct ") || header.starts_with("type ") {
                        report.push_str(&format!("  pub {}\n", header));
                        last_header = Some(header.to_string());
                        in_enum = false;
                        continue;
                    }
                    // Campos públicos
                    if last_header.is_some() && header.contains(':') {
                        report.push_str(&format!("    {}\n", header));
                        continue;
                    }
                    // Variantes de enum
                    if in_enum && (header.starts_with('|') || (!header.starts_with("fn ") && !header.contains(':') && !header.starts_with("type "))) {
                        if !enum_has_variant {
                            // Só imprime o cabeçalho do enum se houver pelo menos uma variante
                            report.push_str(&format!("  pub {}\n", last_header.as_ref().unwrap()));
                            enum_has_variant = true;
                        }
                        report.push_str(&format!("    {}\n", header.trim_start_matches('|').trim()));
                        continue;
                    }
                    // Outros itens públicos
                    report.push_str(&format!("  pub {}\n", header));
                } else {
                    if in_enum && !enum_has_variant {
                        // Não imprime enums sem variantes
                        last_header = None;
                        in_enum = false;
                        continue;
                    }
                    last_header = None;
                    in_enum = false;
                }
            }
        }
        report.push('\n');
    }
    // Gerar nome com timestamp
    let now = Local::now();
    let filename = format!(
        "xtask/reports/inventory_report_{:04}_{:02}_{:02}_{:02}_{:02}_{:02}.txt",
        now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second()
    );
    let mut file = fs::File::create(&filename)?;
    file.write_all(report.as_bytes())?;
    println!("[inventory] Report saved to {}", filename);
    Ok(())
}

fn find_lib_rs(package: &Package) -> Option<std::path::PathBuf> {
    let manifest_dir = package.manifest_path.parent()?;
    let src_lib = manifest_dir.join("src/lib.rs");
    if src_lib.exists() {
        Some(src_lib.into())
    } else {
        None
    }
}

fn print_public_items(src: &str) {
    for line in src.lines() {
        if line.trim_start().starts_with("pub ") {
            println!("  {}", line.trim());
        }
    }
}
