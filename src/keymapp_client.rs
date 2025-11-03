use keymapp::keyboard_service_client::KeyboardServiceClient;
use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use crate::pedal::PedalPosition;

pub mod keymapp {
    tonic::include_proto!("api");
}

pub struct KeymappClient {
    rx: Receiver<PedalPosition>,
}

impl KeymappClient {
    pub async fn new(rx: Receiver<PedalPosition>) -> Result<Self> {
        // KeyboardServiceClient::connect("http://[::1]:50051").await?;
        Ok(Self {
            rx
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.rx.recv().await {
                None => {
                    println!("Channel closed");
                    return Ok(());
                }
                Some(pedal_position) => {
                    println!("Got position: {:?}", pedal_position);
                }
            };
        }
    }
}
