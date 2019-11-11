#![allow(dead_code)]
use rusb::UsbContext;

use std::fmt::Debug;


mod protocol;
pub mod error;
pub use error::*;

pub use protocol::UsbResult;

use protocol::{
    EndpointIdentifier,
    UsbcommandPacket,
    UsbCommandResponse,
};

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
        let _cmd = Usbcommand::open(DeviceType::DepthProcessor, 0)
            .expect_err("Device should not be connected, so this should err");
    }

    #[test]
    #[ignore]
    fn usbcommand_create() {
        let cmd = Usbcommand::open(DeviceType::DepthProcessor, 0).unwrap();

        println!("PID: {}", cmd.pid());
        println!("Serial Number: {}", cmd.serial_number());

        std::mem::drop(cmd);
    }

    #[test]
    fn read_serial_number() {
        let mut cmd = Usbcommand::open(DeviceType::DepthProcessor, 0).unwrap();

        let mut snbuffer: [u8; 128] = [0; 128];

        let (transferred, result) = cmd.read(0x00000115, Option::None, &mut snbuffer).unwrap();

        let sn_from_cmd = String::from_utf8(snbuffer.to_vec()).unwrap();
        let sn_from_descriptor = cmd.serial_number();

        println!("transferred: {}", transferred);
        println!("result: {:?}", result);
        println!("SN from command       : {}", sn_from_cmd);
        println!("SN from usb descriptor: {}", sn_from_descriptor);
    }
}

pub enum DeviceType {
    DepthProcessor,
    ColorImuProcessor,
}


/// A command pipe to an Azure Kinect USB device is represented here
pub struct Usbcommand {
    endpoint_identifier: EndpointIdentifier,
    device_handle: rusb::DeviceHandle<rusb::Context>,
    serial_number: String,
    timeout_duration: std::time::Duration,
    transaction_id: u32,
}

impl std::fmt::Debug for Usbcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Usbcommand ({:?})", self.endpoint_identifier)
    }
}

impl<'a> Usbcommand {
    /// Opens a connection to a device
    ///
    /// # Examples
    ///
    /// ```
    /// use usbcommand::{Usbcommand, DeviceType};
    /// let mut _cmd = Usbcommand::open(DeviceType::DepthProcessor, 0).unwrap();
    /// ```
    pub fn open(device_type: DeviceType, device_index: usize) -> Result<Usbcommand, Error> {

        // Select the endpoint information for this device type
        let endpoint_identifier = match device_type {
            DeviceType::DepthProcessor => protocol::DEPTH_ENDPOINT_IDENTIFIER,
            DeviceType::ColorImuProcessor => protocol::COLOR_ENDPOINT_IDENTIFIER,
        };

        let context: Box<rusb::Context> = Box::new(rusb::Context::new().unwrap());

        let device = context
            .devices()?
            .iter()
            .filter(|device| {
                let device_desc = device.device_descriptor().unwrap();

                device_desc.vendor_id() == endpoint_identifier.vid
                    && device_desc.product_id() == endpoint_identifier.pid
            })
            .skip(device_index)
            .take(1)
            .last();

        let device = match device {
            Option::Some(x) => x,
            Option::None => {
                println!("Failed to find device");
                return Err(Error::Fail);
            }
        };

        let timeout = std::time::Duration::new(5, 0);
        let mut handle = device.open()?;

        let serial_number_string_index = device
            .device_descriptor()?
            .serial_number_string_index()
            .unwrap();

        let language = *match handle.read_languages(timeout)?.first() {
            Option::Some(x) => x,
            Option::None => {
                println!("Failed to find language");
                return Err(Error::Fail);
            }
        };

        let serial_number =
            handle.read_string_descriptor(language, serial_number_string_index, timeout)?;

        if handle.active_configuration()? != 1 {
            handle.set_active_configuration(1)?;
        }

        match handle.kernel_driver_active(endpoint_identifier.interface) {
            Ok(x) => {
                if x == true {
                    handle.detach_kernel_driver(endpoint_identifier.interface)?;
                }
            }
            Err(_) => {}
        }

        handle.claim_interface(endpoint_identifier.interface)?;

        Ok(Self {
            endpoint_identifier: endpoint_identifier,
            device_handle: handle,
            serial_number: serial_number,
            timeout_duration: timeout,
            transaction_id: 0,
        })
    }

    /// Gets the Product ID of the device
    ///
    /// # Examples
    ///
    /// ```
    /// use usbcommand::{Usbcommand, DeviceType};
    /// let mut cmd = Usbcommand::open(DeviceType::DepthProcessor, 0).unwrap();
    /// let pid = cmd.pid();
    /// assert_ne!(pid, 0);
    /// ```
    pub fn pid(&self) -> u16 {
        self.endpoint_identifier.pid
    }

    /// Gets the Serial Number of the device
    ///
    /// # Examples
    ///
    /// ```
    /// use usbcommand::{Usbcommand, DeviceType};
    /// let mut cmd = Usbcommand::open(DeviceType::DepthProcessor, 0).unwrap();
    /// let serial_number = cmd.serial_number();
    /// assert_eq!(serial_number, "000070792012");
    /// ```
    pub fn serial_number(&self) -> &String {
        &self.serial_number
    }

    pub fn read(
        &mut self,
        cmd_code: u32,
        cmd_data: Option<&[u8]>,
        rx_data: &mut [u8],
    ) -> Result<(usize, UsbResult), Error> {
        let transaction_id = self.transaction_id;
        self.transaction_id = transaction_id + 1;

        // Construct the command packet (containing a header and the cmd_data in a contiguous block)
        let packet = UsbcommandPacket::new(cmd_code, transaction_id, cmd_data);

        // Send the command
        self.device_handle.write_bulk(
            self.endpoint_identifier.cmd_tx_endpoint,
            packet.as_bytes(),
            self.timeout_duration,
        )?;

        // Read the payload
        let transfer_size = self.device_handle.read_bulk(
            self.endpoint_identifier.cmd_rx_endpoint,
            rx_data,
            self.timeout_duration,
        )?;

        let mut response = UsbCommandResponse::new();

        // Get the response status
        self.device_handle.read_bulk(
            self.endpoint_identifier.cmd_rx_endpoint,
            response.as_mut_bytes(),
            self.timeout_duration,
        )?;

        let response_tx_id = response.packet_transaction_id;
        let response_packet_type = response.packet_type;
        assert_eq!(response_tx_id, transaction_id);
        assert_eq!(response_packet_type, 0x0A6FE000);

        Ok((transfer_size, response.status))
    }

    pub fn write(
        &mut self,
        cmd_code: u32,
        cmd_data: Option<&[u8]>,
        tx_data: &[u8],
    ) -> Result<(usize, UsbResult), Error> {
        let transaction_id = self.transaction_id;
        self.transaction_id = transaction_id + 1;

        // Construct the command packet (containing a header and the cmd_data in a contiguous block)
        let packet = UsbcommandPacket::new(cmd_code, transaction_id, cmd_data);

        // Send the command
        self.device_handle.write_bulk(
            self.endpoint_identifier.cmd_tx_endpoint,
            packet.as_bytes(),
            self.timeout_duration,
        )?;

        // Write the payload
        let transfer_size = self.device_handle.write_bulk(
            self.endpoint_identifier.cmd_tx_endpoint,
            tx_data,
            self.timeout_duration,
        )?;

        let mut response = UsbCommandResponse::new();

        // Get the response status
        self.device_handle.read_bulk(
            self.endpoint_identifier.cmd_rx_endpoint,
            response.as_mut_bytes(),
            self.timeout_duration,
        )?;

        let response_tx_id = response.packet_transaction_id;
        let response_packet_type = response.packet_type;
        assert_eq!(response_tx_id, transaction_id);
        assert_eq!(response_packet_type, 0x0A6FE000);

        Ok((transfer_size, response.status))
    }
}
