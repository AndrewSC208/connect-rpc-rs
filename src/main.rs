use crate::common::runtime::Runtime;

mod common;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match Runtime::new().default().await {
        Ok(x) => {
          x.execute().await;
          Ok(())
        },
        Err(_e) => Ok(())
    }
}