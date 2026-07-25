// SPDX-FileCopyrightText: 2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::file::hash_bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, metadata};
use std::io::{Read, Seek, SeekFrom, Write, copy};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MAGIC: &[u8; 8] = b"ALETHEIA";
const VERSION_1: u8 = 1;
const VERSION_2: u8 = 2;
const CURRENT_VERSION: u8 = VERSION_2;
const MIN_HEADER_SIZE: usize = 34;
const INDEX_CHECKSUM_HEX_LEN: usize = 64; // Only present in v2 and onward

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Checksum mismatch - expected: {0}, actual: {1}")]
    ChecksumMismatch(String, String),
    #[error("File not found in archive: {0}")]
    FileNotFound(String),
    #[error("Invalid archive format")]
    InvalidArchive,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8)
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Deserialize, Serialize)]
pub enum CompressionType {
    None = 0,
    Zstd = 1
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FileEntry {
    pub checksum: String,
    compression: CompressionType,
    data_offset: u64,
    data_size: u64,
    pub modified: SystemTime,
    pub shrunk_path: String
}

pub struct ArchiveWriter {
    files: Vec<(FileEntry, PathBuf)>,
    game: String,
    path: PathBuf
}

pub struct ArchiveReader {
    decompressed: HashMap<String, Vec<u8>>,
    pub created: u64,
    pub files: Vec<FileEntry>,
    pub game: String
}

impl ArchiveWriter {
    pub fn new(game: String, path: &Path) -> Self {
        Self { files: vec![], game, path: path.to_path_buf() }
    }

    pub fn add_file(&mut self, shrunk_path: &str, source: &Path, hash: String) {
        self.files.push((
            FileEntry {
                checksum: hash,
                compression: CompressionType::None,
                data_offset: 0,
                data_size: 0,
                modified: SystemTime::UNIX_EPOCH,
                shrunk_path: shrunk_path.to_owned()
            },
            source.to_path_buf()
        ));
    }

    pub fn finalize(self) -> Result<()> {
        if self.files.is_empty() {
            return Ok(());
        }

        let mut file = File::create(&self.path)?;
        let header_size = MIN_HEADER_SIZE + INDEX_CHECKSUM_HEX_LEN + self.game.len();
        file.write_all(&vec![0u8; header_size])?;

        let mut next_offset = header_size as u64;
        let mut entries = Vec::new();

        for (mut entry, source_path) in self.files {
            let metadata = metadata(&source_path)?;
            let uncompressed_size = metadata.len();
            let start_pos = file.stream_position()?;

            let mut source = File::open(&source_path)?;
            let compression = if uncompressed_size >= 1024 {
                let mut encoder = zstd::Encoder::new(&mut file, 3)?;
                copy(&mut source, &mut encoder)?;
                encoder.finish()?;
                CompressionType::Zstd
            } else {
                copy(&mut source, &mut file)?;
                CompressionType::None
            };

            let end_pos = file.stream_position()?;
            let data_size = end_pos - start_pos;

            entry.compression = compression;
            entry.data_offset = next_offset;
            entry.data_size = data_size;
            entry.modified = metadata.modified()?;

            entries.push(entry);
            next_offset += data_size;
        }

        let index_offset = next_offset;
        let index_bytes = postcard::to_allocvec(&entries)?;
        let index_checksum = hash_bytes(&index_bytes);

        file.write_all(&index_bytes)?;

        let index_size = file.stream_position()? - index_offset;

        Self::write_header(&mut file, index_offset, index_size, &self.game, &index_checksum)?;

        Ok(())
    }

    fn write_header(file: &mut File, index_offset: u64, index_size: u64, game: &str, index_checksum: &str) -> Result<()> {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let game_bytes = game.as_bytes();
        let game_len = u8::try_from(game_bytes.len()).unwrap();

        let mut buf = Vec::with_capacity(MIN_HEADER_SIZE + INDEX_CHECKSUM_HEX_LEN + game_bytes.len());
        buf.extend_from_slice(MAGIC);
        buf.push(CURRENT_VERSION);
        buf.extend_from_slice(index_checksum.as_bytes());
        buf.extend_from_slice(&now.to_le_bytes());
        buf.push(game_len);
        buf.extend_from_slice(game_bytes);
        buf.extend_from_slice(&index_offset.to_le_bytes());
        buf.extend_from_slice(&index_size.to_le_bytes());

        file.seek(SeekFrom::Start(0))?;
        file.write_all(&buf)?;

        Ok(())
    }
}

impl ArchiveReader {
    fn parse(file: &mut File) -> Result<(String, u64, u64, Vec<FileEntry>)> {
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(Error::InvalidArchive);
        }

        let mut version = [0u8; 1];
        file.read_exact(&mut version)?;

        let index_checksum = match version[0] {
            VERSION_1 => None,
            VERSION_2 => {
                let mut buf = [0u8; INDEX_CHECKSUM_HEX_LEN];
                file.read_exact(&mut buf)?;
                Some(String::from_utf8(buf.to_vec()).map_err(|_| Error::InvalidArchive)?)
            }
            other => return Err(Error::UnsupportedVersion(other))
        };

        let mut created_bytes = [0u8; 8];
        file.read_exact(&mut created_bytes)?;
        let created = u64::from_le_bytes(created_bytes);

        let mut game_len_bytes = [0u8; 1];
        file.read_exact(&mut game_len_bytes)?;
        let game_name_len = u8::from_le_bytes(game_len_bytes);

        let mut game_name_bytes = vec![0u8; game_name_len as usize];
        file.read_exact(&mut game_name_bytes)?;
        let game = String::from_utf8(game_name_bytes).map_err(|_| Error::InvalidArchive)?;

        let mut index_offset_bytes = [0u8; 8];
        file.read_exact(&mut index_offset_bytes)?;
        let index_offset = u64::from_le_bytes(index_offset_bytes);

        let mut index_size_bytes = [0u8; 8];
        file.read_exact(&mut index_size_bytes)?;
        let index_size = u64::from_le_bytes(index_size_bytes);

        let file_size = file.metadata()?.len();
        let index_end = index_offset.checked_add(index_size).ok_or(Error::InvalidArchive)?;
        if index_end > file_size {
            return Err(Error::InvalidArchive);
        }

        file.seek(SeekFrom::Start(index_offset))?;
        let mut index_bytes = vec![0u8; index_size as usize];
        file.read_exact(&mut index_bytes)?;

        if let Some(checksum) = index_checksum {
            let actual = hash_bytes(&index_bytes);
            if actual != checksum {
                return Err(Error::ChecksumMismatch(checksum, actual));
            }
        }

        let files: Vec<FileEntry> = postcard::from_bytes(&index_bytes)?;

        Ok((game, created, file_size, files))
    }

    pub fn read_index(path: &Path) -> Result<(String, u64, Vec<FileEntry>)> {
        let mut file = File::open(path)?;
        let (game, created, _file_size, files) = Self::parse(&mut file)?;
        Ok((game, created, files))
    }

    pub fn open(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let (game, created, file_size, files) = Self::parse(&mut file)?;
        let mut decompressed = HashMap::with_capacity(files.len());

        for entry in &files {
            let data_end = entry.data_offset.checked_add(entry.data_size).ok_or(Error::InvalidArchive)?;
            if data_end > file_size {
                return Err(Error::InvalidArchive);
            }

            let data = Self::decompress(&mut file, entry)?;
            let checksum = hash_bytes(&data);
            if checksum != entry.checksum {
                return Err(Error::ChecksumMismatch(entry.checksum.clone(), checksum));
            }

            decompressed.insert(entry.shrunk_path.clone(), data);
        }

        Ok(Self { decompressed, created, files, game })
    }

    fn decompress(file: &mut File, entry: &FileEntry) -> Result<Vec<u8>> {
        file.seek(SeekFrom::Start(entry.data_offset))?;
        let mut compressed = vec![0u8; usize::try_from(entry.data_size).map_err(|_| Error::InvalidArchive)?];
        file.read_exact(&mut compressed)?;

        match entry.compression {
            CompressionType::None => Ok(compressed),
            CompressionType::Zstd => zstd::decode_all(&compressed[..]).map_err(|_| Error::InvalidArchive)
        }
    }

    pub fn extract_file(&self, shrunk_path: &str, dest: &Path) -> Result<()> {
        let entry =
            self.files.iter().find(|e| e.shrunk_path == shrunk_path).ok_or_else(|| Error::FileNotFound(shrunk_path.to_owned()))?;

        let data = &self.decompressed[shrunk_path];
        let mut output = File::create(dest)?;
        output.write_all(data)?;
        output.set_modified(entry.modified)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        std::fs::create_dir_all("tests").unwrap();

        let temp = Path::new("tests");
        let archive_path = temp.join("test.aletheia");

        let test_file = temp.join("test.txt");
        let mut f = File::create(&test_file).unwrap();
        f.write_all(b"Hello, World!").unwrap();
        drop(f);

        let mut writer = ArchiveWriter::new("Test Game".into(), &archive_path);
        writer.add_file("test.txt", &test_file, "288a86a79f20a3d6dccdca7713beaed178798296bdfa7913fa2a62d9727bf8f8".to_owned());
        writer.finalize().unwrap();

        let reader = ArchiveReader::open(&archive_path).unwrap();
        assert_eq!(&reader.game, "Test Game");
        assert_eq!(reader.files.len(), 1);
        assert_eq!(reader.files[0].checksum, "288a86a79f20a3d6dccdca7713beaed178798296bdfa7913fa2a62d9727bf8f8");

        let extract_path = temp.join("extracted.txt");
        reader.extract_file("test.txt", &extract_path).unwrap();

        let content = std::fs::read_to_string(&extract_path).unwrap();
        assert_eq!(content, "Hello, World!");

        std::fs::remove_dir_all("tests").unwrap();
    }
}
