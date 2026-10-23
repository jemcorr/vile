//! The vile command-line tool.

mod args;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, Result, anyhow, bail};
use clap::Parser;
use vile_core::diff::{diff, format_diff};
use vile_core::ops::{copy_layer, fill_layer, mirror_layer, move_layer, set_keys, swap_layers};
use vile_core::validate::macro_buffer_bytes;
use vile_core::{
    Board, BoardKey, Catalog, CatalogContext, Document, Keycode, LoadReport, MirrorMode, OpError,
    Pos, Slot,
};

/// Board rows in the order `show` prints them.
const BOARD_ROWS: [&str; 4] = ["top", "home", "bottom", "thumb"];

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
        args::Command::Diff { old, new, git } => {
            if git {
                diff_against_git(&old)
            } else {
                let new = new.ok_or_else(|| anyhow!("NEW is required unless --git is given"))?;
                let old_doc = load(&old)?;
                let new_doc = load(&new)?;
                print_diff(&old_doc, &new_doc)
            }
        }
        args::Command::Show { file, layer } => show(&file, layer),
        args::Command::Keycodes { query, limit, file } => {
            keycodes(query.as_deref(), limit, file.as_deref())
        }
        args::Command::Set {
            file,
            layer,
            key,
            keycode,
            out,
        } => set_key(&file, layer, &key, &keycode, &out),
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
        args::LayerCommand::Move {
            file,
            from,
            to,
            out,
        } => {
            let before = load(&file)?;
            let mut after = before.clone();
            let warnings = move_layer(&mut after, from, to)?;
            finish(&file, &before, &after, &warnings, &out)
        }
        args::LayerCommand::Mirror {
            file,
            layer,
            mode,
            out,
        } => {
            let before = load(&file)?;
            let board = Board::for_document(&before)
                .ok_or_else(|| anyhow!("unknown board; mirroring needs the key geometry"))?;
            let mode = match mode {
                args::MirrorArg::Swap => MirrorMode::Swap,
                args::MirrorArg::LeftToRight => MirrorMode::LeftToRight,
                args::MirrorArg::RightToLeft => MirrorMode::RightToLeft,
            };
            let mut after = before.clone();
            let warnings = mirror_layer(&mut after, &board, layer, mode)?;
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
    println!("macro buffer: {} bytes", macro_buffer_bytes(&doc));
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

fn print_diff(old: &Document, new: &Document) -> Result<()> {
    let board = Board::for_document(new);
    let entries = diff(old, new, board.as_ref());
    if entries.is_empty() {
        println!("no changes");
    } else {
        print!("{}", format_diff(&entries));
    }
    Ok(())
}

/// Diff from the committed (git HEAD) version of `path` to the file as it is now.
fn diff_against_git(path: &Path) -> Result<()> {
    let bytes = vile_core::git::read_head_version(path).map_err(|e| {
        anyhow!(
            "cannot read the git HEAD version of {}: {e}",
            path.display()
        )
    })?;
    let (head, _report) = Document::from_vil_bytes(&bytes).map_err(|e| {
        anyhow!(
            "the git HEAD version of {} does not load: {e}",
            path.display()
        )
    })?;
    let current = load(path)?;
    print_diff(&head, &current)
}

/// Matrix position from "r{row}c{col}" with decimal digits.
fn matrix_position(key: &str) -> Option<Pos> {
    let rest = key.strip_prefix('r')?;
    let (row, col) = rest.split_once('c')?;
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(row) || !digits(col) {
        return None;
    }
    Some(Pos {
        row: row.parse().ok()?,
        col: col.parse().ok()?,
    })
}

fn set_key(
    file: &Path,
    layer: usize,
    key: &str,
    keycode: &str,
    out: &args::OutputArgs,
) -> Result<()> {
    let before = load(file)?;
    let pos = match matrix_position(key) {
        Some(pos) => pos,
        None => {
            let board = Board::for_document(&before)
                .ok_or_else(|| anyhow!("unknown board; use rRcC positions"))?;
            let found = board
                .key_by_id(key)
                .ok_or_else(|| anyhow!("unknown key id \"{key}\""))?;
            Pos {
                row: found.row,
                col: found.col,
            }
        }
    };
    let kc = before.catalog().parse_input(keycode)?;
    let mut after = before.clone();
    let warnings = set_keys(&mut after, layer, &[pos], kc)?;
    finish(file, &before, &after, &warnings, out)
}

/// One cell of `show`: the label with '/' for line breaks, cut to 7 characters, padded to 7.
fn cell_text(doc: &Document, catalog: &Catalog, layer: usize, row: usize, col: usize) -> String {
    let label = match doc.get(layer, row, col) {
        Some(Slot::Key(kc)) => catalog.label(kc),
        _ => "-".to_string(),
    };
    let short: String = label.replace('\n', "/").chars().take(7).collect();
    format!("{short:<7}")
}

fn show(file: &Path, only: Option<usize>) -> Result<()> {
    let doc = load(file)?;
    let board = Board::for_document(&doc);
    let catalog = doc.catalog();
    let layers: Vec<usize> = match only {
        Some(layer) if layer < doc.layer_count() => vec![layer],
        Some(layer) => {
            return Err(OpError::NoSuchLayer {
                layer,
                count: doc.layer_count(),
            }
            .into());
        }
        None => (0..doc.layer_count()).collect(),
    };
    for layer in layers {
        println!("L{layer}");
        match &board {
            Some(board) => {
                for row in BOARD_ROWS {
                    let cells_of = |side: &str| -> Vec<String> {
                        let prefix = format!("{row}-{side}-");
                        let mut keys: Vec<&BoardKey> = board
                            .keys
                            .iter()
                            .filter(|key| key.id.starts_with(&prefix))
                            .collect();
                        keys.sort_by(|a, b| a.x.total_cmp(&b.x));
                        keys.iter()
                            .map(|key| cell_text(&doc, &catalog, layer, key.row, key.col))
                            .collect()
                    };
                    let line = format!(
                        "{}   {}",
                        cells_of("left").join(" "),
                        cells_of("right").join(" ")
                    );
                    println!("{}", line.trim_end());
                }
            }
            None => {
                for row in 0..doc.rows() {
                    let cells: Vec<String> = (0..doc.cols())
                        .map(|col| cell_text(&doc, &catalog, layer, row, col))
                        .collect();
                    println!("{}", cells.join(" ").trim_end());
                }
            }
        }
        println!();
    }
    Ok(())
}

fn keycodes(query: Option<&str>, limit: usize, file: Option<&Path>) -> Result<()> {
    let catalog = match file {
        Some(path) => load(path)?.catalog(),
        None => Catalog::new(CatalogContext::default()),
    };
    let limit = if limit == 0 { usize::MAX } else { limit };
    for hit in catalog.search(query.unwrap_or(""), limit) {
        println!(
            "{}\t{}\t{}",
            hit.name,
            hit.label.replace('\n', "/"),
            hit.group
        );
    }
    Ok(())
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
