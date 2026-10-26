//! Fuzzy keycode search for the CLI and the inline keycode picker.

use super::{Catalog, Keycode, LayerOp, QK_KB, QK_MACRO, QK_TAP_DANCE, table};

/// One search result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchHit {
    /// Canonical wire name ("KC_ESCAPE"), or a template such as "LSFT(kc)".
    pub name: String,
    /// Text to put into the input box: the wire name, or "LSFT(" for a template.
    pub insert: String,
    /// True when `name` is a complete keycode; false for templates.
    pub complete: bool,
    /// Key cap label ("" for templates).
    pub label: String,
    /// Group name: a TSV group, or "layers", "macro", "tap_dance", "user", "layer_tap".
    pub group: &'static str,
    /// The name or alias that matched best, as spelled in the table.
    pub matched: String,
    /// Higher is better. See PLAN.md step 2.6.
    pub score: i32,
}

/// A name the search can return, with the keys it matches on.
struct Candidate {
    name: String,
    insert: String,
    complete: bool,
    label: String,
    group: &'static str,
    keys: Vec<String>,
    /// Descriptions and tooltips: matched whole or as a substring, never letter by letter.
    prose: Vec<String>,
}

/// Uppercase ASCII letters and digits only.
fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// A match key as compared: itself, and without a leading KC_ or QK_.
fn variants(key: &str) -> Vec<String> {
    let mut out = vec![norm(key)];
    if let Some(rest) = key.strip_prefix("KC_") {
        out.push(norm(rest));
    }
    if let Some(rest) = key.strip_prefix("QK_") {
        out.push(norm(rest));
    }
    out
}

fn is_subsequence(q: &str, x: &str) -> bool {
    let mut wanted = q.chars().peekable();
    for c in x.chars() {
        if wanted.peek() == Some(&c) {
            wanted.next();
        }
    }
    wanted.peek().is_none()
}

/// The score of one normalized string against the normalized query, and
/// whether it matched only letter by letter.
fn score_of(x: &str, q: &str, fuzzy: bool) -> Option<(i32, bool)> {
    let extra = x.len() as i32 - q.len() as i32;
    if x == q {
        Some((1000, false))
    } else if x.starts_with(q) {
        Some((800 - extra, false))
    } else if let Some(i) = x.find(q) {
        Some((600 - i as i32, false))
    } else if fuzzy && is_subsequence(q, x) {
        Some((400 - extra, true))
    } else {
        None
    }
}

impl Catalog {
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let candidates = self.candidates();
        let q = norm(query);
        if q.is_empty() {
            return candidates
                .into_iter()
                .take(limit)
                .map(|c| SearchHit {
                    matched: c.name.clone(),
                    name: c.name,
                    insert: c.insert,
                    complete: c.complete,
                    label: c.label,
                    group: c.group,
                    score: 0,
                })
                .collect();
        }
        let mut scored: Vec<(i32, usize, SearchHit)> = Vec::new();
        let mut fuzzy_only = Vec::new();
        for (index, c) in candidates.into_iter().enumerate() {
            let mut best: Option<(i32, bool, &str)> = None;
            let names = c.keys.iter().map(|key| (key, true));
            let prose = c.prose.iter().map(|text| (text, false));
            for (key, fuzzy) in names.chain(prose) {
                for x in variants(key) {
                    if let Some((score, letters)) = score_of(&x, &q, fuzzy)
                        && best.is_none_or(|(b, _, _)| score > b)
                    {
                        best = Some((score, letters, key.as_str()));
                    }
                }
            }
            if let Some((score, letters, key)) = best {
                fuzzy_only.push(letters);
                scored.push((
                    score,
                    index,
                    SearchHit {
                        matched: key.to_string(),
                        name: c.name,
                        insert: c.insert,
                        complete: c.complete,
                        label: c.label,
                        group: c.group,
                        score,
                    },
                ));
            }
        }
        // Letter-by-letter matches are a fallback for typos: drop them once anything matches properly.
        if fuzzy_only.iter().any(|letters| !letters) {
            let mut letters = fuzzy_only.into_iter();
            scored.retain(|_| !letters.next().unwrap_or(false));
        }
        scored.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then(a.2.name.len().cmp(&b.2.name.len()))
                .then(a.1.cmp(&b.1))
        });
        scored
            .into_iter()
            .take(limit)
            .map(|(_, _, hit)| hit)
            .collect()
    }

    /// Candidates in the order of step 2.5: plain keycodes, generated layer,
    /// macro, tap dance and user names, then templates.
    fn candidates(&self) -> Vec<Candidate> {
        let ctx = self.context();
        let mut out = Vec::new();
        for entry in table().iter().filter(|e| !e.group.is_function()) {
            let name = self.wire_name(Keycode(entry.code));
            if name.starts_with("0x") {
                continue;
            }
            let mut keys = vec![name.clone(), entry.name.to_string()];
            keys.extend(entry.aliases.iter().map(|alias| alias.to_string()));
            let picker = self.picker(Keycode(entry.code));
            keys.push(picker.label);
            let prose = vec![
                self.description(Keycode(entry.code)),
                self.tooltip(Keycode(entry.code)),
                picker.summary,
            ];
            out.push(Candidate {
                insert: name.clone(),
                label: self.label(Keycode(entry.code)),
                name,
                complete: true,
                group: entry.group.as_str(),
                keys,
                prose,
            });
        }
        let generated = |name: String, code: u16, group: &'static str| Candidate {
            insert: name.clone(),
            label: self.label(Keycode(code)),
            keys: vec![name.clone()],
            prose: Vec::new(),
            name,
            complete: true,
            group,
        };
        for op in LayerOp::ALL {
            for n in 0..u16::from(ctx.layers.min(32)) {
                out.push(generated(
                    format!("{}({n})", op.name()),
                    op.base() + n,
                    "layers",
                ));
            }
        }
        for n in 0..ctx.macros.min(256) {
            out.push(generated(format!("M{n}"), QK_MACRO + n, "macro"));
        }
        for n in 0..ctx.tap_dances.min(256) {
            out.push(generated(format!("TD({n})"), QK_TAP_DANCE + n, "tap_dance"));
        }
        for n in 0..16u16 {
            out.push(generated(format!("USER{n:02}"), QK_KB + n, "user"));
        }
        for entry in table()
            .iter()
            .filter(|e| matches!(e.group, super::Group::Mods | super::Group::ModTap))
        {
            let name = format!("{}(kc)", entry.name);
            let mut keys = vec![name.clone(), entry.name.to_string()];
            keys.extend(entry.aliases.iter().map(|alias| alias.to_string()));
            out.push(Candidate {
                insert: format!("{}(", entry.name),
                label: String::new(),
                name,
                complete: false,
                group: entry.group.as_str(),
                keys,
                prose: Vec::new(),
            });
        }
        for n in 0..u16::from(ctx.layers.min(16)) {
            let name = format!("LT{n}(kc)");
            out.push(Candidate {
                insert: format!("LT{n}("),
                label: String::new(),
                keys: vec![name.clone()],
                prose: Vec::new(),
                name,
                complete: false,
                group: "layer_tap",
            });
        }
        out
    }
}
