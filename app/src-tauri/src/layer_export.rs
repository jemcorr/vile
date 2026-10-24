use std::path::Path;

const MAX_PIXELS: u64 = 16_777_216;

/// Reject empty, oversized or non-PNG IPC payloads before opening a save dialog.
pub fn validate_png(png: &[u8]) -> Result<(), String> {
    if png.len() < 45
        || png.len() > 64 * 1024 * 1024
        || !png.starts_with(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR")
        || !png.ends_with(b"\0\0\0\0IEND\xaeB`\x82")
    {
        return Err("The export did not produce a PNG image".into());
    }
    let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
    if width == 0
        || height == 0
        || width > 16384
        || height > 16384
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err("The exported PNG exceeds the image size limit".into());
    }
    Ok(())
}

pub fn write_png(path: &Path, source: Option<&Path>, png: &[u8]) -> Result<(), String> {
    validate_png(png)?;
    if !path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
    {
        return Err("Choose a filename ending in .png".into());
    }
    if let Some(source) = source
        && (path == source
            || path
                .canonicalize()
                .ok()
                .zip(source.canonicalize().ok())
                .is_some_and(|(target, layout)| target == layout))
    {
        return Err("Choose a PNG path other than the open layout file".into());
    }
    vile_core::io::write_atomic(path, png)
        .map_err(|error| format!("Could not export {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = include_bytes!("../icons/32x32.png");

    #[test]
    fn rejects_invalid_and_oversized_images() {
        assert!(validate_png(PNG).is_ok());
        assert!(validate_png(b"not a PNG").is_err());
        assert!(validate_png(&PNG[..PNG.len() - 12]).is_err());
        let mut oversized = PNG.to_vec();
        oversized[16..20].copy_from_slice(&16384u32.to_be_bytes());
        oversized[20..24].copy_from_slice(&16384u32.to_be_bytes());
        assert!(validate_png(&oversized).is_err());
    }

    #[test]
    fn writes_png_without_replacing_the_layout_or_damaging_files_on_failure() {
        let directory = std::env::temp_dir().join(format!(
            "vile-export-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let layout = directory.join("layout.vil");
        let target = directory.join("layers.png");
        std::fs::write(&layout, "layout contents").unwrap();
        std::fs::write(&target, "old image").unwrap();
        assert!(write_png(&layout, Some(&layout), PNG).is_err());
        assert!(write_png(&target, Some(&layout), b"invalid").is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"old image");
        write_png(&target, Some(&layout), PNG).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), PNG);
        assert_eq!(std::fs::read_to_string(&layout).unwrap(), "layout contents");
        assert!(write_png(&target, Some(&target), PNG).is_err());
        assert!(write_png(&directory.join("missing/file.png"), Some(&layout), PNG).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
