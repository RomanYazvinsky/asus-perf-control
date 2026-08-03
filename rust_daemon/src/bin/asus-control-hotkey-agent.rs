#[path = "../hotkey_agent.rs"]
mod hotkey_agent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    hotkey_agent::run().await
}
