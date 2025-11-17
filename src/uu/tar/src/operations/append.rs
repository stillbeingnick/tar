use uucore::error::UResult;
use crate::options::{TarOption, TarParams};
use crate::operations::TarOperation;
use std::fs::{OpenOptions, File};
use std::io::{Seek, SeekFrom};
use tar::{Archive, Builder, Header};

pub struct Append;

impl TarOperation for Append {
    fn exec(&self, params: &TarParams) -> UResult<()> {

        // NOTE: might have to seek reader to end of entries
        // Then write the entry
        let mut f = OpenOptions::new().write(true).open(params.archive())?;
        let mut builder = Builder::new(f);
        // seek to end minus 2 blocks for empty
        builder.get_mut().seek(SeekFrom::End(-1024))?;
        for file in params.files() {
            let mut ff = File::open(file)?;
            builder.append_file(file, &mut ff)?;
        }
        builder.into_inner()?;
        Ok(()) 
    }
}
