

//! Build script to configure linking of ProfitDLL on Windows
//!
//! This script automatically configures linking of ProfitDLL.dll
//! when compiling for Windows, including:
//! - Automatic DLL location
//! - Search directory configuration
//! - Validation of required files

use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Windows-specific configuration
    if cfg!(target_os = "windows") {
        configure_windows_dll();
    } else {
    println!("cargo:warning=ProfitDLL is only supported on Windows. Using mock implementation.");
    }
}

fn configure_windows_dll() {
    println!("🔧 Configuring ProfitDLL for Windows...");

    // Default directories where ProfitDLL may be installed
    let possible_paths = vec!["C:\\ProfitDLL", ".", "./lib", "./dll"];

    // Try to locate ProfitDLL.dll
    let mut dll_found = false;
    for path in &possible_paths {
        let dll_path = Path::new(path).join("ProfitDLL.dll");
        if dll_path.exists() {
            println!("✅ ProfitDLL.dll found at: {path}");
            println!("cargo:rustc-link-search=native={path}");
            dll_found = true;
            break;
        }
    }

    if !dll_found {
        // Check environment variable
        if let Ok(dll_path) = env::var("PROFITDLL_PATH") {
            let dll_file = Path::new(&dll_path).join("ProfitDLL.dll");
            if dll_file.exists() {
                println!("✅ ProfitDLL.dll found via PROFITDLL_PATH: {dll_path}");
                println!("cargo:rustc-link-search=native={dll_path}");
                dll_found = true;
            }
        }
    }

    if dll_found {
        // We do not configure static linking: pure dynamic loading via libloading.
        // Set conditional feature
        println!("cargo:rustc-cfg=feature=\"real_dll\"");

        println!("🚀 ProfitDLL successfully configured!");
    } else {
        println!("⚠️  ProfitDLL.dll not found. To use the real DLL:");
    println!("   1. Install ProfitDLL from Nelogica");
        println!("   2. Or set PROFITDLL_PATH with the DLL path");
        println!("   3. Or place ProfitDLL.dll in the project directory");
        println!("   Using mock implementation.");

        // Set feature for mock
        println!("cargo:rustc-cfg=feature=\"mock_dll\"");
    }

    // Additional Windows configurations
    println!("cargo:rustc-link-lib=kernel32");
    println!("cargo:rustc-link-lib=user32");
}
