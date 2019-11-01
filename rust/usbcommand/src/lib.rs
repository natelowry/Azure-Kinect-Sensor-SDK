#![allow(dead_code)]

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
    fn usbcommand_create() {
        let mut _cmd = Usbcommand::new(DeviceType::DepthProcessor);

        _cmd.open(0);
    }
}

enum AllocationSource {
    User,
    Depth,
    Color,
    IMU,
    UsbDepth,
    UsbIMU,
}

struct EndpointIdentifier {
    vid: u16,
    pid: u16,
    interface: u8,
    cmd_tx_endpoint: u8,
    cmd_rx_endpoint: u8,
    stream_endpoint: u8,
    source: AllocationSource,
}

const DEPTH_ENDPOINT_IDNTIFIER: EndpointIdentifier = EndpointIdentifier {
    vid: 0x045e,
    pid: 0x097c,
    interface: 0,
    cmd_tx_endpoint: 0x02,
    cmd_rx_endpoint: 0x81,
    stream_endpoint: 0x83,
    source: AllocationSource::UsbDepth,
};

const COLOR_ENDPOINT_IDNTIFIER: EndpointIdentifier = EndpointIdentifier {
    vid: 0x045e,
    pid: 0x097d,
    interface: 2,
    cmd_tx_endpoint: 0x04,
    cmd_rx_endpoint: 0x83,
    stream_endpoint: 0x82,
    source: AllocationSource::UsbIMU,
};

struct Usbcommand {
    libusb_context: rusb::Context,
    endpoint_identifier: EndpointIdentifier,
}

enum DeviceType {
    DepthProcessor,
    ColorImuProcessor
}

impl Usbcommand {
    pub fn new(device_type: DeviceType) -> Usbcommand {
        let endpoint_identifier;
        match device_type {
            DeviceType::DepthProcessor => {
                endpoint_identifier = DEPTH_ENDPOINT_IDNTIFIER;
            }
            DeviceType::ColorImuProcessor => {
                endpoint_identifier = COLOR_ENDPOINT_IDNTIFIER;
            }
        }

        Usbcommand {
            libusb_context: rusb::Context::new().unwrap(),
            endpoint_identifier: endpoint_identifier,
        }
    }

    pub fn open(&mut self, device_index: usize) {

        let device = self.libusb_context.devices().unwrap().iter().filter(
            |device| {
                let device_desc = device.device_descriptor().unwrap();

                device_desc.vendor_id() == self.endpoint_identifier.vid &&
                device_desc.product_id() == self.endpoint_identifier.pid
            }
        ).skip(device_index).take(1).last().unwrap();

        let dd = device.device_descriptor().unwrap();

        println!(
            "Azure Kinect Device: Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            dd.vendor_id(),
            dd.product_id()
        );
        
        
    }
}
