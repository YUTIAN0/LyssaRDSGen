//! Command-line interface

use crate::keygen::{generate_lkp, generate_spk, validate_tskey};
use crate::types::{LicenseInfo, SPKCurve, LICENSE_TYPES};
use clap::Parser;

#[derive(Parser)]
#[command(name = "lyssa_rds_gen")]
#[command(author = "LyssaRDSGen Contributors")]
#[command(version = "1.0.0")]
#[command(about = "Generate RDS License Keys", long_about = "Generate RDS License Keys\n\nRun without arguments or with --gui to launch GUI mode.\nProvide arguments to use CLI mode.")]
pub struct Cli {
    /// Launch GUI mode (graphical interface)
    #[arg(long, conflicts_with = "tui")]
    pub gui: bool,

    /// Launch TUI mode (terminal interface)
    #[arg(long, conflicts_with = "gui")]
    pub tui: bool,
    /// Product ID (e.g., 00490-92005-99454-AT527)
    #[arg(long)]
    pub pid: Option<String>,

    /// Existing License Server ID (SPK) - skip SPK generation and only generate LKP
    #[arg(long)]
    pub spk: Option<String>,

    /// License count (1-9999) - generates LKP when provided with --license
    #[arg(long)]
    pub count: Option<u32>,

    /// License version and type (e.g., 029_10_2) - generates LKP when provided with --count
    #[arg(long)]
    pub license: Option<String>,

    /// List all supported license types
    #[arg(long)]
    pub list: bool,
}

pub fn run_cli() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle --list flag
    if cli.list {
        list_licenses();
        return Ok(());
    }

    // Require PID for key generation
    let pid = cli.pid.as_ref().ok_or_else(|| {
        anyhow::anyhow!("--pid is required for key generation. Use --help for more information.")
    })?;

    // Validate --spk parameter requirements
    if cli.spk.is_some() && (cli.count.is_none() || cli.license.is_none()) {
        anyhow::bail!("When using --spk, both --count and --license must be provided");
    }

    // Validate LKP parameters if either is provided
    if (cli.count.is_none()) != (cli.license.is_none()) {
        anyhow::bail!("Both --count and --license must be provided together for LKP generation");
    }

    println!("Generating keys for PID: {}\n", pid);

    // Handle SPK - either validate existing or generate new
    let _spk = if let Some(existing_spk) = &cli.spk {
        println!("{}", "=".repeat(60));
        println!("Validating provided SPK: {}", existing_spk);
        
        let is_valid = validate_tskey(
            pid,
            existing_spk,
            SPKCurve::gx(),
            SPKCurve::gy(),
            SPKCurve::kx(),
            SPKCurve::ky(),
            num_bigint::BigUint::from(SPKCurve::A),
            SPKCurve::p(),
            true,
        )?;
        
        if !is_valid {
            println!("{}", "=".repeat(60));
            anyhow::bail!("Provided SPK does not match the PID");
        }
        
        println!("SPK validation successful!");
        println!("{}", "=".repeat(60));
        existing_spk.clone()
    } else {
        println!("{}", "=".repeat(60));
        let spk = generate_spk(pid)?;
        println!("License Server ID (SPK):\n{}", spk);
        println!("{}", "=".repeat(60));
        spk
    };

    // Generate LKP if parameters provided
    if let (Some(count), Some(license_type)) = (cli.count, cli.license.as_ref()) {
        let license_info = LicenseInfo::parse(license_type)?;

        if !(1..=9999).contains(&count) {
            anyhow::bail!("License count must be between 1 and 9999");
        }

        println!("\nLicense Type: {}", license_info.description);
        println!("License Count: {}\n", count);
        println!("{}", "=".repeat(60));
        
        let lkp = generate_lkp(
            pid,
            count,
            license_info.chid,
            license_info.major_ver,
            license_info.minor_ver,
        )?;
        
        println!("License Key Pack (LKP):\n{}", lkp);
        println!("{}", "=".repeat(60));
    }

    println!();
    Ok(())
}

fn list_licenses() {
    println!("\nSupported License Version and Type:\n");
    for (code, description) in LICENSE_TYPES {
        println!("  {:12} - {}", code, description);
    }
    println!();
}
