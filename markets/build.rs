// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Build script para configurar linkagem da ProfitDLL em Windows
//!
//! Este script configura automaticamente a linkagem da ProfitDLL.dll
//! quando compilando para Windows, incluindo:
//! - Localização automática da DLL
//! - Configuração de diretórios de busca
//! - Validação de arquivos necessários

use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Configuração específica para Windows
    if cfg!(target_os = "windows") {
        configure_windows_dll();
    } else {
        println!("cargo:warning=ProfitDLL só é suportada em Windows. Usando implementação mock.");
    }
}

fn configure_windows_dll() {
    println!("🔧 Configurando ProfitDLL para Windows...");

    // Diretórios padrão onde a ProfitDLL pode estar instalada
    let possible_paths = vec![
        "C:\\Program Files\\Nelogica\\ProfitDLL",
        "C:\\Program Files (x86)\\Nelogica\\ProfitDLL",
        "C:\\ProfitDLL",
        ".",
        "./lib",
        "./dll",
    ];

    // Tentar localizar ProfitDLL.dll
    let mut dll_found = false;
    for path in &possible_paths {
        let dll_path = Path::new(path).join("ProfitDLL.dll");
        if dll_path.exists() {
            println!("✅ ProfitDLL.dll encontrada em: {path}");
            println!("cargo:rustc-link-search=native={path}");
            dll_found = true;
            break;
        }
    }

    if !dll_found {
        // Verificar variável de ambiente
        if let Ok(dll_path) = env::var("PROFITDLL_PATH") {
            let dll_file = Path::new(&dll_path).join("ProfitDLL.dll");
            if dll_file.exists() {
                println!("✅ ProfitDLL.dll encontrada via PROFITDLL_PATH: {dll_path}");
                println!("cargo:rustc-link-search=native={dll_path}");
                dll_found = true;
            }
        }
    }

    if dll_found {
        // Não configuramos linkagem estática: carregamento dinâmico puro via libloading.
        // Definir feature condicional
        println!("cargo:rustc-cfg=feature=\"real_dll\"");

        println!("🚀 ProfitDLL configurada com sucesso!");
    } else {
        println!("⚠️  ProfitDLL.dll não encontrada. Para usar a DLL real:");
        println!("   1. Instale a ProfitDLL da Nelógica");
        println!("   2. Ou defina PROFITDLL_PATH com o caminho da DLL");
        println!("   3. Ou coloque ProfitDLL.dll no diretório do projeto");
        println!("   Usando implementação mock.");

        // Definir feature para mock
        println!("cargo:rustc-cfg=feature=\"mock_dll\"");
    }

    // Configurações adicionais do Windows
    println!("cargo:rustc-link-lib=kernel32");
    println!("cargo:rustc-link-lib=user32");
}
