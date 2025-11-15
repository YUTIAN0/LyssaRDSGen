//! LyssaRDSGen - RDS License Key Generator
//! 
//! This application generates Service Provider Keys (SPKs) and License Key Packs (LKPs)
//! for Microsoft Remote Desktop Services.

// Hide console window on Windows in release mode when using GUI
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions), feature = "gui"),
    windows_subsystem = "windows"
)]

mod cli;
mod crypto;
mod keygen;
mod types;

#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "tui")]
mod tui;

use std::env;

fn main() {
    // Check if we should run GUI or TUI mode
    let args: Vec<String> = env::args().collect();
    
    // Check for explicit --tui flag
    let run_tui = args.contains(&"--tui".to_string());
    
    // Run GUI if:
    // 1. No arguments provided (just the program name)
    // 2. Only --gui flag is provided
    // 3. --gui flag is provided without other CLI arguments
    let run_gui = !run_tui && (
        args.len() == 1 || 
        (args.len() == 2 && args.contains(&"--gui".to_string())) ||
        (args.contains(&"--gui".to_string()) && 
         !args.iter().any(|a| a.starts_with("--") && a != "--gui"))
    );
    
    #[cfg(feature = "tui")]
    if run_tui {
        if let Err(e) = tui::run_tui() {
            eprintln!("TUI Error: {}", e);
            std::process::exit(1);
        }
        return;
    }
    
    #[cfg(not(feature = "tui"))]
    if run_tui {
        eprintln!("TUI feature not enabled. Rebuild with --features tui");
        std::process::exit(1);
    }
    
    #[cfg(feature = "gui")]
    if run_gui {
        if let Err(e) = gui::run_gui() {
            eprintln!("GUI Error: {}", e);
            std::process::exit(1);
        }
        return;
    }
    
    #[cfg(not(feature = "gui"))]
    if run_gui {
        eprintln!("GUI feature not enabled. Rebuild with --features gui");
        std::process::exit(1);
    }
    
    // Run CLI mode
    if let Err(e) = cli::run_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
