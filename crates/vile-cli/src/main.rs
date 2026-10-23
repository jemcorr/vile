use clap::Parser;

/// Edit Vial .vil layout files safely.
#[derive(Parser)]
#[command(name = "vile", version)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
    println!("vile: no command yet; run vile --help");
}
