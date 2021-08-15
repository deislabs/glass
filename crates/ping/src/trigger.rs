use anyhow::Error;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time;

#[async_trait]
pub trait Ping: Clone + Send + Sync + 'static {
    async fn execute(&self, input: String) -> Result<String, Error>;
}

pub struct TimerTrigger {
    pub interval: Duration,
}

impl TimerTrigger {
    pub async fn run(&self, runtime: impl Ping) -> Result<(), Error> {
        let mut interval = time::interval(self.interval);
        loop {
            interval.tick().await;
            let res = runtime
                .execute(format!(
                    "{}",
                    chrono::Local::now().format("%Y-%m-%d][%H:%M:%S")
                ))
                .await?;

            log::info!("{}\n", res);
        }
    }
}
