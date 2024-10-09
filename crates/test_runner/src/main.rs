use anyhow::Result;
use cairo_vm::Felt252 as Felt;
use clap::Parser;
use log::info;
use std::path::{Path, PathBuf};
use test_runner::test_utils::load_and_run_cairo_function;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Sierra program JSON file
    #[arg(short, long)]
    sierra_path: PathBuf,

    /// Name of the function to run
    #[arg(short, long)]
    function: String,

    /// Arguments for the function, as a JSON array
    #[arg(short, long)]
    args: String,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    info!("Running function {} with args {}", args.function, args.args);

    let sierra_path = Path::new(&args.sierra_path);
    let result = load_and_run_cairo_function::<Vec<Felt>>(&args.function, &sierra_path, &args.args)?;
    println!("Result: {:?}", result);

    Ok(())
}
