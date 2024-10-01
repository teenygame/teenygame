use gloo_net::http::Request;

pub async fn read(path: &str) -> Result<Vec<u8>, anyhow::Error> {
    Ok(Request::get(&path).send().await?.binary().await?)
}
