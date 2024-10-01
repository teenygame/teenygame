pub async fn read(path: &str) -> Result<Vec<u8>, anyhow::Error> {
    Ok(tokio::fs::read(path).await?)
}
