use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    https_dns::run().await
}
