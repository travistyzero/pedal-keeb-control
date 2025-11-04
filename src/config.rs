use crate::pedal::PedalUsbConfig;

pub struct Config {
    pub keymapp_socket: String,
    pub mouse_layer: u8,
    pub pedal_device_config: PedalUsbConfig,
}

impl Default for Config {
    fn default() -> Self {
        let keymapp_socket = get_keymapp_socket_location();

            Config {
                keymapp_socket,
                mouse_layer: 2,
                pedal_device_config: Default::default(),
            }
    }
}

#[cfg(target_os="linux")]
fn get_keymapp_socket_location() -> String {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    std::path::PathBuf::from(&home).join(".config/.keymapp/keymapp.sock").to_string_lossy().to_string()
}

#[cfg(target_os="macos")]
fn get_keymapp_socket_location() -> String {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    std::path::PathBuf::from(&home).join("Library/Containers/io.zsa.keymapp/Data/Library/Application Support/.keymapp/keymapp.sock").to_string_lossy().to_string()
}
