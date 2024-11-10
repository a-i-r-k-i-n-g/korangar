//! A GRF file containing game assets.
mod builder;
mod mixcrypt;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

#[cfg(feature = "debug")]
use korangar_debug::logging::{Colorize, Timer};
use ragnarok_bytes::{ByteStream, FixedByteSize, FromBytes};
use ragnarok_formats::archive::{AssetTable, FileTableRow, Header};
use yazi::{decompress, Format};

pub use self::builder::NativeArchiveBuilder;
use crate::loaders::archive::native::mixcrypt::decrypt_file;
use crate::loaders::archive::Archive;

/// Represents a GRF file. GRF Files are an archive to store game assets.
/// Each GRF contains a [`Header`] with metadata (number of files, size,
/// etc.) and a table [`AssetTable`] with information about individual assets.
type FileTable = HashMap<String, FileTableRow>;

pub struct NativeArchive {
    file_table: FileTable,
    file_handle: Mutex<File>,
}

impl Archive for NativeArchive {
    fn from_path(path: &Path) -> Self {
        #[cfg(feature = "debug")]
        let timer = Timer::new_dynamic(format!("load game data from {}", path.display().magenta()));
        let mut file = File::open(path).unwrap();

        let mut file_header_buffer = vec![0u8; Header::size_in_bytes()];
        file.read_exact(&mut file_header_buffer).unwrap();
        let file_header = Header::from_bytes(&mut ByteStream::<()>::without_metadata(&file_header_buffer)).unwrap();

        assert_eq!(file_header.version, 0x200, "invalid grf version");

        let _ = file.seek(SeekFrom::Current(file_header.file_table_offset as i64)).unwrap();
        let mut file_table_buffer = vec![0; AssetTable::size_in_bytes()];

        file.read_exact(&mut file_table_buffer).unwrap();
        let file_table = AssetTable::from_bytes(&mut ByteStream::<()>::without_metadata(&file_table_buffer)).unwrap();

        let mut compressed_file_table_buffer = vec![0u8; file_table.compressed_size as usize];
        file.read_exact(&mut compressed_file_table_buffer).unwrap();
        let (decompressed, _checksum) = decompress(&compressed_file_table_buffer, Format::Zlib).unwrap();

        let file_count = file_header.get_file_count();

        let mut file_table_byte_stream = ByteStream::<()>::without_metadata(&decompressed);
        let mut assets = HashMap::with_capacity(file_count);

        for _index in 0..file_count {
            let file_information = FileTableRow::from_bytes(&mut file_table_byte_stream).unwrap();
            let file_name = file_information.file_name.to_lowercase();

            assets.insert(file_name, file_information);
        }

        #[cfg(feature = "debug")]
        timer.stop();

        // TODO: only take 64..? bytes so that loaded game archives can be extended
        // aswell
        Self {
            file_table: assets,
            file_handle: Mutex::new(file),
        }
    }

    fn get_file_by_path(&self, asset_path: &str) -> Option<Vec<u8>> {
        self.file_table.get(asset_path).map(|file_information| {
            let mut compressed_file_buffer = vec![0u8; file_information.compressed_size_aligned as usize];

            let position = file_information.offset as u64 + Header::size_in_bytes() as u64;

            {
                // Since the calling threads are sharing the IO bandwidth anyhow, I don't think
                // we need to allow this to run in parallel.
                let mut file_handle = self.file_handle.lock().unwrap();
                file_handle.seek(SeekFrom::Start(position)).unwrap();
                file_handle
                    .read_exact(&mut compressed_file_buffer)
                    .expect("Can't read archive content");
            }

            decrypt_file(file_information, &mut compressed_file_buffer);

            let (uncompressed_file_buffer, _checksum) =
                decompress(&compressed_file_buffer, Format::Zlib).expect("Can't decompress archive content");

            uncompressed_file_buffer
        })
    }

    fn get_files_with_extension(&self, files: &mut Vec<String>, extension: &str) {
        let found_files = self
            .file_table
            .iter()
            .filter(|(file_name, row)| file_name.ends_with(extension) && row.flags == 0x01)
            .map(|(file_name, _)| file_name.clone());

        files.extend(found_files);
    }
}
