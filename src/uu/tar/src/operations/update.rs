use crate::operations::TarOperation;
use crate::operations::Append;
use tar::Archive;
use std::fs::OpenOptions;
use crate::options::TarParams;
use uucore::error::{UResult, USimpleError};

pub(crate) struct Update;

impl TarOperation for Update {
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
