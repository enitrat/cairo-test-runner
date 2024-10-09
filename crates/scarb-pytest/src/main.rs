use clap::Parser;
use scarb_pytest::{Args, run_scarb_pytest};
use anyhow::Result;

fn main() -> Result<()> {
    let args: Args = Args::parse();
    if let Err(err) = run_scarb_pytest(args) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    Ok(())
}
