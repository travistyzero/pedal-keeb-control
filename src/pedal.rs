use std::ops::Not;
use std::time::Duration;
use anyhow::{anyhow, Context, Result};
use rusb::{DeviceHandle, GlobalContext};
use tokio::sync::mpsc::Sender;

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
    usb_config: PedalUsbConfig
}

pub struct PedalUsbConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface: u8,
    pub interrupt: u8
}

impl Default for PedalUsbConfig {
    fn default() -> Self {
        PedalUsbConfig {
            vendor_id: 0x3553,
            product_id: 0xb001,
            interface: 0x00,
            interrupt: 0x81,
        }
    }
}

impl Pedal {
    pub fn new(tx: Sender<PedalPosition>, usb_config: PedalUsbConfig) -> Result<Self> {
        let device_handle = rusb::open_device_with_vid_pid(usb_config.vendor_id, usb_config.product_id).ok_or(anyhow!("Failed to open device"))?;
        device_handle.set_auto_detach_kernel_driver(true).context(anyhow!("Failed to auto detach kernel driver"))?;
        device_handle.claim_interface(usb_config.interface).context(anyhow!("Failed to claim interface"))?;

        Ok(Pedal {
            pedal_position: PedalPosition::Up,
            device_handle,
            read_timeout: Duration::from_millis(20),
            poll_interval: Duration::from_millis(80),
            run: false,
            tx,
            usb_config
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.run = true;
        let mut buf = [0u8; 128];

        while self.run {
            let  read_result = self.device_handle.read_interrupt(self.usb_config.interrupt, &mut buf, self.read_timeout);

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