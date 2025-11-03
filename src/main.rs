use std::thread;
use std::time::Duration;

const VENDOR_ID: u16 = 0x3553;
const PRODUCT_ID: u16 = 0xb001;


fn main() {
    let mut pedal_down = false;

    let device_handle = rusb::open_device_with_vid_pid(VENDOR_ID,PRODUCT_ID).expect("Failed to open device");
    device_handle.set_auto_detach_kernel_driver(true).expect("Failed to detach kernel driver");
    device_handle.claim_interface(0x000).expect("Failed to claim interface");

    let mut buf = [0u8; 128];
    let timeout = Duration::from_millis(10);

    device_handle.claim_interface(0x000).expect("Failed to claim interface");

    let t = thread::spawn(move || {
        loop {
            let read_result = device_handle.read_interrupt(0x81, &mut buf, timeout);

            match read_result {
                Err(rusb::Error::Timeout) => {},
                Ok(_) => {
                    pedal_down = !pedal_down;
                },
                Err(_) => {
                    panic!("Failed to read from device: {}", read_result.err().unwrap());
                }
            };

            println!("Pedal down: {}", pedal_down);

            thread::sleep(Duration::from_millis(100));
        }
    });

    t.join().unwrap();

}
