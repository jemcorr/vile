//! Narrow read-only Vial protocol. HID output reports request information;
//! this crate exposes no keyboard-state mutation, unlock or bootloader command.
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::Path,
    sync::{OnceLock, mpsc},
    time::{Duration, Instant},
};
use vile_core::metadata::fingerprint;
use vile_core::{Board, Document};

pub const MAX_COMPRESSED: usize = 256 * 1024;
pub const MAX_DEFINITION: usize = 4 * 1024 * 1024;
const MAX_CACHE: u64 = (MAX_DEFINITION * 2) as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadRequest {
    Identify,
    DefinitionSize,
    DefinitionChunk(u32),
    MacroCapacity,
}
impl ReadRequest {
    fn packet(self) -> [u8; 33] {
        let mut packet = [0; 33];
        match self {
            Self::Identify => {
                packet[1] = 0xfe;
                packet[2] = 0;
            }
            Self::DefinitionSize => {
                packet[1] = 0xfe;
                packet[2] = 1;
            }
            Self::DefinitionChunk(block) => {
                packet[1] = 0xfe;
                packet[2] = 2;
                packet[3..7].copy_from_slice(&block.to_le_bytes());
            }
            Self::MacroCapacity => {
                packet[1] = 0x0d;
            }
        }
        packet
    }
}
pub trait ReadTransport {
    fn exchange(&mut self, request: ReadRequest) -> Result<[u8; 32], String>;
}
struct HidTransport(hidapi::HidDevice);
impl ReadTransport for HidTransport {
    fn exchange(&mut self, request: ReadRequest) -> Result<[u8; 32], String> {
        let packet = request.packet();
        let written = self.0.write(&packet).map_err(|e| e.to_string())?;
        if written != packet.len() {
            return Err("Incomplete HID read-request packet".into());
        }
        let mut response = [0; 32];
        let count = self
            .0
            .read_timeout(&mut response, 500)
            .map_err(|e| e.to_string())?;
        if count != response.len() {
            return Err(format!("Expected 32 response bytes, received {count}"));
        }
        Ok(response)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefinitionCache {
    pub version: u32,
    pub uid: String,
    pub protocol: u32,
    pub digest: String,
    pub definition: serde_json::Value,
    pub macro_capacity: Option<usize>,
}
impl DefinitionCache {
    pub fn board(&self, doc: &Document) -> Result<Board, String> {
        if self.version != 1
            || self.protocol != 6
            || doc.vial_protocol != 6
            || self.uid != doc.uid.to_string()
        {
            return Err("Definition cache UID or protocol does not match the layout".into());
        }
        let bytes = serde_json::to_vec(&self.definition).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_DEFINITION || fingerprint(&bytes) != self.digest {
            return Err("Definition cache has an invalid digest or size".into());
        }
        vile_core::geometry::from_vial_definition(&self.definition, doc)
    }
    pub fn save(&self, directory: &Path, doc: &Document) -> Result<(), String> {
        self.board(doc)?;
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        vile_core::io::write_atomic(&directory.join(format!("{}.json", doc.uid)), &bytes)
            .map_err(|e| e.to_string())
    }
    pub fn load(directory: &Path, doc: &Document) -> Result<Option<Self>, String> {
        let file = match std::fs::File::open(directory.join(format!("{}.json", doc.uid))) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        let mut bytes = Vec::new();
        file.take(MAX_CACHE + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_CACHE {
            return Err("Definition cache exceeds size limit".into());
        }
        let cache: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        cache.board(doc)?;
        Ok(Some(cache))
    }
}
fn identity(response: [u8; 32], uid: u64) -> Result<(), String> {
    let protocol = u32::from_le_bytes(response[0..4].try_into().unwrap());
    let actual = u64::from_le_bytes(response[4..12].try_into().unwrap());
    if protocol != 6 {
        return Err(format!("Unsupported Vial protocol {protocol}; expected 6"));
    }
    if actual != uid {
        return Err("Connected keyboard UID does not match the file".into());
    }
    Ok(())
}
fn decompress(bytes: &[u8]) -> Result<serde_json::Value, String> {
    let stream =
        xz2::stream::Stream::new_stream_decoder(64 * 1024 * 1024, 0).map_err(|e| e.to_string())?;
    let mut decoder = xz2::read::XzDecoder::new_stream(bytes, stream);
    let mut data = Vec::new();
    (&mut decoder)
        .take(MAX_DEFINITION as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|e| format!("Invalid compressed definition: {e}"))?;
    if data.len() > MAX_DEFINITION {
        return Err("Decompressed definition exceeds size limit".into());
    }
    if decoder.total_in() != bytes.len() as u64 {
        return Err("Unexpected trailing data in compressed definition".into());
    }
    serde_json::from_slice(&data).map_err(|e| format!("Invalid definition JSON: {e}"))
}
/// Reads information only after validating identity. The deadline bounds multi-packet transfers.
pub fn fetch(
    transport: &mut impl ReadTransport,
    doc: &Document,
) -> Result<DefinitionCache, String> {
    if doc.vial_protocol != 6 {
        return Err("Only Vial protocol 6 definitions are supported".into());
    }
    let deadline = Instant::now() + Duration::from_secs(15);
    identity(transport.exchange(ReadRequest::Identify)?, doc.uid)?;
    let response = transport.exchange(ReadRequest::DefinitionSize)?;
    let size = u32::from_le_bytes(response[..4].try_into().unwrap()) as usize;
    if size == 0 || size > MAX_COMPRESSED {
        return Err("Compressed definition size is outside supported bounds".into());
    }
    let mut bytes = Vec::with_capacity(size);
    for block in 0..size.div_ceil(32) {
        if Instant::now() > deadline {
            return Err("Definition read exceeded its time limit".into());
        }
        let response = transport.exchange(ReadRequest::DefinitionChunk(block as u32))?;
        bytes.extend_from_slice(&response[..(size - bytes.len()).min(32)]);
    }
    let definition = decompress(&bytes)?;
    identity(transport.exchange(ReadRequest::Identify)?, doc.uid)?;
    let macro_capacity = match transport.exchange(ReadRequest::MacroCapacity) {
        Ok(response) if response[0] == 0x0d => {
            Some(u16::from_be_bytes([response[1], response[2]]) as usize)
        }
        _ => None,
    };
    let digest = fingerprint(&serde_json::to_vec(&definition).map_err(|e| e.to_string())?);
    let cache = DefinitionCache {
        version: 1,
        uid: doc.uid.to_string(),
        protocol: 6,
        digest,
        definition,
        macro_capacity,
    };
    cache.board(doc)?;
    Ok(cache)
}
struct ReadJob {
    doc: Document,
    reply: mpsc::SyncSender<Result<DefinitionCache, String>>,
}

/// Enumerate only Vial raw-HID interfaces. No keyboard configuration is written.
pub fn read_matching_definition(doc: &Document) -> Result<DefinitionCache, String> {
    // The macOS HID manager keeps the run loop of its initialization thread.
    // Tokio's temporary blocking workers can exit, leaving that manager with a
    // dead run loop. Keep initialization, enumeration and reads on one living
    // thread, and serialize requests so refreshes cannot race each other.
    static WORKER: OnceLock<Result<mpsc::Sender<ReadJob>, String>> = OnceLock::new();
    let worker = WORKER.get_or_init(|| {
        let (sender, receiver) = mpsc::channel::<ReadJob>();
        std::thread::Builder::new()
            .name("vile-device-reads".into())
            .spawn(move || {
                for job in receiver {
                    let result = read_on_device_thread(&job.doc);
                    let _ = job.reply.send(result);
                }
            })
            .map_err(|e| format!("Cannot start device read thread: {e}"))?;
        Ok(sender)
    });
    let worker = worker.as_ref().map_err(Clone::clone)?;
    let (reply, result) = mpsc::sync_channel(1);
    worker
        .send(ReadJob {
            doc: doc.clone(),
            reply,
        })
        .map_err(|_| "Device read thread stopped".to_string())?;
    result
        .recv()
        .map_err(|_| "Device read thread stopped".to_string())?
}

fn read_on_device_thread(doc: &Document) -> Result<DefinitionCache, String> {
    let api = hidapi::HidApi::new().map_err(|e| e.to_string())?;
    let mut last = "No matching Vial keyboard is connected".to_string();
    let mut found = None;
    for info in api
        .device_list()
        .filter(|info| {
            info.usage_page() == 0xff60
                && info.usage() == 0x61
                && info
                    .serial_number()
                    .is_some_and(|s| s.contains("vial:f64c2b3c"))
        })
        .take(8)
    {
        let device = match info.open_device(&api) {
            Ok(device) => device,
            Err(e) => {
                last = e.to_string();
                continue;
            }
        };
        let mut transport = HidTransport(device);
        // Identify before fetching the definition so other keyboards are never queried for their configuration.
        match transport
            .exchange(ReadRequest::Identify)
            .and_then(|data| identity(data, doc.uid))
        {
            Ok(()) => {
                if found.is_some() {
                    return Err(
                        "Multiple keyboards have this UID; connect only the target keyboard".into(),
                    );
                }
                found = Some(transport);
            }
            Err(e) => last = e,
        }
    }
    let mut transport = found.ok_or(last)?;
    fetch(&mut transport, doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    struct Mock {
        uid: u64,
        compressed: Vec<u8>,
        requests: Vec<ReadRequest>,
        disconnected: bool,
    }
    impl ReadTransport for Mock {
        fn exchange(&mut self, request: ReadRequest) -> Result<[u8; 32], String> {
            self.requests.push(request);
            if self.disconnected {
                return Err("disconnected".into());
            }
            let mut response = [0; 32];
            match request {
                ReadRequest::Identify => {
                    response[..4].copy_from_slice(&6u32.to_le_bytes());
                    response[4..12].copy_from_slice(&self.uid.to_le_bytes());
                }
                ReadRequest::DefinitionSize => {
                    response[..4].copy_from_slice(&(self.compressed.len() as u32).to_le_bytes())
                }
                ReadRequest::DefinitionChunk(i) => {
                    let start = i as usize * 32;
                    let end = (start + 32).min(self.compressed.len());
                    response[..end - start].copy_from_slice(&self.compressed[start..end]);
                }
                ReadRequest::MacroCapacity => {
                    response[0] = 0x0d;
                    response[1..3].copy_from_slice(&1024u16.to_be_bytes());
                }
            }
            Ok(response)
        }
    }
    fn doc() -> Document {
        Document::from_vil_bytes(
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/fixtures/corne.vil"
            ))
            .unwrap(),
        )
        .unwrap()
        .0
    }
    fn mock(doc: &Document) -> Mock {
        let rows: Vec<_> = (0..doc.rows())
            .map(|r| {
                (0..doc.cols())
                    .map(|c| serde_json::json!(format!("{r},{c}")))
                    .collect::<Vec<_>>()
            })
            .collect();
        // Only positions actually present in the file are part of the KLE definition.
        let rows: Vec<_> = rows
            .into_iter()
            .enumerate()
            .map(|(r, row)| {
                row.into_iter()
                    .enumerate()
                    .filter_map(|(c, v)| {
                        matches!(doc.get(0, r, c), Some(vile_core::Slot::Key(_))).then_some(v)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let value = serde_json::json!({"matrix":{"rows":doc.rows(),"cols":doc.cols()},"layouts":{"keymap":rows}});
        let mut encoder = xz2::write::XzEncoder::new(Vec::new(), 6);
        encoder
            .write_all(&serde_json::to_vec(&value).unwrap())
            .unwrap();
        Mock {
            uid: doc.uid,
            compressed: encoder.finish().unwrap(),
            requests: Vec::new(),
            disconnected: false,
        }
    }
    #[test]
    fn requests_are_only_allowlisted_reads() {
        for request in [
            ReadRequest::Identify,
            ReadRequest::DefinitionSize,
            ReadRequest::DefinitionChunk(9),
            ReadRequest::MacroCapacity,
        ] {
            let p = request.packet();
            assert_eq!(p[0], 0);
            assert!(p[1] == 0x0d || p[1] == 0xfe && p[2] <= 2);
            assert!(p[7..].iter().all(|b| *b == 0));
        }
    }
    #[test]
    fn fetch_and_offline_cache_round_trip() {
        let doc = doc();
        let mut mock = mock(&doc);
        let cache = fetch(&mut mock, &doc).unwrap();
        assert_eq!(cache.macro_capacity, Some(1024));
        let dir = tempfile::tempdir().unwrap();
        cache.save(dir.path(), &doc).unwrap();
        assert_eq!(
            DefinitionCache::load(dir.path(), &doc)
                .unwrap()
                .unwrap()
                .board(&doc)
                .unwrap()
                .keys
                .len(),
            46
        );
    }
    #[test]
    fn wrong_identity_stops_before_definition_reads() {
        let doc = doc();
        let mut mock = mock(&doc);
        mock.uid += 1;
        assert!(fetch(&mut mock, &doc).is_err());
        assert_eq!(mock.requests, vec![ReadRequest::Identify]);
    }
    #[test]
    fn corrupt_cache_and_disconnect_fail_without_mutation() {
        let doc = doc();
        let mut mock = mock(&doc);
        mock.disconnected = true;
        assert!(fetch(&mut mock, &doc).is_err());
        mock.disconnected = false;
        let mut cache = fetch(&mut mock, &doc).unwrap();
        cache.digest = "corrupt".into();
        assert!(cache.board(&doc).is_err());
    }
    #[test]
    fn compressed_and_expanded_limits_are_enforced() {
        let doc = doc();
        let mut mock = mock(&doc);
        mock.compressed = vec![0; MAX_COMPRESSED + 1];
        assert!(fetch(&mut mock, &doc).is_err());
        let mut enc = xz2::write::XzEncoder::new(Vec::new(), 6);
        enc.write_all(&vec![b' '; MAX_DEFINITION + 1]).unwrap();
        assert!(
            decompress(&enc.finish().unwrap())
                .unwrap_err()
                .contains("size limit")
        );
    }
}
