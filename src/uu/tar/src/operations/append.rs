use crate::operations::TarOperation;
use crate::options::TarParams;
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use tar::{Archive, Builder};
use uucore::error::{UResult, USimpleError};

/// Appends a single file or a list of files to the end of an archive
///
/// Arguments passed through the command line will influence how the
/// archive is appended. Both entire directories and individual files
/// can be appended in a single operation of tar.
///
/// # Append Vs. Update
///
/// While seeming to do the same thing Append and Update have two different
/// purposes.
///
/// Appending will always append the requested file to an archive, while 
/// update will only append the requested file to an archive if that 
/// file has been modifed after the recorded modified date of that same
/// file in the archive.
///
/// # Compression
///
/// It is not possible to append a file to a compressed archive without
/// first decompressing it.
///
/// So the order of operations for appending a file to a compressed archive is:
///     Decompression -> Append File -> Recompress
///
pub(crate) struct Append;

impl TarOperation for Append {
    fn exec(&self, params: &TarParams) -> UResult<()> {
        let archive = Archive::new(OpenOptions::new()
            .write(true)
            .read(true)
            .open(params.archive())?
        );

        let block_size = params.block_size().try_into().map_err(|x| USimpleError::new(1, format!("Invalid block size: {}", x)))?;
        let files_appended = Append::append_files_to_archive(
            archive,
            block_size,
            params.files()
        )?;
        // print file names during append
        if params.is_verbose(){
            for file_name in files_appended {
                println!("{}", file_name.as_str());
            }
        }
        Ok(())
    }
}

impl Append {
    pub(crate) fn append_files_to_archive(mut archive: Archive<File>, block_size: u64, files: &[PathBuf]) -> UResult<Vec<String>> { 
        // attempt to open archive entries and go to the last entry
        // .last() runs the iterator till None so tar-rs's odd way of 
        // creating the iterator using Read/Write is ok
        let end_pos = if let Some(Ok(last_entry)) = archive.entries()?.last() {
            // align to block size boundry
            if (last_entry.size() % block_size) == 0 {
                last_entry.size() + block_size + last_entry.raw_header_position()
            } else {
                block_size - (last_entry.size() % block_size)
                    + last_entry.size()
                    + block_size
                    + last_entry.raw_header_position()
            }
        } else {
            // if there is no last entry, which would mean there are no entries
            0
        };
        let mut builder = Builder::new(archive.into_inner());

        // seek to end minus 2 blocks for empty
        builder.get_mut().seek(SeekFrom::Start(end_pos))?;
       
        let mut files_appended: Vec<String> = Vec::new();
        for file in files {
            let mut ff = File::open(file)?;
            builder.append_file(file, &mut ff)?;
            files_appended.push(file.to_string_lossy().to_string());
        }
        Ok(files_appended)
    }
}
