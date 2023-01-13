use crate::resources::*;
use framework::prelude::*;
use packets::structures::AssetDataType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::sync::Arc;

struct ServerAssetDownload {
    remote_path: String,
    last_modified: u64,
    expected_size: usize,
    save_to_disk: bool,
    data_type: AssetDataType,
    data: Vec<u8>,
}

struct CachedServerAsset {
    remote_path: String,
    local_path: String,
    last_modified: u64,
    data: Option<Vec<u8>>,
}

pub struct StoredServerAsset {
    pub remote_path: String,
    pub last_modified: u64,
}

impl CachedServerAsset {
    fn new(path_prefix: &str, remote_path: String, last_modified: u64) -> Self {
        let encoded_remote_path = uri_encode(&remote_path);

        let mut local_path = path_prefix.to_string();
        write!(&mut local_path, "{last_modified}-{encoded_remote_path}").unwrap();

        Self {
            remote_path,
            local_path,
            last_modified,
            data: None,
        }
    }

    fn decode_local(path_prefix: &str, local_name: &str) -> Option<Self> {
        let local_path = format!("{path_prefix}{local_name}");

        let (last_modified_str, encoded_remote_path) = local_name.split_once('-')?;

        Some(Self {
            local_path,
            remote_path: uri_decode(encoded_remote_path)?,
            last_modified: last_modified_str.parse().ok()?,
            data: None,
        })
    }
}

pub struct ServerAssetManager {
    path_prefix: String,
    stored_assets: RefCell<HashMap<String, CachedServerAsset>>,
    textures: RefCell<HashMap<String, Arc<Texture>>>,
    sounds: RefCell<HashMap<String, SoundBuffer>>,
    current_download: Option<ServerAssetDownload>,
}

impl ServerAssetManager {
    pub fn new(game_io: &GameIO, address: &str) -> Self {
        let address = packets::address_parsing::strip_data(address).replace(':', "_p");
        let address = uri_encode(&address);

        const SEP: char = std::path::MAIN_SEPARATOR;
        let path_prefix =
            ResourcePaths::clean_folder(&format!("{}{}", ResourcePaths::CACHE_FOLDER, address));

        // find stored assets
        let assets = Self::find_stored_assets(&path_prefix);

        // setup texture map
        let mut textures = HashMap::new();

        let local_assets = &game_io.resource::<Globals>().unwrap().assets;
        textures.insert(
            ResourcePaths::BLANK.to_string(),
            local_assets.texture(game_io, ResourcePaths::BLANK),
        );

        // setup sound map
        let mut sounds = HashMap::new();

        sounds.insert(
            ResourcePaths::BLANK.to_string(),
            local_assets.audio(ResourcePaths::BLANK),
        );

        Self {
            path_prefix,
            stored_assets: RefCell::new(assets),
            textures: RefCell::new(textures),
            sounds: RefCell::new(sounds),
            current_download: None,
        }
    }

    fn find_stored_assets(path: &str) -> HashMap<String, CachedServerAsset> {
        let mut assets = HashMap::new();

        if let Err(err) = fs::create_dir_all(path) {
            log::error!("failed to create cache folder in \"{path}\": {err}");
            return assets;
        }

        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                log::error!("failed to find cache folder \"{path}\": {err}");
                return assets;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    log::error!("error while reading from cache folder in \"{path}\": {err}");
                    return assets;
                }
            };

            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    continue;
                }
            }

            let file_name = entry.file_name();
            let file_name_str = match file_name.to_str() {
                Some(name) => name,
                None => continue,
            };

            if let Some(asset) = CachedServerAsset::decode_local(path, file_name_str) {
                assets.insert(asset.remote_path.clone(), asset);
            }
        }

        assets
    }

    pub fn delete_asset(&self, remote_path: &str) {
        let asset = match self.stored_assets.borrow_mut().remove(remote_path) {
            Some(asset) => asset,
            None => return,
        };

        let _ = fs::remove_file(&asset.local_path);
    }

    pub fn store_asset(&self, remote_path: String, last_modified: u64, data: Vec<u8>, write: bool) {
        let mut asset =
            CachedServerAsset::new(&self.path_prefix, remote_path.clone(), last_modified);

        if write {
            let _ = fs::write(&asset.local_path, &data);
        }

        asset.data = Some(data);

        self.stored_assets.borrow_mut().insert(remote_path, asset);
    }

    pub fn start_download(
        &mut self,
        remote_path: String,
        last_modified: u64,
        expected_size: usize,
        data_type: AssetDataType,
        write: bool,
    ) {
        self.current_download = Some(ServerAssetDownload {
            remote_path,
            last_modified,
            expected_size,
            save_to_disk: write,
            data_type,
            data: Vec::new(),
        });
    }

    pub fn receive_download_data(&mut self, game_io: &GameIO, data: Vec<u8>) {
        let download = match &mut self.current_download {
            Some(download) => download,
            None => {
                log::warn!("Received data for a server asset when no download has started");
                return;
            }
        };

        download.data.extend(data);

        if download.data.len() < download.expected_size {
            // still working on this file
            return;
        }

        let download = self.current_download.take().unwrap();

        if download.data.len() > download.expected_size {
            log::warn!(
                "Downloaded size for {:?} is larger than expected, discarding file",
                download.remote_path
            );

            return;
        }

        let mut data = download.data;

        if download.data_type == AssetDataType::CompressedText {
            use flate2::read::ZlibDecoder;
            use std::io::Read;

            let source = std::mem::take(&mut data);

            let mut decoder = ZlibDecoder::new(&*source);

            if let Err(e) = decoder.read_to_end(&mut data) {
                log::error!("failed to decompress text from server: {e}");
            }
        }

        let remote_path = download.remote_path;

        self.store_asset(
            remote_path.clone(),
            download.last_modified,
            data,
            download.save_to_disk,
        );

        match download.data_type {
            AssetDataType::Texture => {
                // cache as texture
                self.texture(game_io, &remote_path);
            }
            AssetDataType::Audio => {
                // cache as audio
                self.audio(&remote_path);
            }
            _ => {}
        }
    }

    pub fn stored_assets(&self) -> Vec<StoredServerAsset> {
        self.stored_assets
            .borrow()
            .values()
            .map(|asset| StoredServerAsset {
                remote_path: asset.remote_path.clone(),
                last_modified: asset.last_modified,
            })
            .collect()
    }
}

impl AssetManager for ServerAssetManager {
    fn local_path(&self, path: &str) -> String {
        self.stored_assets
            .borrow()
            .get(path)
            .map(|asset| asset.local_path.clone())
            .unwrap_or_default()
    }

    fn binary(&self, path: &str) -> Vec<u8> {
        if path == ResourcePaths::BLANK {
            return Vec::new();
        }

        let mut stored_assets = self.stored_assets.borrow_mut();
        let asset = match stored_assets.get_mut(path) {
            Some(asset) => asset,
            None => return Vec::new(),
        };

        match &asset.data {
            Some(data) => data.clone(),
            None => {
                let res = fs::read(&asset.local_path);

                if let Err(err) = &res {
                    log::warn!("failed to load {:?}: {}", path, err);
                }

                let bytes = res.unwrap_or_default();

                asset.data = Some(bytes.clone());

                bytes
            }
        }
    }

    fn text(&self, path: &str) -> String {
        if path == ResourcePaths::BLANK {
            return String::new();
        }

        let bytes = self.binary(path);
        let res = String::from_utf8(bytes);

        if let Err(err) = &res {
            log::warn!("failed to read {:?} as a string: {}", path, err);
        }

        res.unwrap_or_default()
    }

    fn texture(&self, game_io: &GameIO, path: &str) -> Arc<Texture> {
        let mut textures = self.textures.borrow_mut();

        if let Some(texture) = textures.get(path) {
            texture.clone()
        } else {
            let bytes = self.binary(path);
            let texture = match Texture::load_from_memory(game_io, &bytes) {
                Ok(texture) => texture,
                Err(_) => textures.get(ResourcePaths::BLANK).unwrap().clone(),
            };

            textures.insert(path.to_string(), texture.clone());
            texture
        }
    }

    fn audio(&self, path: &str) -> SoundBuffer {
        let mut sounds = self.sounds.borrow_mut();

        if let Some(sound) = sounds.get(path) {
            sound.clone()
        } else {
            let sound = SoundBuffer(Arc::new(self.binary(path)));
            sounds.insert(path.to_string(), sound.clone());
            sound
        }
    }
}

fn uri_encode(path: &str) -> String {
    let mut encoded_string = String::with_capacity(path.len());

    for b in path.bytes() {
        if b.is_ascii_alphanumeric() || b == b'.' || b == b' ' || b == b'-' || b == b'_' {
            // doesn't need to be encoded
            encoded_string.push(b as char);
            continue;
        }

        // needs encoding
        write!(&mut encoded_string, "%{:0>2X}", b).unwrap();
    }

    encoded_string
}

fn uri_decode(path: &str) -> Option<String> {
    let mut decoded_string = String::with_capacity(path.len());

    let mut chars = path.chars().enumerate();

    while let Some((i, c)) = chars.next() {
        if c != '%' {
            // doesn't need to be decoded
            decoded_string.push(c);
            continue;
        }

        // needs decoding

        // skip two, also verifies that two characters exist for the next lines
        chars.next()?;
        chars.next()?;

        let b = u8::from_str_radix(&path[i + 1..i + 3], 16).ok()?;
        decoded_string.push(b as char);
    }

    Some(decoded_string)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn uri_encoding() {
        const INPUT: &str = "a.b c-d_e:d?e%";
        const EXPECTED: &str = "a.b c-d_e%3Ad%3Fe%25";
        const ENCODED_MALFORMED: &str = "%";
        const BLANK: &str = "";

        assert_eq!(uri_encode(INPUT), EXPECTED);
        assert_eq!(uri_decode(EXPECTED), Some(INPUT.to_string()));
        assert_eq!(uri_decode(ENCODED_MALFORMED), None);
        assert_eq!(uri_decode(BLANK), Some(BLANK.to_string()));
    }
}
