pub use failure::err_msg;
use failure::Error;

pub type Result<T> = std::result::Result<T, Error>;
