use crate::operations::TarOperation;
use crate::operations::Append;
use tar::Archive;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::collections::hash_map::HashMap;
use crate::options::TarParams;
use uucore::error::{UResult, USimpleError};

pub(crate) struct Update;

impl TarOperation for Update {
    fn exec(&self, params: &TarParams) -> UResult<()> {
        let mut archive = Archive::new(OpenOptions::new()
            .write(true)
            .read(true)
            .open(params.archive())?
        );

        let block_size = params.block_size().try_into().map_err(|x| USimpleError::new(1, format!("Invalid block size: {}", x)))?;

        // Wrap up entries and their mod times so they can be retrieved and checked 
        // during selection for appending
        let archive_members: HashMap<PathBuf, i64> = HashMap::from_iter(archive.entries()?.into_iter().filter_map(|x| {
            if let Ok((p, t)) = x.and_then(|entry| {
                Ok((entry.header().path().unwrap().to_path_buf(),
                    entry.header().mtime().unwrap().try_into().unwrap()))
            })
            {
                return Some((p, t))
            }
            None
        }));

        // reseek the archive to the beginning since the
        // File handle is seeked during the call to entries
        let mut fp = archive.into_inner();
        fp.seek(SeekFrom::Start(0))?;

        archive = Archive::new(fp);

        // if the file that listed in the Files argument is present in the archive
        // AND the modified time on the file in the file system NOT in the archive
        // is GREATER than the modifed time recorded for that file in the archive
        // OR if the file from the Files in the file argument is NOT present in the 
        // archive append it
        //
        // TODO: Handle errors during opening and archive traversal
        // 
        let files_to_append: Vec<PathBuf> = params.files().iter().filter_map(|f| {
            if archive_members.get(f).is_some_and(|entry_time| {
                File::open(f).is_ok_and(|file| {
                    file.metadata().is_ok_and(|meta| {
                        if meta.mtime() <= *entry_time {
                            true
                        } else {
                            false
                        }
                    })
                })
            }){
                None
            } else {
                Some(f.to_owned())
            } 
        }).collect();

        let files_appended = Append::append_files_to_archive(
            archive,
            block_size,
            &files_to_append
        )?;
        // print file names during update
        if params.is_verbose(){
            for file_name in files_appended {
                println!("{}", file_name.as_str());
            }
        }
        Ok(())
    }
}
