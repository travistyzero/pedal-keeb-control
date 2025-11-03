
mod keymapp_client;
mod pedal;

use tokio::sync::mpsc;
use crate::pedal::{Pedal, PedalPosition};
use anyhow::Result;
use crate::keymapp_client::KeymappClient;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel::<PedalPosition>(10);

    let mut producer = Pedal::new(tx)?;
    let mut consumer = KeymappClient::new(rx).await?;

    let producer_task = tokio::spawn(async move {
       producer.run().await
    });

    let consumer_task = tokio::spawn(async move {
        consumer.run().await
    });

    tokio::try_join!(producer_task, consumer_task).map(|_| ())?;

    println!("Done");

    Ok(())
}
