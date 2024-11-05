use gloo_net::http::Request;

use super::Error;

pub async fn read(path: &str) -> Result<Vec<u8>, Error> {
    if url::Url::parse(path).is_ok() {
        // Don't allow use of URLs.
        return Err(Error::NotFound);
    }

    let resp = Request::get(&path)
        .send()
        .await
        .map_err(|e| Error::Other(e.into()))?;

    if resp.status() == 404 {
        return Err(Error::NotFound);
    }

    Ok(resp.binary().await.map_err(|e| Error::Other(e.into()))?)
}
