use crate::operations::TarOperation;
use crate::options::TarParams;
use uucore::error::UResult;

pub(crate) struct Update;

impl TarOperation for Update {
    fn exec(&self, options: &TarParams) -> UResult<()> {
        Ok(())
    }
}
