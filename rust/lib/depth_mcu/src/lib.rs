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
        Self { device: device }
    }

    pub fn serialnum(&mut self) -> Result<String, usbcommand::Error> {
        let mut snbuffer: [u8; 128] = [0; 128];

        let command = protocol::DeviceCommands::DepthReadProductSN;

        let (transferred, result) = self
            .device
            .read(command.command_code(), Option::None, &mut snbuffer)
            .unwrap();

        if result.0 != 0 {
            return Err(usbcommand::Error::Fail);
        }

        let slice = &snbuffer[0..transferred];
        let vec = slice.to_vec();
        Ok(String::from_utf8(vec).unwrap())
    }

    pub fn wait_is_ready(&mut self) -> Result<(), usbcommand::Error> {
        let mut retries = 20;
        while retries > 0 {
            let result = self.version();

            if result.is_ok() {
                return Ok(());
            }

            retries = retries - 1;
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        Err(usbcommand::Error::Fail)
    }

    pub fn version(&mut self) -> Result<protocol::FirmwareVersions, usbcommand::Error> {
        let mut fwversions = protocol::FirmwareVersions::new();
        let command = protocol::DeviceCommands::ComponentVersionGet;
        let buffer = fwversions.as_mut_bytes();

        let (transferred, result) =
            self.device
                .read(command.command_code(), Option::None, buffer)?;

        if result.0 == 0 && transferred == buffer.len() {
            return Ok(fwversions);
        }

        Err(usbcommand::Error::Fail)
    }
}
