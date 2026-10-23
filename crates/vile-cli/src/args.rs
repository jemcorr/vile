//! Command-line arguments (clap derive). Keep flag names exactly as written:
//! the integration tests and PLAN.md depend on them.
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Edit Vial .vil layout files safely. Commands that change a file print a
/// semantic diff and write nothing unless --write or --output is given.
#[derive(Parser)]
#[command(name = "vile", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Parse and validate a .vil file and summarize it
    Check { file: PathBuf },
    /// Print the semantic diff from OLD to NEW
    Diff { old: PathBuf, new: PathBuf },
    /// Layer operations
    Layer {
        #[command(subcommand)]
        op: LayerCommand,
    },
}

#[derive(Subcommand)]
pub enum LayerCommand {
    /// Set every key of LAYER to KC_NO, or to KC_TRNS with --transparent
    Clear {
        file: PathBuf,
        layer: usize,
        #[arg(long)]
        transparent: bool,
        #[command(flatten)]
        out: OutputArgs,
    },
    /// Overwrite layer DST with a copy of layer SRC
    Copy {
        file: PathBuf,
        src: usize,
        dst: usize,
        #[command(flatten)]
        out: OutputArgs,
    },
    /// Swap layers A and B and renumber every reference to them
    Swap {
        file: PathBuf,
        a: usize,
        b: usize,
        #[command(flatten)]
        out: OutputArgs,
    },
}

#[derive(Args)]
pub struct OutputArgs {
    /// Write the result back to FILE (atomic replace)
    #[arg(short = 'w', long, conflicts_with = "output")]
    pub write: bool,
    /// Write the result to PATH and leave FILE unchanged
    #[arg(short = 'o', long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}
