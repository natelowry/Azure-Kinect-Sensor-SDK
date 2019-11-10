#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

mod protocol;


pub struct DepthMcu {
    device: usbcommand::Usbcommand,
}

impl DepthMcu {
    pub fn new(device: usbcommand::Usbcommand) -> Self {
        Self {
            device: device,
        }
    }

    pub fn serialnum(&self) -> Result<String, usbcommand::Error> {
        //self.device.read(cmd_code: u32, cmd_data: Option<&[u8]>, rx_data: &mut [u8])
        Err(usbcommand::Error::Fail)
    }
}