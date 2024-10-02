#[cfg(feature = "tokio")]
use tokio::fs::read as read_impl;

#[cfg(feature = "smol")]
use smol::fs::read as read_impl;

use std::io::ErrorKind;

use super::Error;

fn convert_error(e: std::io::Error) -> Error {
    if e.kind() == ErrorKind::NotFound {
        Error::NotFound
    } else {
        Error::Other(e.into())
    }
}

pub async fn read(path: &str) -> Result<Vec<u8>, Error> {
    Ok(read_impl(path).await.map_err(convert_error)?)
}
