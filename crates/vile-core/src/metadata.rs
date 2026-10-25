//! Per-layout user names. Identity includes the adjacent file, UID and content.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Names {
    pub layers: BTreeMap<usize, String>,
    pub macros: BTreeMap<usize, String>,
    pub combos: BTreeMap<usize, String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sidecar {
    pub version: u32,
    pub uid: String,
    pub fingerprint: String,
    pub names: Names,
}
pub fn fingerprint(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut fingerprint = String::with_capacity(digest.len() * 2);
    for byte in digest {
        fingerprint.push(char::from(HEX[(byte >> 4) as usize]));
        fingerprint.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    fingerprint
}

pub fn sidecar_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".vile.json");
    PathBuf::from(name)
}
impl Names {
    pub fn validate(&self, doc: &crate::Document) -> Result<(), String> {
        for (map, count) in [
            (&self.layers, doc.layer_count()),
            (&self.macros, doc.macros.len()),
            (&self.combos, doc.combo.as_ref().map_or(0, Vec::len)),
        ] {
            if map.iter().any(|(&index, name)| {
                index >= count || name.chars().count() > 100 || name.chars().any(char::is_control)
            }) {
                return Err("Names contain an invalid item index or text".into());
            }
        }
        Ok(())
    }

    pub fn remap_layers(&mut self, permutation: &[usize]) {
        self.layers = self
            .layers
            .iter()
            .map(|(&old, name)| (permutation.get(old).copied().unwrap_or(old), name.clone()))
            .collect();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fingerprint_keeps_lowercase_sha256_hex_format() {
        assert_eq!(
            super::fingerprint(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
