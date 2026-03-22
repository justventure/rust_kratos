#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rust_kratos::startup::run().await
}
