use crate::operations::Append;
use crate::operations::TarOperation;
use crate::options::TarParams;
use std::collections::hash_map::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use tar::Archive;
use uucore::error::{UResult, USimpleError};

/// [`Update`] is used to selectively append updated versions of files
/// into an archive. If the files that are passed in via the command line
/// have been updated since the their previous versions were archived. If
/// the archive has the latest version of the files already no updates are
/// made.
///
/// # Behaviors
/// Intuitivly the command title being [`Update`] invokes a certain predisposed
/// understanding of what it might do. But tar doesn't do that. [`Update`] does find
/// the files that have been updated since the archive was created. This is where the
/// inituition you have diverges. [`Update`] instead of updating the file in place
/// within the archive, the updated version of the file is appended to the end of
/// the archive. This is done because of the origin of tar which tar's full name is
/// The Tape Archive Utility, since tape is linear it is not really ideal to seek back
/// and forth to scalpel an entry out and update it, along with the rest of the data on
/// the tape. Since the entire archive member and archive members after the one being
/// updated would have to be shuffled it is both memory and time intensive.
/// Tape is linear and not Random-Access so you really have to approach tar operations
/// with this mindset
///
/// # Considerations
/// It is often better to update an archive by recreating it from the file listing of the
/// previous version of the archive then removing the old version of it. This recommendation
/// scales with compression, since an archive needs to be uncompressed to do any operation on
/// it, it makes even more since to recreate -> recompress -> tear down old.
///
/// If you do in fact use tar to manage Tape archiving then [`Update`] might make sense, with
/// the caveat that the archives are not compressed.
///
// TODO: Update and Append when handling duplicate file arguments create a symlink to the first
// file added
pub(crate) struct Update;

impl TarOperation for Update {
    fn exec(&self, params: &TarParams) -> UResult<()> {
        // when an archive is passed in from the command line
        // during Update or Append a new file is created if the
        // file path passed in doesn't exist
        let mut archive = Archive::new(
            OpenOptions::new()
                .append(true)
                .read(true)
                .create(true)
                // update and append to a file
                .open(params.archive())?,
        );

        // Wrap up entries and their mod times so they can be retrieved and checked
        // during selection for appending. This selects the most recent timestamp
        // of the file so there is no erronius appending of files
        let mut archive_members: HashMap<PathBuf, i64> = HashMap::new();
        for entry in archive.entries()? {
            let e = entry
                .map_err(|x| USimpleError::new(1, format!("error accessing entry: {:?}", x)))?;
            let path = e.path()?.to_path_buf();
            let mtime: i64 = e
                .header()
                .mtime()?
                .try_into()
                .map_err(|x| USimpleError::new(1, format!("invalid mtime: {:?}", x)))?;
            archive_members
                .entry(path)
                .and_modify(|x| {
                    if mtime > *x {
                        *x = mtime;
                    }
                })
                .or_insert(mtime);
        }

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
        let mut files_to_append: Vec<PathBuf> = Vec::new();
        for f in params.files() {
            if let Some(entry_time) = archive_members.get(f) {
                let file = File::open(f)?;
                let meta = file.metadata()?;
                if meta.mtime() > *entry_time {
                    files_to_append.push(f.to_path_buf());
                }
            } else {
                files_to_append.push(f.to_path_buf());
            }
        }

        let files_appended = Append::append_files_to_archive(archive, &files_to_append)?;
        // print file names during update
        if params.is_verbose() {
            for file_name in files_appended {
                println!("{}", file_name.as_str());
            }
        }
        Ok(())
    }
}
