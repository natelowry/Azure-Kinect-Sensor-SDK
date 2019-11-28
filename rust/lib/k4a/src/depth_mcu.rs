mod protocol;

pub use protocol::FPS;

use crate::usbcommand;

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

pub struct DepthMcu<S> {
    device: usbcommand::Usbcommand,
    state: S,
}

pub struct Off {}

pub struct Powered {
    mode: protocol::SensorMode,
}

pub struct Streaming {
    mode: protocol::SensorMode,
}

impl DepthMcu<Off> {
    pub fn new(device: usbcommand::Usbcommand) -> Self {
        Self {
            device: device,
            state: Off {},
        }
    }

    /// Set the capture mode of the device, powering on the depth sensor
    ///
    /// This operation powers on the depth sensor, enabling new functionality
    /// provided by the PoweredDepthMcu type.
    ///
    /// A failure in this method leaves the firmware in an nondeterministic state
    /// so the DepthMcu object can no longer safely be re-used.
    pub fn set_capture_mode(
        mut self,
        mode: CaptureMode,
    ) -> Result<DepthMcu<Powered>, usbcommand::Error> {
        let command = protocol::DeviceCommands::DepthModeSet;
        let sensor_mode = mode.sensor_mode();
        let command_argument = sensor_mode.as_bytes();

        self.device
            .write(command.command_code(), Option::Some(&command_argument), &[])?;

        Ok(DepthMcu {
            device: self.device,
            state: Powered { mode: sensor_mode },
        })
    }
}

impl DepthMcu<Powered> {
    /// Gets the sensor calibration from the device
    ///
    /// This operation can only be performed on a depth device that has been powered on.
    ///
    /// A failure in this method leaves the firmware in an nondeterministic state
    /// so the PoweredDepthMcu object can no longer safely be re-used.
    pub fn calibration(&mut self) -> Result<std::vec::Vec<u8>, usbcommand::Error> {
        let command = protocol::DeviceCommands::NVDataGet;

        // Allocate a buffer larger than the total possible calibration size
        let mut cal_buffer = vec![0; 2000000];

        let transferred = self.device.read(
            command.command_code(),
            Option::Some(&protocol::NvTag::IRSensorCalibration.as_bytes()),
            &mut cal_buffer,
        )?;

        // Trim the buffer to the actual size received, and free excess memory
        cal_buffer.truncate(transferred);
        cal_buffer.shrink_to_fit();

        Ok(cal_buffer)
    }

    pub fn start_streaming(
        mut self,
        fps: protocol::FPS,
    ) -> Result<DepthMcu<Streaming>, usbcommand::Error> {
        self.device.write(
            protocol::DeviceCommands::DepthFPSSet.command_code(),
            Option::Some(&fps.as_bytes()),
            &[],
        )?;

        self.device.write(
            protocol::DeviceCommands::DepthStart.command_code(),
            Option::None,
            &[],
        )?;

        self.device.write(
            protocol::DeviceCommands::DepthStreamStart.command_code(),
            Option::None,
            &[],
        )?;

        let payload_size = self.state.mode.payload_size().padded_size;

        self.device.stream_start(payload_size, |mut buffer| {
            println!("Read buffer. Size: {}", (*buffer).as_mut().len())
        })?;

        Ok(DepthMcu {
            device: self.device,
            state: Streaming {
                mode: self.state.mode,
            },
        })
    }
}

impl DepthMcu<Streaming> {
    pub fn stop_streaming(mut self) -> Result<DepthMcu<Powered>, usbcommand::Error> {
        self.device.stream_stop()?;

        self.device.write(
            protocol::DeviceCommands::DepthStreamStop.command_code(),
            Option::None,
            &[],
        )?;

        self.device.write(
            protocol::DeviceCommands::DepthStop.command_code(),
            Option::None,
            &[],
        )?;

        return Ok(DepthMcu {
            device: self.device,
            state: Powered {
                mode: self.state.mode,
            },
        });
    }
}

impl<T> DepthMcu<T> {
    pub fn serialnum(&mut self) -> Result<String, usbcommand::Error> {
        let mut serial_number_buffer: [u8; 128] = [0; 128];

        let command = protocol::DeviceCommands::DepthReadProductSN;

        let transferred = self.device.read(
            command.command_code(),
            Option::None,
            &mut serial_number_buffer,
        )?;

        let slice = &serial_number_buffer[0..transferred];
        let vec = slice.to_vec();

        Ok(String::from_utf8(vec)?)
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
        let mut versions = protocol::FirmwareVersions::new();
        let buffer = versions.as_mut_bytes();

        let transferred = self.device.read(
            protocol::DeviceCommands::ComponentVersionGet.command_code(),
            Option::None,
            buffer,
        )?;

        if transferred != buffer.len() {
            return Err(usbcommand::Error::Protocol(
                usbcommand::error::ProtocolError::ResponseSizeMismatch(
                    usbcommand::error::Mismatch::new(buffer.len(), transferred),
                ),
            ));
        }

        return Ok(versions);
    }

    pub fn extrinsic_calibration(&mut self) -> Result<String, usbcommand::Error> {
        // Over allocate a full megabyte
        let mut cal_buffer = vec![0; 1024 * 1024];

        let transferred = self.device.read(
            protocol::DeviceCommands::DepthReadCalibrationData.command_code(),
            Option::None,
            &mut cal_buffer,
        )?;

        // Trim to the size received, plus an extra character for NULL termination
        cal_buffer.truncate(transferred + 1);
        cal_buffer.shrink_to_fit();

        // Convert the results to a String
        Ok(String::from_utf8(cal_buffer)?)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
