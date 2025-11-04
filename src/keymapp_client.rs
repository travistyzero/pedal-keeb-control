use crate::pedal::PedalPosition;
use anyhow::{Context, Result};
use keymapp::keyboard_service_client::KeyboardServiceClient;
use std::path::Path;
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tokio::sync::mpsc::Receiver;
use tonic::Request;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;
use crate::keymapp_client::keymapp::{SetLayerRequest};

pub mod keymapp {
    tonic::include_proto!("api");
}

pub struct KeymappClient {
    rx: Receiver<PedalPosition>,
    client: KeyboardServiceClient<Channel>,
}

impl KeymappClient {
    pub async fn new(rx: Receiver<PedalPosition>) -> Result<Self> {
        let connector = service_fn(async |_: Uri| {
            let stream = UnixStream::connect(Path::new("/home/travis/.config/.keymapp/keymapp.sock")).await?;
            Ok::<TokioIo<UnixStream>, anyhow::Error>(TokioIo::new(stream))
        });

        let channel = Endpoint::try_from("http://[::]:50051")
            .context("Invalid endpoint URL")?
            .connect_with_connector(connector)
            .await
            .context("Failed to connect to keymapp server")?;

        let client = KeyboardServiceClient::new(channel);

        Ok(Self { rx, client })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.rx.recv().await {
                None => {
                    println!("Channel closed");
                    return Ok(());
                }
                Some(pedal_position) => {
                    match pedal_position {
                        PedalPosition::Down => {
                            let req = Request::new(SetLayerRequest { layer: 2 });
                            self.client.set_layer(req).await.context("Failed to set layer")?;
                        }
                        PedalPosition::Up => {
                            let req = Request::new(SetLayerRequest{ layer: 2 });
                            self.client.unset_layer(req).await.context("Failed to unset layer")?;
                        }
                    }

                }
            };
        }
    }
}
