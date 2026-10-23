//! The vile command-line tool.

mod args;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, Result, anyhow, bail};
use clap::Parser;
use vile_core::diff::{diff, format_diff};
use vile_core::ops::{copy_layer, fill_layer, swap_layers};
use vile_core::{Board, Document, Keycode, LoadReport, Slot};

fn main() -> ExitCode {
    match run(args::Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: args::Cli) -> Result<()> {
    match cli.command {
        args::Command::Check { file } => check(&file),
        args::Command::Diff { old, new } => {
            let old_doc = load(&old)?;
            let new_doc = load(&new)?;
            let board = Board::for_document(&new_doc);
            let entries = diff(&old_doc, &new_doc, board.as_ref());
            if entries.is_empty() {
                println!("no changes");
            } else {
                print!("{}", format_diff(&entries));
            }
            Ok(())
        }
        args::Command::Layer { op } => run_layer(op),
    }
}

fn run_layer(op: args::LayerCommand) -> Result<()> {
    match op {
        args::LayerCommand::Clear {
            file,
            layer,
            transparent,
            out,
        } => {
            let before = load(&file)?;
            let mut after = before.clone();
            let kc = if transparent {
                Keycode::TRNS
            } else {
                Keycode::NO
            };
            let warnings = fill_layer(&mut after, layer, kc)?;
            finish(&file, &before, &after, &warnings, &out)
        }
        args::LayerCommand::Copy {
            file,
            src,
            dst,
            out,
        } => {
            let before = load(&file)?;
            let mut after = before.clone();
            let warnings = copy_layer(&mut after, src, dst)?;
            finish(&file, &before, &after, &warnings, &out)
        }
        args::LayerCommand::Swap { file, a, b, out } => {
            let before = load(&file)?;
            let mut after = before.clone();
            let warnings = swap_layers(&mut after, a, b)?;
            finish(&file, &before, &after, &warnings, &out)
        }
    }
}

/// Loads a file and prints its load warnings to stderr.
fn load(path: &Path) -> Result<Document> {
    let (doc, report) = read(path)?;
    for issue in &report.warnings {
        eprintln!(
            "warning: {}: {}: {}",
            path.display(),
            issue.path,
            issue.message
        );
    }
    Ok(doc)
}

fn read(path: &Path) -> Result<(Document, LoadReport)> {
    Document::load(path).map_err(|e| anyhow!("{}: {e}", path.display()))
}

fn check(file: &Path) -> Result<()> {
    let (doc, report) = read(file)?;
    let keys_per_layer = doc.layout.first().map_or(0, |layer| {
        layer
            .iter()
            .flatten()
            .filter(|slot| matches!(slot, Slot::Key(_)))
            .count()
    });
    println!(
        "ok: {} layers, {}x{} matrix, {} keys per layer",
        doc.layer_count(),
        doc.rows(),
        doc.cols(),
        keys_per_layer
    );
    println!("uid: {}", doc.uid);
    println!(
        "macros: {} slots, {} used",
        doc.macros.len(),
        doc.macros.iter().filter(|m| !m.actions.is_empty()).count()
    );
    section_line(
        "combos",
        doc.combo.as_ref().map(|list| {
            let used = list
                .iter()
                .filter(|c| c.keys.iter().any(|k| *k != Keycode::NO) || c.output != Keycode::NO)
                .count();
            (list.len(), used)
        }),
    );
    section_line(
        "tap dances",
        doc.tap_dance.as_ref().map(|list| {
            let used = list
                .iter()
                .filter(|t| {
                    [t.on_tap, t.on_hold, t.on_double_tap, t.on_tap_hold]
                        .iter()
                        .any(|k| *k != Keycode::NO)
                })
                .count();
            (list.len(), used)
        }),
    );
    section_line(
        "key overrides",
        doc.key_override.as_ref().map(|list| {
            let used = list
                .iter()
                .filter(|k| k.trigger != Keycode::NO || k.replacement != Keycode::NO)
                .count();
            (list.len(), used)
        }),
    );
    section_line(
        "alt repeat keys",
        doc.alt_repeat_key.as_ref().map(|list| {
            let used = list
                .iter()
                .filter(|a| a.keycode != Keycode::NO || a.alt_keycode != Keycode::NO)
                .count();
            (list.len(), used)
        }),
    );
    match Board::for_document(&doc) {
        Some(board) => println!("board: {}", board.name),
        None => println!("board: unknown (positions are shown as rRcC)"),
    }
    if report.canonical {
        println!("round trip: byte-identical");
    } else {
        println!("round trip: saving will normalize this file");
    }
    for issue in &report.warnings {
        println!("warning: {}: {}", issue.path, issue.message);
    }
    Ok(())
}

/// "{label}: {slots} slots, {used} used", or "{label}: absent".
fn section_line(label: &str, section: Option<(usize, usize)>) {
    match section {
        Some((slots, used)) => println!("{label}: {slots} slots, {used} used"),
        None => println!("{label}: absent"),
    }
}

/// Prints the staged changes, then writes them when a target is given.
fn finish(
    file: &Path,
    before: &Document,
    after: &Document,
    warnings: &[String],
    out: &args::OutputArgs,
) -> Result<()> {
    for warning in warnings {
        eprintln!("warning: {warning}");
    }
    let board = Board::for_document(after);
    let entries = diff(before, after, board.as_ref());
    if entries.is_empty() {
        println!("no changes");
        eprintln!("nothing written");
        return Ok(());
    }
    print!("{}", format_diff(&entries));

    let target: Option<PathBuf> = if out.write {
        Some(file.to_path_buf())
    } else {
        out.output.clone()
    };
    match target {
        Some(target) => {
            let bytes = after.to_vil_bytes();
            let loads_back = Document::from_vil_bytes(&bytes)
                .map(|(reloaded, _)| reloaded == *after)
                .unwrap_or(false);
            if !loads_back {
                bail!("refusing to write: the output does not load back identically (bug in vile)");
            }
            vile_core::io::write_atomic(&target, &bytes)
                .with_context(|| format!("cannot write {}", target.display()))?;
            eprintln!("wrote {}", target.display());
        }
        None => eprintln!("dry run: nothing written (use --write or --output PATH)"),
    }
    Ok(())
}
