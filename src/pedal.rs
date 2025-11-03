use std::ops::Not;
use std::thread;
use std::time::Duration;
use anyhow::{anyhow, Context, Result};
use log::info;
use rusb::{DeviceHandle, GlobalContext};
use tokio::sync::mpsc::Sender;

const VENDOR_ID: u16 = 0x3553;
const PRODUCT_ID: u16 = 0xb001;

#[derive(Debug, Copy, Clone)]
pub enum PedalPosition {
    Down,
    Up
}

impl Not for PedalPosition {
    type Output = PedalPosition;

    fn not(self) -> Self::Output {
        match self {
            PedalPosition::Down => PedalPosition::Up,
            PedalPosition::Up => PedalPosition::Down
        }
    }
}

pub struct Pedal {
    pedal_position: PedalPosition,
    device_handle: DeviceHandle<GlobalContext>,
    read_timeout:Duration,
    poll_interval: Duration,
    run: bool,
    tx: Sender<PedalPosition>,
}

impl Pedal {
    pub fn new(tx: Sender<PedalPosition>) -> Result<Self> {
        let device_handle = rusb::open_device_with_vid_pid(VENDOR_ID,PRODUCT_ID).ok_or(anyhow!("Failed to open device"))?;
        device_handle.set_auto_detach_kernel_driver(true).context(anyhow!("Failed to auto detach kernel driver"))?;
        device_handle.claim_interface(0x000).context(anyhow!("Failed to claim interface"))?;

        Ok(Pedal {
            pedal_position: PedalPosition::Up,
            device_handle,
            read_timeout: Duration::from_millis(10),
            poll_interval: Duration::from_millis(100),
            run: false,
            tx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.run = true;
        let mut buf = [0u8; 128];

        while self.run {
            let  read_result = self.device_handle.read_interrupt(0x81, &mut buf, self.read_timeout);

            match read_result {
                Err(rusb::Error::Timeout) => {},
                Ok(_) => {
                    self.pedal_position = !self.pedal_position;
                    println!("Pedal position: {:?}",  self.pedal_position);
                    self.tx.send(self.pedal_position).await.context("Failed to send Pedal position")?;
                },
                Err(e) => {
                    return Err(e).context(anyhow!("Failed to read from device"));
                }
            };


            tokio::time::sleep(self.poll_interval).await;
        }

        println!("Pedal stopped");

        Ok(())
    }


}