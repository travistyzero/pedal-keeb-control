use crate::keymapp_client::keymapp::SetLayerRequest;
use crate::pedal::PedalPosition;
use anyhow::{Context, Result};
use hyper_util::rt::TokioIo;
use keymapp::keyboard_service_client::KeyboardServiceClient;
use std::path::Path;
use tokio::net::UnixStream;
use tokio::sync::mpsc::Receiver;
use tonic::Request;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

pub mod keymapp {
    tonic::include_proto!("api");
}

pub struct KeymappClient {
    rx: Receiver<PedalPosition>,
    client: KeyboardServiceClient<Channel>,
    mouse_layer: u8,
}

impl KeymappClient {
    pub async fn new(
        rx: Receiver<PedalPosition>,
        keymapp_socket: String,
        mouse_layer: u8,
    ) -> Result<Self> {
        let connector = service_fn(move |_: Uri| {
            let socket = keymapp_socket.clone();
            async move {
                let stream = UnixStream::connect(Path::new(&socket)).await?;
                Ok::<TokioIo<UnixStream>, anyhow::Error>(TokioIo::new(stream))
            }
        });

        let channel = Endpoint::try_from("http://[::]:50051")
            .context("Invalid endpoint URL")?
            .connect_with_connector(connector)
            .await
            .context("Failed to connect to keymapp server")?;

        let client = KeyboardServiceClient::new(channel);

        Ok(Self {
            rx,
            client,
            mouse_layer,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.rx.recv().await {
                None => {
                    println!("Channel closed");
                    return Ok(());
                }

                Some(PedalPosition::Down) => {
                    let req = Request::new(SetLayerRequest {
                        layer: self.mouse_layer as i32,
                    });
                    self.client
                        .set_layer(req)
                        .await
                        .context("Failed to set layer")?;
                }

                Some(PedalPosition::Up) => {
                    let req = Request::new(SetLayerRequest {
                        layer: self.mouse_layer as i32,
                    });
                    self.client
                        .unset_layer(req)
                        .await
                        .context("Failed to unset layer")?;
                }
            };
        }
    }
}
