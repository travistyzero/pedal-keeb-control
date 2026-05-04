
mod keymapp_client;
mod pedal;
mod config;

use tokio::sync::mpsc;
use crate::pedal::{Pedal, PedalPosition};
use anyhow::Result;
use crate::config::Config;
use crate::keymapp_client::KeymappClient;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::default();

    let (tx, rx) = mpsc::channel::<PedalPosition>(10);

    let mut producer = Pedal::new(tx, config.pedal_device_config)?;
    let mut consumer = KeymappClient::new(rx, config.keymapp_socket, config.mouse_layer).await?;

    let producer_task = tokio::spawn(async move {
       producer.run().await
    });

    let consumer_task = tokio::spawn(async move {
        consumer.run().await
    });

    println!("Ready");

    tokio::try_join!(producer_task, consumer_task).map(|_| ())?;

    println!("Done");

    Ok(())
}
