pub struct UsbResult(pub u32);

pub const USB_RESULT_OK: UsbResult = UsbResult(0);

impl PartialEq for UsbResult {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Debug for UsbResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0x00000000 => write!(f, "UsbResult(DEV_CMD_STATUS_SUCCESS)"),
            0x00000001 => write!(f, "UsbResult(DEV_CMD_STATUS_ERROR)"),
            0x00000003 => write!(f, "UsbResult(DEV_CMD_STATUS_INVALID_PARAMETER)"),
            0x00000007 => write!(f, "UsbResult(DEV_CMD_STATUS_COMMAND_BUSY)"),
            0x00000008 => write!(f, "UsbResult(DEV_CMD_STATUS_NOT_IMPLEMENTED)"),
            0x00000009 => write!(f, "UsbResult(DEV_CMD_STATUS_OUT_OF_MEMORY)"),
            0x0000000D => write!(f, "UsbResult(DEV_CMD_STATUS_PARAM_BAD_TAG)"),
            0x00000012 => write!(f, "UsbResult(DEV_CMD_STATUS_INVALID_PAYLOAD_SIZE)"),
            0x00000063 => write!(f, "UsbResult(DEV_CMD_STATUS_FAILED)"),
            0x00000101 => write!(f, "UsbResult(DEV_CMD_STATUS_WRONG_COMMAND_STATE)"),
            0x00000102 => write!(f, "UsbResult(DEV_CMD_STATUS_WRONG_DEVICE_STATE)"),
            0x00000480 => write!(f, "UsbResult(DEV_CMD_STATUS_ADC_INVALID_CHANNEL)"),
            0x00000481 => write!(f, "UsbResult(DEV_CMD_STATUS_ADC_INCORRECT_CHANNEL)"),
            0x00000482 => write!(f, "UsbResult(DEV_CMD_STATUS_ADC_TIMEOUT)"),
            0x00000483 => write!(f, "UsbResult(DEV_CMD_STATUS_ADC_UNKNOWN_DEVICE)"),
            0x00000484 => write!(f, "UsbResult(DEV_CMD_STATUS_ADC_UNSUPPORTED_DEVICE)"),
            0x00000485 => write!(f, "UsbResult(DEV_CMD_STATUS_ADC_UNSUPPORTED_SIGNAL)"),
            0x00000486 => write!(f, "UsbResult(DEV_CMD_STATUS_ADC_INVALID_INPUT)"),
            0x00000487 => write!(f, "UsbResult(DEV_CMD_STATUS_ADC_DATA_NOT_AVAILABLE)"),
            x => write!(f, "UsbResult({})", x),
        }
    }
}

#[derive(Debug)]
pub struct EndpointIdentifier {
    pub vid: u16,
    pub pid: u16,
    pub interface: u8,
    pub cmd_tx_endpoint: u8,
    pub cmd_rx_endpoint: u8,
    pub stream_endpoint: u8,
}

pub const DEPTH_ENDPOINT_IDENTIFIER: EndpointIdentifier = EndpointIdentifier {
    vid: 0x045e,
    pid: 0x097c,
    interface: 0,
    cmd_tx_endpoint: 0x02,
    cmd_rx_endpoint: 0x81,
    stream_endpoint: 0x83,
};

pub const COLOR_ENDPOINT_IDENTIFIER: EndpointIdentifier = EndpointIdentifier {
    vid: 0x045e,
    pid: 0x097d,
    interface: 2,
    cmd_tx_endpoint: 0x04,
    cmd_rx_endpoint: 0x83,
    stream_endpoint: 0x82,
};

pub const REQUEST_PACKET_TYPE: u32 = 0x06022009;
pub const RESPONSE_PACKET_TYPE: u32 = 0x0A6FE000;
/// Header structure in USB commands
#[repr(C, packed)]
pub struct UsbcommandHeader {
    pub packet_type: u32,
    pub packet_transaction_id: u32,
    pub payload_size: u32,
    pub command: u32,
    reserved: u32,
}

// A structure that packs the header and data in contiguous memory.
#[repr(C, packed)]
pub struct UsbcommandPacket {
    header: UsbcommandHeader,
    data: [u8; 128],
}

impl UsbcommandPacket {
    pub fn new(command: u32, tx_id: u32, data: Option<&[u8]>) -> UsbcommandPacket {
        let data_size: u32;
        let mut payload: [u8; 128] = [0; 128];

        match data {
            Option::Some(x) => {
                let s = &mut payload[0..x.len()];
                s.copy_from_slice(x);
                data_size = x.len() as u32;
            }
            Option::None => {
                data_size = 0;
            }
        };

        UsbcommandPacket {
            header: UsbcommandHeader {
                packet_type: REQUEST_PACKET_TYPE,
                packet_transaction_id: tx_id,
                payload_size: data_size,
                command: command,
                reserved: 0,
            },
            data: payload,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let buffer = (self as *const UsbcommandPacket) as *const u8;
            let size =
                ::std::mem::size_of::<UsbcommandPacket>() + self.header.payload_size as usize;

            ::std::slice::from_raw_parts(buffer, size)
        }
    }
}

/// Response structure in USB commands
#[repr(C, packed)]
pub struct UsbCommandResponse {
    pub packet_type: u32,
    pub packet_transaction_id: u32,
    pub status: UsbResult,
    reserved: u32,
}

impl UsbCommandResponse {
    pub fn new() -> Self {
        UsbCommandResponse {
            packet_type: 0,
            packet_transaction_id: 0,
            status: UsbResult(0),
            reserved: 0,
        }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            ::std::slice::from_raw_parts_mut(
                (self as *mut UsbCommandResponse) as *mut u8,
                ::std::mem::size_of::<UsbCommandResponse>(),
            )
        }
    }
}
