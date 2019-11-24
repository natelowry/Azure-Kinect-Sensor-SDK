#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

mod protocol;

pub enum CaptureMode {
    Nfov2x2Binned,
    NfovUnbinned,
    Wfov2x2Binned,
    WfovUnbinned,
    PassiveIR,
}

impl CaptureMode {
    pub fn sensor_mode(&self) -> protocol::SensorMode {
        match self {
            CaptureMode::Nfov2x2Binned => protocol::SensorMode::LongThrowNative,
            CaptureMode::NfovUnbinned => protocol::SensorMode::LongThrowNative,
            CaptureMode::Wfov2x2Binned => protocol::SensorMode::QuarterMegaPixel,
            CaptureMode::WfovUnbinned => protocol::SensorMode::MegaPixel,
            CaptureMode::PassiveIR => protocol::SensorMode::PseudoCommon,
        }
    }
}

enum Mode {
    Off,
    On(protocol::SensorMode),
}

pub struct DepthMcu {
    device: usbcommand::Usbcommand,
    mode: Mode,
}


impl DepthMcu {
    pub fn new(device: usbcommand::Usbcommand) -> Self {
        Self {
            device: device,
            mode: Mode::Off,
        }
    }

    pub fn serialnum(&mut self) -> Result<String, usbcommand::Error> {
        let mut snbuffer: [u8; 128] = [0; 128];

        let command = protocol::DeviceCommands::DepthReadProductSN;

        let transferred = self
            .device
            .read(command.command_code(), Option::None, &mut snbuffer)?;

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

        Err(usbcommand::Error::Timeout)
    }

    pub fn version(&mut self) -> Result<protocol::FirmwareVersions, usbcommand::Error> {
        let mut fwversions = protocol::FirmwareVersions::new();
        let command = protocol::DeviceCommands::ComponentVersionGet;
        let buffer = fwversions.as_mut_bytes();

        let transferred = self
            .device
            .read(command.command_code(), Option::None, buffer)?;

        assert_eq!(transferred, buffer.len());

        return Ok(fwversions);
    }

    pub fn set_capture_mode(&mut self, mode: CaptureMode) -> Result<(), usbcommand::Error> {
        let command = protocol::DeviceCommands::DepthModeSet;
        let sensor_mode = mode.sensor_mode();
        let command_argument = sensor_mode.as_bytes();

        self.mode = Mode::On(sensor_mode);

        let transferred =
            self.device
                .write(command.command_code(), Option::Some(&command_argument), &[])?;

        assert_eq!(transferred, 0);

        Ok(())
    }

    pub fn set_fps(&mut self, fps: protocol::FPS) -> Result<(), usbcommand::Error> {
        let command = protocol::DeviceCommands::DepthFPSSet;

        let transferred =
            self.device
                .write(command.command_code(), Option::Some(&fps.as_bytes()), &[])?;

        assert_eq!(transferred, 0);

        Ok(())
    }
    
    pub fn calibration<'a>(&mut self, buffer: &'a mut [u8]) -> Result<&'a [u8], usbcommand::Error> {

        let command = protocol::DeviceCommands::DepthReadCalibrationData;


        let transferred =
            self.device
                .read(command.command_code(), Option::Some(&protocol::NvTag::IRSensorCalibration.as_bytes()), buffer)?;

        Ok(&buffer[0..transferred])
    }
}
