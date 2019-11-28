#![allow(dead_code)]

use rusb::UsbContext;

use std::fmt::Debug;

pub mod error;
mod protocol;
pub use error::*;

pub use protocol::UsbResult;

use protocol::{EndpointIdentifier, UsbCommandResponse, UsbcommandPacket};
use std::sync::{Arc, Mutex};
use std::convert::TryInto;

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

        let transferred = cmd.read(0x00000115, Option::None, &mut snbuffer).unwrap();

        let sn_from_cmd = String::from_utf8(snbuffer.to_vec()).unwrap();
        let sn_from_descriptor = cmd.serial_number();

        println!("transferred: {}", transferred);
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
    device_handle: Arc<Mutex<rusb::DeviceHandle<rusb::Context>>>,
    serial_number: String,
    timeout_duration: std::time::Duration,
    transaction_id: u32,
    streaming_thread: Option<(std::sync::mpsc::Sender<()>, std::thread::JoinHandle<()>)>,
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
                return Err(Error::NoDevice);
            }
        };

        let timeout = std::time::Duration::new(2, 0);
        let mut handle = device.open()?;

        let serial_number_string_index = device
            .device_descriptor()?
            .serial_number_string_index()
            .unwrap();

        let language = *match handle.read_languages(timeout)?.first() {
            Option::Some(x) => x,
            Option::None => {
                println!("Failed to find language");
                panic!("Device does not have language descriptor");
            }
        };

        let serial_number =
            handle.read_string_descriptor(language, serial_number_string_index, timeout)?;

        if handle.active_configuration()? != 1 {
            handle.set_active_configuration(1)?;
        }

        // Ignoring errors, detach the kernel driver if it is active.
        if let Ok(active) = handle.kernel_driver_active(endpoint_identifier.interface) {
            if active == true {
                handle.detach_kernel_driver(endpoint_identifier.interface)?;
            }
        }

        handle.claim_interface(endpoint_identifier.interface)?;

        Ok(Self {
            endpoint_identifier: endpoint_identifier,
            device_handle: Arc::new(Mutex::new(handle)),
            serial_number: serial_number,
            timeout_duration: timeout,
            transaction_id: 0,
            streaming_thread: Option::None,
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
    ) -> Result<usize, Error> {
        let transaction_id = self.transaction_id;
        self.transaction_id = transaction_id + 1;

        // Construct the command packet (containing a header and the cmd_data in a contiguous block)
        let packet = UsbcommandPacket::new(cmd_code, transaction_id, cmd_data, rx_data.len().try_into().unwrap());

        let device = self.device_handle.lock().unwrap();

        // Send the command
        let command_bytes_written = device.write_bulk(
            self.endpoint_identifier.cmd_tx_endpoint,
            packet.as_bytes(),
            self.timeout_duration,
        )?;

        assert!(command_bytes_written == packet.as_bytes().len());

        // Read the payload
        let transfer_size = device.read_bulk(
            self.endpoint_identifier.cmd_rx_endpoint,
            rx_data,
            self.timeout_duration,
        )?;

        assert!(transfer_size <= rx_data.len());

        let mut response = UsbCommandResponse::new();

        // Get the response status
        let response_size = device.read_bulk(
            self.endpoint_identifier.cmd_rx_endpoint,
            response.as_mut_bytes(),
            self.timeout_duration,
        )?;

        let response_tx_id = response.packet_transaction_id;
        let response_packet_type = response.packet_type;

        if response_size != response.as_mut_bytes().len() {
            return Err(Error::Protocol(ProtocolError::ResponseSizeMismatch(
                Mismatch::new(response.as_mut_bytes().len(), response_size),
            )));
        }

        if response_tx_id != transaction_id {
            return Err(Error::Protocol(ProtocolError::TransactionIdMismatch(
                Mismatch::new(transaction_id, response_tx_id),
            )));
        }

        if response_packet_type != protocol::RESPONSE_PACKET_TYPE {
            return Err(Error::Protocol(ProtocolError::PacketTypeMismatch(
                Mismatch::new(protocol::RESPONSE_PACKET_TYPE, response_packet_type),
            )));
        }

        let status = response.status;
        if status != protocol::USB_RESULT_OK {
            let err = Error::Firmware(status);
            return Err(err);
        }

        Ok(transfer_size)
    }

    pub fn write(
        &mut self,
        cmd_code: u32,
        cmd_data: Option<&[u8]>,
        tx_data: &[u8],
    ) -> Result<usize, Error> {
        let transaction_id = self.transaction_id;
        self.transaction_id = transaction_id + 1;

        // Construct the command packet (containing a header and the cmd_data in a contiguous block)
        let packet = UsbcommandPacket::new(cmd_code, transaction_id, cmd_data, tx_data.len().try_into().unwrap());

        let device = self.device_handle.lock().unwrap();

        // Send the command
        let command_bytes_written = device.write_bulk(
            self.endpoint_identifier.cmd_tx_endpoint,
            packet.as_bytes(),
            self.timeout_duration,
        )?;

        assert!(command_bytes_written == packet.as_bytes().len());

        // Write the payload
        let transfer_size;
        // Firmware does not expect a zero-length packet
        if tx_data.len() > 0 {
            transfer_size = device.write_bulk(
                self.endpoint_identifier.cmd_tx_endpoint,
                tx_data,
                self.timeout_duration,
            )?;
        } else {
            transfer_size = 0;
        }

        assert!(transfer_size <= tx_data.len());

        let mut response = UsbCommandResponse::new();

        // Get the response status
        let response_size = device.read_bulk(
            self.endpoint_identifier.cmd_rx_endpoint,
            response.as_mut_bytes(),
            self.timeout_duration,
        )?;

        let response_tx_id = response.packet_transaction_id;
        let response_packet_type = response.packet_type;

        if response_size != response.as_mut_bytes().len() {
            return Err(Error::Protocol(ProtocolError::ResponseSizeMismatch(
                Mismatch::new(response.as_mut_bytes().len(), response_size),
            )));
        }

        if response_tx_id != transaction_id {
            return Err(Error::Protocol(ProtocolError::TransactionIdMismatch(
                Mismatch::new(transaction_id, response_tx_id),
            )));
        }

        if response_packet_type != protocol::RESPONSE_PACKET_TYPE {
            return Err(Error::Protocol(ProtocolError::PacketTypeMismatch(
                Mismatch::new(protocol::RESPONSE_PACKET_TYPE, response_packet_type),
            )));
        }

        let status = response.status;
        if status != protocol::USB_RESULT_OK {
            let err = Error::Firmware(status);
            return Err(err);
        }

        Ok(transfer_size)
    }

    pub fn stream_start(&mut self, payload_size: usize) -> Result<(), Error> {
        let timeout = self.timeout_duration;
        let endpoint = self.endpoint_identifier.stream_endpoint;

        let handle = self.device_handle.clone();

        let (tx, rx) = std::sync::mpsc::channel::<()>();

        let join_handle = std::thread::spawn(move || {
            let mut buffer = vec![0; payload_size];

            loop {
                if rx.try_recv().is_ok() {
                    return ();
                }

                {
                    let device = handle.lock().unwrap();
                    println!("Reading from stream");
                    match device.read_bulk(endpoint, &mut buffer, timeout) {
                        Result::Ok(x) => println!("Data! {}", x),
                        Result::Err(e) => println!("Error! {}", e),
                    }
                }
            }
        });

        self.streaming_thread = Option::Some((tx, join_handle));

        Ok(())
    }

    pub fn stream_stop(&mut self) -> Result<(), Error> {
        if let Some((tx, join_handle)) = self.streaming_thread.take() {
            // Signal the thread to exit
            tx.send(()).unwrap();

            // Wait for it to complete
            join_handle.join().unwrap();
        }

        Ok(())
    }
}
