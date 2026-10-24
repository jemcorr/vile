//! Pure Vial-definition/KLE geometry parsing. No device access.
use crate::board::KeyRect;
use crate::{Board, BoardKey, Document};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// KLE alignment maps serialized label positions into the twelve label slots.
const LABELS: [[i8; 12]; 8] = [
    [0, 6, 2, 8, 9, 11, 3, 5, 1, 4, 7, 10],
    [1, 7, -1, -1, 9, 11, 4, -1, -1, -1, -1, 10],
    [3, -1, 5, -1, 9, 11, -1, -1, 4, -1, -1, 10],
    [4, -1, -1, -1, 9, 11, -1, -1, -1, -1, -1, 10],
    [0, 6, 2, 8, 10, -1, 3, 5, 1, 4, 7, -1],
    [1, 7, -1, -1, 10, -1, 4, -1, -1, -1, -1, -1],
    [3, -1, 5, -1, 10, -1, -1, -1, 4, -1, -1, -1],
    [4, -1, -1, -1, 10, -1, -1, -1, -1, -1, -1, -1],
];

struct EncoderEntry {
    index: usize,
    direction: usize,
    alternative: Option<(usize, usize)>,
}

fn pair(text: &str) -> Result<(usize, usize), String> {
    let (a, b) = text
        .split_once(',')
        .ok_or("Expected a row,column or layout,option pair")?;
    Ok((
        a.trim().parse().map_err(|_| "Invalid pair index")?,
        b.trim().parse().map_err(|_| "Invalid pair index")?,
    ))
}
fn number(value: &Value, key: &str, default: f64) -> Result<f64, String> {
    let n = match value.get(key) {
        Some(v) => v.as_f64().ok_or_else(|| format!("Invalid KLE {key}"))?,
        None => default,
    };
    if !n.is_finite() || n.abs() > 10000.0 {
        return Err(format!("KLE {key} is outside supported bounds"));
    }
    Ok(n)
}
fn choices(definition: &Value, options: i64) -> Result<Vec<usize>, String> {
    let Some(labels) = definition
        .pointer("/layouts/labels")
        .and_then(Value::as_array)
    else {
        return Ok(Vec::new());
    };
    if options < 0 {
        return Err("Layout options are unavailable for this definition".into());
    }
    let mut value = options as u64;
    let mut selected = Vec::new();
    let mut total = 0;
    for label in labels.iter().rev() {
        let count = if label.is_string() {
            2
        } else {
            label
                .as_array()
                .filter(|v| v.len() >= 3)
                .ok_or("Invalid layout option labels")?
                .len()
                - 1
        };
        let bits = usize::BITS - (count - 1).leading_zeros();
        total += bits;
        if total > 32 {
            return Err("Layout options exceed 32 bits".into());
        }
        let choice = (value & ((1u64 << bits) - 1)) as usize;
        if choice >= count {
            return Err("The file selects an unavailable layout option".into());
        }
        selected.push(choice);
        value >>= bits;
    }
    selected.reverse();
    Ok(selected)
}

fn alternative(
    definition: &Value,
    selected: &[usize],
    label: &str,
) -> Result<Option<(usize, usize)>, String> {
    if label.is_empty() {
        return Ok(None);
    }
    let (group, option) = pair(label)?;
    if group >= selected.len() {
        return Err("KLE alternative references an unknown option group".into());
    }
    let label = definition
        .pointer(&format!("/layouts/labels/{group}"))
        .ok_or("KLE alternative references an unknown option group")?;
    let count = if label.is_string() {
        2
    } else {
        label
            .as_array()
            .filter(|choices| choices.len() >= 3)
            .ok_or("Invalid layout option labels")?
            .len()
            - 1
    };
    if option >= count {
        return Err("KLE alternative references an unknown option".into());
    }
    Ok(Some((group, option)))
}

fn encoder_legend(label: &str) -> Result<(usize, usize), String> {
    let (index, direction) = pair(label)?;
    if direction > 1 {
        return Err("KLE encoder direction must be 0 or 1".into());
    }
    Ok((index, direction))
}

fn encoder_capacity(doc: &Document) -> Result<usize, String> {
    let Some(layers) = &doc.encoder_layout else {
        return Err("Definition declares encoders but the file has no encoder_layout".into());
    };
    let Some(first) = layers.first() else {
        return Err("Definition declares encoders but encoder_layout has no layers".into());
    };
    if layers.len() != doc.layer_count() || layers.iter().any(|layer| layer.len() != first.len()) {
        return Err("Definition encoders do not match the file's encoder_layout".into());
    }
    Ok(first.len())
}
fn top_left(key: &BoardKey) -> (f64, f64) {
    let a = key.r.to_radians();
    let mut min = (f64::INFINITY, f64::INFINITY);
    let rects = std::iter::once(KeyRect {
        x: 0.0,
        y: 0.0,
        w: key.w,
        h: key.h,
    })
    .chain(key.secondary.clone());
    for rect in rects {
        for (dx, dy) in [(0.0, 0.0), (rect.w, 0.0), (0.0, rect.h), (rect.w, rect.h)] {
            let x = key.x + rect.x + dx - key.rx;
            let y = key.y + rect.y + dy - key.ry;
            min.0 = min.0.min(x * a.cos() - y * a.sin() + key.rx);
            min.1 = min.1.min(x * a.sin() + y * a.cos() + key.ry);
        }
    }
    min
}

pub fn from_vial_definition(definition: &Value, doc: &Document) -> Result<Board, String> {
    if definition.pointer("/matrix/rows").and_then(Value::as_u64) != Some(doc.rows() as u64)
        || definition.pointer("/matrix/cols").and_then(Value::as_u64) != Some(doc.cols() as u64)
    {
        return Err("Definition matrix dimensions do not match the file".into());
    }
    let rows = definition
        .pointer("/layouts/keymap")
        .and_then(Value::as_array)
        .ok_or("Definition has no KLE keymap")?;
    let selected = choices(definition, doc.layout_options)?;
    let mut parsed: Vec<(BoardKey, Option<(usize, usize)>)> = Vec::new();
    let mut encoders = Vec::new();
    let (mut x, mut y, mut rx, mut ry, mut r) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut align = 4usize;
    for row in rows {
        let Some(items) = row.as_array() else {
            if row.is_object() {
                continue;
            }
            return Err("Invalid KLE row".into());
        };
        let (mut w, mut h, mut x2, mut y2, mut w2, mut h2) = (1.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let mut decal = false;
        for (index, item) in items.iter().enumerate() {
            if item.is_object() {
                if index != 0 && ["r", "rx", "ry"].iter().any(|k| item.get(k).is_some()) {
                    return Err("KLE rotation must start a row".into());
                }
                if item.get("r").is_some() {
                    r = number(item, "r", r)?;
                }
                if item.get("rx").is_some() {
                    rx = number(item, "rx", rx)?;
                    x = rx;
                    y = ry;
                }
                if item.get("ry").is_some() {
                    ry = number(item, "ry", ry)?;
                    x = rx;
                    y = ry;
                }
                x += number(item, "x", 0.0)?;
                y += number(item, "y", 0.0)?;
                w = number(item, "w", w)?;
                h = number(item, "h", h)?;
                x2 = number(item, "x2", x2)?;
                y2 = number(item, "y2", y2)?;
                w2 = number(item, "w2", if item.get("w").is_some() { w } else { w2 })?;
                h2 = number(item, "h2", if item.get("h").is_some() { h } else { h2 })?;
                if let Some(a) = item.get("a") {
                    align = a
                        .as_u64()
                        .filter(|n| *n < 8)
                        .ok_or("Invalid KLE alignment")? as usize;
                }
                if let Some(d) = item.get("d") {
                    decal = d.as_bool().ok_or("Invalid KLE decal")?;
                }
                continue;
            }
            let label = item.as_str().ok_or("Invalid KLE key")?;
            let mut normalized = [""; 12];
            for (i, text) in label.split('\n').enumerate() {
                if i >= 12 {
                    return Err("Too many KLE labels".into());
                }
                let position = LABELS[align][i];
                if position >= 0 {
                    normalized[position as usize] = text;
                }
            }
            if w <= 0.0 || h <= 0.0 || w > 100.0 || h > 100.0 {
                return Err("Invalid KLE key dimensions".into());
            }
            if !decal && normalized[4] == "e" {
                let (encoder, direction) = encoder_legend(normalized[0])?;
                if encoder >= encoder_capacity(doc)? {
                    return Err("KLE encoder is outside encoder_layout".into());
                }
                encoders.push(EncoderEntry {
                    index: encoder,
                    direction,
                    alternative: alternative(definition, &selected, normalized[8])?,
                });
            } else if !decal && normalized[0].contains(',') {
                let (row, col) = pair(normalized[0])?;
                if row >= doc.rows() || col >= doc.cols() {
                    return Err("KLE key is outside the matrix".into());
                }
                let alternative = alternative(definition, &selected, normalized[8])?;
                let secondary =
                    if x2 != 0.0 || y2 != 0.0 || (w2 != 0.0 && w2 != w) || (h2 != 0.0 && h2 != h) {
                        Some(KeyRect {
                            x: x2,
                            y: y2,
                            w: if w2 == 0.0 { w } else { w2 },
                            h: if h2 == 0.0 { h } else { h2 },
                        })
                    } else {
                        None
                    };
                if secondary
                    .as_ref()
                    .is_some_and(|s| s.w <= 0.0 || s.h <= 0.0 || s.w > 100.0 || s.h > 100.0)
                {
                    return Err("Invalid secondary key rectangle".into());
                }
                parsed.push((
                    BoardKey {
                        id: format!("r{row}c{col}"),
                        row,
                        col,
                        x,
                        y,
                        w,
                        h,
                        r,
                        rx,
                        ry,
                        secondary,
                        name: None,
                    },
                    alternative,
                ));
            }
            x += w;
            w = 1.0;
            h = 1.0;
            x2 = 0.0;
            y2 = 0.0;
            w2 = 0.0;
            h2 = 0.0;
            decal = false;
        }
        y += 1.0;
        x = rx;
    }
    if parsed.len() > 4096 {
        return Err("Definition has too many keys".into());
    }
    let mut origins: BTreeMap<(usize, usize), (f64, f64)> = BTreeMap::new();
    for (key, alternative) in &parsed {
        if let Some(group) = alternative {
            let p = top_left(key);
            origins
                .entry(*group)
                .and_modify(|q| {
                    q.0 = q.0.min(p.0);
                    q.1 = q.1.min(p.1);
                })
                .or_insert(p);
        }
    }
    let mut board = Board::matrix(doc);
    board.keys.clear();
    board.id = format!("vial-{}", doc.uid);
    board.name = definition
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Vial keyboard")
        .into();
    board.source = "Vial definition".into();
    let mut encoder_indices = BTreeSet::new();
    let mut encoder_actions = BTreeSet::new();
    for encoder in encoders {
        if encoder
            .alternative
            .is_some_and(|(group, option)| selected[group] != option)
        {
            continue;
        }
        if !encoder_actions.insert((encoder.index, encoder.direction)) {
            return Err("Definition repeats an active encoder direction".into());
        }
        encoder_indices.insert(encoder.index);
    }
    board.encoder_indices = Some(encoder_indices.into_iter().collect());
    let mut positions = BTreeSet::new();
    for (mut key, alternative) in parsed {
        if let Some((group, option)) = alternative {
            if selected[group] != option {
                continue;
            }
            let base = origins
                .get(&(group, 0))
                .ok_or("KLE option has no default position")?;
            let origin = origins[&(group, option)];
            let dx = base.0 - origin.0;
            let dy = base.1 - origin.1;
            key.x += dx;
            key.y += dy;
            key.rx += dx;
            key.ry += dy;
        }
        if !positions.insert((key.row, key.col)) {
            return Err("Definition repeats an active matrix position".into());
        }
        if !matches!(doc.get(0, key.row, key.col), Some(crate::Slot::Key(_))) {
            return Err("Definition references a hole in the file's matrix".into());
        }
        if let Some(name) = definition
            .pointer("/vile/key_names")
            .and_then(|v| v.get(&key.id))
            .and_then(Value::as_str)
        {
            if name.is_empty() || name.len() > 100 || name.chars().any(char::is_control) {
                return Err("Invalid semantic key name".into());
            }
            key.name = Some(name.into());
        }
        board.keys.push(key);
    }
    if board.keys.is_empty() {
        return Err("Definition contains no active matrix keys".into());
    }
    // Vial saves the union of matrix positions across alternatives. Keep inactive slots editable below the physical board.
    let bottom = board
        .keys
        .iter()
        .map(|k| {
            let extra = k
                .secondary
                .as_ref()
                .map_or(0.0, |s| s.x.abs() + s.y.abs() + s.w + s.h);
            top_left(k).1 + k.h + k.w + extra
        })
        .fold(0.0, f64::max)
        + 1.0;
    for mut key in Board::matrix(doc).keys {
        if !positions.contains(&(key.row, key.col)) {
            key.y += bottom;
            board.keys.push(key);
        }
    }
    let mut semantic_names = BTreeSet::new();
    for key in &board.keys {
        if let Some(name) = &key.name
            && (!semantic_names.insert(name)
                || board
                    .keys
                    .iter()
                    .any(|other| other.id == *name && other.id != key.id))
        {
            return Err("Semantic names must be unique and cannot shadow another matrix ID".into());
        }
    }
    if let Some(pairs) = definition.pointer("/vile/mirror_pairs") {
        let pairs: Vec<[String; 2]> =
            serde_json::from_value(pairs.clone()).map_err(|_| "Invalid explicit mirror pairs")?;
        let mut used = BTreeSet::new();
        for pair in &pairs {
            if pair[0] == pair[1]
                || pair.iter().any(|id| {
                    !board.keys.iter().any(|key| key.id == *id) || !used.insert(id.clone())
                })
            {
                return Err(
                    "Mirror pairs must reference distinct existing keys exactly once".into(),
                );
            }
        }
        board.mirror_pairs = pairs;
    }
    Ok(board)
}
