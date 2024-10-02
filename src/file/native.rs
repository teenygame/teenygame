use std::io::ErrorKind;

use super::Error;

#[cfg(feature = "tokio")]
pub async fn read(path: &str) -> Result<Vec<u8>, Error> {
    Ok(tokio::fs::read(path).await.map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            Error::NotFound
        } else {
            Error::Other(e.into())
        }
    })?)
}

#[cfg(not(feature = "tokio"))]
pub async fn read(path: &str) -> Result<Vec<u8>, Error> {
    Ok(std::fs::read(path).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            Error::NotFound
        } else {
            Error::Other(e.into())
        }
    })?)
}
