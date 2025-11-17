use uucore::error::UResult;
use crate::options::TarParams;
use crate::operations::TarOperation;
use std::fs::File;
use tar::{Archive, Builder};

pub struct Append;

impl TarOperation for Append {
    fn exec(&self, params: &TarParams) -> UResult<()> {

        // NOTE: might have to seek reader to end of entries
        // Then write the entry
        let mut archive = Archive::new(File::open(params.archive())?);
        Ok(()) 
    }
}
