use crate::operations::TarOperation;
use crate::options::TarParams;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use tar::Builder;
use uucore::error::UResult;

pub struct Append;

impl TarOperation for Append {
    fn exec(&self, params: &TarParams) -> UResult<()> {
        // NOTE: might have to seek reader to end of entries
        // Then write the entry
        let f = OpenOptions::new().write(true).open(params.archive())?;
        let mut builder = Builder::new(f);

        // seek to end minus 2 blocks for empty
        builder.get_mut().seek(SeekFrom::End(-1024))?;

        for file in params.files() {
            let mut ff = File::open(file)?;
            builder.append_file(file, &mut ff)?;

            // print file names during append
            if params.is_verbose() {
                if let Some(p) = file.to_str() {
                    println!("{}", p);
                }
            }
        }

        Ok(())
    }
}
