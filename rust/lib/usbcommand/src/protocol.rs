#[derive(Debug)]
pub struct UsbResult(pub u32);

impl std::ops::Deref for UsbResult {
    type Target = u32;

    fn deref(&self) -> &<Self as std::ops::Deref>::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct EndpointIdentifier {
    pub vid: u16,
    pub pid: u16,
    pub interface: u8,
    pub cmd_tx_endpoint: u8,
    pub cmd_rx_endpoint: u8,
    pub stream_endpoint: u8
}

pub const DEPTH_ENDPOINT_IDENTIFIER: EndpointIdentifier = EndpointIdentifier {
    vid: 0x045e,
    pid: 0x097c,
    interface: 0,
    cmd_tx_endpoint: 0x02,
    cmd_rx_endpoint: 0x81,
    stream_endpoint: 0x83
};

pub const COLOR_ENDPOINT_IDENTIFIER: EndpointIdentifier = EndpointIdentifier {
    vid: 0x045e,
    pid: 0x097d,
    interface: 2,
    cmd_tx_endpoint: 0x04,
    cmd_rx_endpoint: 0x83,
    stream_endpoint: 0x82
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