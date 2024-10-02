use std::io::ErrorKind;

use super::Error;

pub async fn read(path: &str) -> Result<Vec<u8>, Error> {
    Ok(async {
        #[cfg(feature = "tokio")]
        {
            return tokio::fs::read(path).await;
        }

        #[cfg(feature = "smol")]
        {
            return smol::fs::read(path).await;
        }

        #[allow(unreachable_code)]
        {
            _ = path;
            panic!("no async runtime available!");
        }
    }
    .await
    .map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            Error::NotFound
        } else {
            Error::Other(e.into())
        }
    })?)
}
