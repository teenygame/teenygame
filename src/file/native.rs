use std::io::ErrorKind;

use super::Error;

fn convert_error(e: std::io::Error) -> Error {
    if e.kind() == ErrorKind::NotFound {
        Error::NotFound
    } else {
        Error::Other(e.into())
    }
}

#[cfg(feature = "tokio")]
pub async fn read(path: &str) -> Result<Vec<u8>, Error> {
    Ok(tokio::fs::read(path).await.map_err(convert_error)?)
}

#[cfg(not(feature = "tokio"))]
pub async fn read(path: &str) -> Result<Vec<u8>, Error> {
    Ok(std::fs::read(path).map_err(convert_error)?)
}
