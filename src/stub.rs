use std::fmt;

use super::Fd;

#[derive(Clone, Debug)]
#[allow(clippy::empty_enum)]
pub enum SocketStat {}

#[derive(Clone)]
pub struct Error;
impl fmt::Debug for Error {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(fmt, "socketstat not implemented on this target yet")
	}
}

pub fn socketstat(_fd: Fd) -> Result<SocketStat, Error> {
	Err(Error)
}
