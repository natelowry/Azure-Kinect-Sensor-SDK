#![allow(dead_code)]

use rusb::UsbContext;

use std::fmt::Debug;
use std::fmt::Display;

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

        let mut snbuffer : [u8; 128] = [0; 128];
        
        let (transferred, result) = cmd.read(0x00000115, Option::None, &mut snbuffer).unwrap();

        let sn_from_cmd = String::from_utf8(snbuffer.to_vec()).unwrap();
        let sn_from_descriptor = cmd.serial_number();

        println!("transferred: {}", transferred);
        println!("result: {:?}", result);
        println!("SN from command       : {}", sn_from_cmd);
        println!("SN from usb descriptor: {}", sn_from_descriptor);

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

impl std::fmt::Debug for Usbcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Usbcommand ({:?})", self.endpoint_identifier)
    }
}

pub enum DeviceType {
    DepthProcessor,
    ColorImuProcessor,
}

#[derive(Debug)]
pub struct UsbResult(u32);

/// Header structure in USB commands
#[repr(C, packed)]
struct UsbcommandHeader {
    packet_type: u32,
    packet_transaction_id: u32,
    payload_size: u32,
    command: u32,
    reserved: u32,
}

// A structure that packs the header and data in contiguous memory.
#[repr(C, packed)]
struct UsbcommandPacket {
    header: UsbcommandHeader
}

impl UsbcommandPacket {
    fn new<T>(command: u32, tx_id: u32, data: Option<T>) -> UsbcommandPacket {
        let data_size = std::mem::size_of::<T>() as u32;

        match data {
                Option::Some(_x) => panic!("not expected"),
                Option::None => [0; 0]
            };

        UsbcommandPacket {
            header: UsbcommandHeader {
                packet_type: 0x06022009,
                packet_transaction_id: tx_id,
                payload_size: data_size,
                command: command,
                reserved: 0,
            }
        }
    }

    fn as_slice(&self) -> &[u8] {

        unsafe {
            let buffer = (self as *const UsbcommandPacket) as *const u8;
            let size = ::std::mem::size_of::<UsbcommandPacket>();

            ::std::slice::from_raw_parts(
                buffer,
                size,
            )
        }
    }
}

/// Response structure in USB commands
#[repr(C, packed)]
struct UsbCommandResponse {
    packet_type: u32,
    packet_transaction_id: u32,
    status: UsbResult,
    reserved: u32,
}

impl UsbCommandResponse {
    fn new() -> Self {
        UsbCommandResponse{
            packet_type: 0,
            packet_transaction_id: 0,
            status: UsbResult(0),
            reserved: 0,
        }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            ::std::slice::from_raw_parts_mut(
                (self as *mut UsbCommandResponse) as *mut u8,
                ::std::mem::size_of::<UsbCommandResponse>()
            )
        }
    }
}


#[derive(Debug)]
pub enum Error {
    NoDevice,
    Fail,
    Access,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "A Usbcommand error occurred")
    }
}

impl std::convert::From<rusb::Error> for Error {
    fn from(t: rusb::Error) -> Self {
        println!("Converted usb error: {}", t);
        match t {
            rusb::Error::Success => Error::Fail,
            rusb::Error::Io => Error::Fail,
            rusb::Error::InvalidParam => panic!("Unexpected usb error"),
            rusb::Error::Access => Error::Access,
            rusb::Error::NoDevice => Error::NoDevice,
            rusb::Error::NotFound => Error::Fail,
            rusb::Error::Busy => Error::Fail,
            rusb::Error::Timeout => Error::Fail,
            rusb::Error::Overflow => Error::Fail,
            rusb::Error::Pipe => Error::Fail,
            rusb::Error::Interrupted => panic!("Unexpected usb error: Interrupted"),
            rusb::Error::NoMem => panic!("Unexpected usb error: NoMem"),
            rusb::Error::NotSupported => panic!("Unexpected usb error: NotSupported"),
            rusb::Error::Other => panic!("Unexpected usb error: Other"),
        }
    }
}

/// A command pipeline to an Azure Kinect USB device is represented here
pub struct Usbcommand {
    endpoint_identifier: EndpointIdentifier,
    device_handle: rusb::DeviceHandle<rusb::Context>,
    serial_number: String,
    timeout_duration: std::time::Duration,
    transaction_id: u32,
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
        let endpoint_identifier = match device_type {
            DeviceType::DepthProcessor => DEPTH_ENDPOINT_IDENTIFIER,
            DeviceType::ColorImuProcessor => COLOR_ENDPOINT_IDENTIFIER,
        };

        let my_local_libusb_context: Box<rusb::Context> = Box::new(rusb::Context::new().unwrap());

        let device = my_local_libusb_context.devices()?.iter()
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
                return Err(Error::Fail)
            }
        };

        let serial_number =
            handle.read_string_descriptor(language, serial_number_string_index, timeout)?;

        if handle.active_configuration()? != 1 {
            handle.set_active_configuration(1)?;
        }

        match handle.kernel_driver_active(endpoint_identifier.interface)
        {
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
            packet.as_slice(),
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
            response.as_mut_slice(),
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
            packet.as_slice(),
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
            response.as_mut_slice(),
            self.timeout_duration,
        )?;

        let response_tx_id = response.packet_transaction_id;
        let response_packet_type = response.packet_type;
        assert_eq!(response_tx_id, transaction_id);
        assert_eq!(response_packet_type, 0x0A6FE000);
        
        Ok((transfer_size, response.status))
    }
}
