//! File-related utility functions.

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod web;

/// Errors that can occur while reading a file.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// File not found.
    #[error("not found")]
    NotFound,

    /// An underlying error occurred.
    #[error("other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Reads a file from the given path and return its bytes.
///
/// On WASM, this will perform a HTTP GET request.
pub async fn read(path: &str) -> Result<Vec<u8>, Error> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        return native::read(path).await;
    }

    #[cfg(target_arch = "wasm32")]
    {
        return web::read(path).await;
    }
}
