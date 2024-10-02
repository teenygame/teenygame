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
    let fut = async {
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
    };
    Ok(fut.await.map_err(convert_error)?)
}
