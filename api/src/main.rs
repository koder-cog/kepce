#[tokio::main]
async fn main() -> anyhow::Result<()> {
    api::run().await
}
