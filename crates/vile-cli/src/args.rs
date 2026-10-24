//! Command-line arguments (clap derive). Keep flag names exactly as written:
//! the integration tests and PLAN.md depend on them.
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Edit Vial .vil layout files safely. Commands that change a file print a
/// semantic diff and write nothing unless --write or --output is given.
#[derive(Parser)]
#[command(name = "vile", version)]
pub struct Cli {
    /// Use a local Vial definition for geometry, names and explicit mirror pairs.
    #[arg(long, global = true)]
    pub definition: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Parse and validate a .vil file and summarize it
    Check { file: PathBuf },
    /// Print the semantic diff from OLD to NEW, or from git HEAD to OLD with --git
    Diff {
        old: PathBuf,
        #[arg(required_unless_present = "git")]
        new: Option<PathBuf>,
        /// Compare the committed version (git HEAD) of OLD with OLD
        #[arg(long, conflicts_with = "new")]
        git: bool,
    },
    /// Print layers as text
    Show {
        file: PathBuf,
        /// Only this layer
        #[arg(long)]
        layer: Option<usize>,
    },
    /// Search keycodes by name or alias
    Keycodes {
        query: Option<String>,
        /// Maximum number of results; 0 means no limit
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// Use the layer, macro and tap dance counts of this file
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Set one key: KEY is a board key id (thumb-left-mid) or a matrix position (r3c4)
    Set {
        file: PathBuf,
        layer: usize,
        key: String,
        keycode: String,
        #[command(flatten)]
        out: OutputArgs,
    },
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
    /// Move layer FROM to index TO, shift the layers in between, renumber references
    Move {
        file: PathBuf,
        from: usize,
        to: usize,
        #[command(flatten)]
        out: OutputArgs,
    },
    /// Mirror LAYER between the left and right halves
    Mirror {
        file: PathBuf,
        layer: usize,
        #[arg(long, value_enum)]
        mode: MirrorArg,
        #[command(flatten)]
        out: OutputArgs,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum MirrorArg {
    /// Exchange the two halves
    Swap,
    /// Copy the left half onto the right half
    LeftToRight,
    /// Copy the right half onto the left half
    RightToLeft,
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
