#![allow(dead_code)]

use rusb::UsbContext;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn libusb_sanity() {
        assert_eq!(2 + 2, 4);

        let context = rusb::Context::new().unwrap();

        for device in context.devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();

            println!(
                "Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id()
            );
        }
    }

    #[test]
    #[ignore]
    fn usbcommand_create_device_not_present() {
        let mut _cmd = Usbcommand::open(DeviceType::DepthProcessor, 0)
            .expect_err("Device should not be connected, so this should err");
    }

    #[test]
    fn usbcommand_create() {
        let mut _cmd = Usbcommand::open(DeviceType::DepthProcessor, 0).unwrap();

        println!("PID: {}", _cmd.get_pid());
        println!("Serial Number: {}", _cmd.get_serial_number());
    }
}

#[derive(Debug)]
enum AllocationSource {
    User,
    Depth,
    Color,
    IMU,
    UsbDepth,
    UsbIMU,
}

#[derive(Debug)]
struct EndpointIdentifier {
    vid: u16,
    pid: u16,
    interface: u8,
    cmd_tx_endpoint: u8,
    cmd_rx_endpoint: u8,
    stream_endpoint: u8,
    source: AllocationSource,
}

const DEPTH_ENDPOINT_IDENTIFIER: EndpointIdentifier = EndpointIdentifier {
    vid: 0x045e,
    pid: 0x097c,
    interface: 0,
    cmd_tx_endpoint: 0x02,
    cmd_rx_endpoint: 0x81,
    stream_endpoint: 0x83,
    source: AllocationSource::UsbDepth,
};

const COLOR_ENDPOINT_IDENTIFIER: EndpointIdentifier = EndpointIdentifier {
    vid: 0x045e,
    pid: 0x097d,
    interface: 2,
    cmd_tx_endpoint: 0x04,
    cmd_rx_endpoint: 0x83,
    stream_endpoint: 0x82,
    source: AllocationSource::UsbIMU,
};

struct Usbcommand {
    endpoint_identifier: EndpointIdentifier,
    device_handle: rusb::DeviceHandle<rusb::Context>,
    serial_number: String,
}

impl std::fmt::Debug for Usbcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Usbcommand ({:?})", self.endpoint_identifier)
    }
}

enum DeviceType {
    DepthProcessor,
    ColorImuProcessor,
}

impl<'a> Usbcommand {
    pub fn open(device_type: DeviceType, device_index: usize) -> Result<Usbcommand, String> {
        let endpoint_identifier = match device_type {
            DeviceType::DepthProcessor => DEPTH_ENDPOINT_IDENTIFIER,
            DeviceType::ColorImuProcessor => COLOR_ENDPOINT_IDENTIFIER,
        };

        let my_local_libusb_context: Box<rusb::Context> = Box::new(rusb::Context::new().unwrap());

        let device_list = my_local_libusb_context.devices().unwrap();

        let device_option = device_list
            .iter()
            .filter(|device| {
                let device_desc = device.device_descriptor().unwrap();

                device_desc.vendor_id() == endpoint_identifier.vid
                    && device_desc.product_id() == endpoint_identifier.pid
            })
            .skip(device_index)
            .take(1)
            .last();

        let device = match device_option {
            Some(r) => r,
            None => {
                return Err(String::from("Unable to find device."));
            }
        };

        let dd = device.device_descriptor().unwrap();

        let serial_number_string_index = dd.serial_number_string_index().unwrap();

        let handle = device.open().unwrap();

        let timeout = std::time::Duration::new(5, 0);

        let languages = handle.read_languages(timeout).unwrap();

        let language = *languages.first().unwrap();

        let serial_number = handle
            .read_string_descriptor(language, serial_number_string_index, timeout)
            .unwrap();

        let new_object = Self {
            endpoint_identifier: endpoint_identifier,
            device_handle: handle,
            serial_number: serial_number,
        };

        Ok(new_object)
    }

    pub fn get_pid(&self) -> u16 {
        self.endpoint_identifier.pid
    }

    pub fn get_serial_number(&self) -> &String {
        &self.serial_number
    }
}
