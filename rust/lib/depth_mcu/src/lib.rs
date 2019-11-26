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

pub use protocol::FPS;

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

struct DepthMcuCommon {
    device: usbcommand::Usbcommand,
}

pub struct DepthMcu {
    common: DepthMcuCommon,
}

pub struct PoweredDepthMcu {
    common: DepthMcuCommon,
    mode: protocol::SensorMode,
}

pub struct StreamingDepthMcu {
    common: Option<DepthMcuCommon>,
    mode: protocol::SensorMode,
}


impl DepthMcu {
    pub fn new(device: usbcommand::Usbcommand) -> Self {
        Self { common: DepthMcuCommon{ device: device } }
    }

    /// Set the capture mode of the device, powering on the depth sensor
    ///
    /// This operation powers on the depth sensor, enabling new functionality
    /// provided by the PoweredDepthMcu type.
    ///
    /// A failure in this method leaves the firmware in an indeterministic state
    /// so the DepthMcu object can no longer safely be re-used.
    pub fn set_capture_mode(
        mut self,
        mode: CaptureMode,
    ) -> Result<PoweredDepthMcu, usbcommand::Error> {
        let command = protocol::DeviceCommands::DepthModeSet;
        let sensor_mode = mode.sensor_mode();
        let command_argument = sensor_mode.as_bytes();

        self.common.device
            .write(command.command_code(), Option::Some(&command_argument), &[])?;

        Ok(PoweredDepthMcu {
            common: self.common,
            mode: sensor_mode,
        })
    }
}

impl PoweredDepthMcu {
    /// Gets the sensor calibration from the device
    ///
    /// This operation can only be performed on a depth device that has been powered on.
    ///
    /// A failure in this method leaves the firmware in an indeterministic state
    /// so the PoweredDepthMcu object can no longer safely be re-used.
    pub fn calibration(&mut self) -> Result<std::vec::Vec<u8>, usbcommand::Error> {
        let command = protocol::DeviceCommands::NVDataGet;

        // Allocate a buffer larger than the total possible calibration size
        let mut cal_buffer = vec![0; 2000000];

        let transferred = self.common.device.read(
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
    ) -> Result<StreamingDepthMcu, usbcommand::Error> {
        self.common.device.write(
            protocol::DeviceCommands::DepthFPSSet.command_code(),
            Option::Some(&fps.as_bytes()),
            &[],
        )?;

        self.common.device.write(
            protocol::DeviceCommands::DepthStart.command_code(),
            Option::None,
            &[],
        )?;

        self.common.device.write(
            protocol::DeviceCommands::DepthStreamStart.command_code(),
            Option::None,
            &[],
        )?;

        
        let payload_size = self.mode.payload_size().padded_size;

        self.common.device.stream_start(payload_size)?;

        Ok(StreamingDepthMcu {
                common: Option::Some(self.common),
                mode: self.mode,
        }
            )
    }
}

impl StreamingDepthMcu {

    fn stop_streaming_internal(&mut self) -> Result<(), usbcommand::Error> {
            if let Option::Some(common) = self.common.as_mut() {
                let device = &mut common.device;

                device.stream_stop()?;
            
                device.write(
                    protocol::DeviceCommands::DepthStreamStop.command_code(),
                    Option::None,
                    &[],
                )?;

                device.write(
                    protocol::DeviceCommands::DepthStop.command_code(),
                    Option::None,
                    &[],
                )?;
            }
            Ok(())
    }
    pub fn stop_streaming(mut self) -> Result<PoweredDepthMcu, usbcommand::Error> {

        self.stop_streaming_internal()?;

        return Ok(PoweredDepthMcu {
            common: self.common.take().unwrap(),
            mode: self.mode,
        });
        
    }
}

impl Drop for StreamingDepthMcu {
    fn drop(&mut self) {
        let _ = self.stop_streaming_internal();
    }
}

impl DepthMcuCommonOperations for DepthMcu {
    fn serialnum(&mut self) -> Result<String, usbcommand::Error>
    {
        self.common.serialnum()
    }

    fn wait_is_ready(&mut self) -> Result<(), usbcommand::Error>
    {
        self.common.wait_is_ready()
    }

    fn version(&mut self) -> Result<protocol::FirmwareVersions, usbcommand::Error>
    {
        self.common.version()
    }

    fn extrinsic_calibration(&mut self) -> Result<String, usbcommand::Error>
    {
        self.common.extrinsic_calibration()
    }
}

impl DepthMcuCommonOperations for PoweredDepthMcu {
    fn serialnum(&mut self) -> Result<String, usbcommand::Error>
    {
        self.common.serialnum()
    }

    fn wait_is_ready(&mut self) -> Result<(), usbcommand::Error>
    {
        self.common.wait_is_ready()
    }

    fn version(&mut self) -> Result<protocol::FirmwareVersions, usbcommand::Error>
    {
        self.common.version()
    }

    fn extrinsic_calibration(&mut self) -> Result<String, usbcommand::Error>
    {
        self.common.extrinsic_calibration()
    }
}

impl DepthMcuCommonOperations for StreamingDepthMcu {
    fn serialnum(&mut self) -> Result<String, usbcommand::Error>
    {
        self.common.as_mut().unwrap().serialnum()
    }

    fn wait_is_ready(&mut self) -> Result<(), usbcommand::Error>
    {
        self.common.as_mut().unwrap().wait_is_ready()
    }

    fn version(&mut self) -> Result<protocol::FirmwareVersions, usbcommand::Error>
    {
        self.common.as_mut().unwrap().version()
    }

    fn extrinsic_calibration(&mut self) -> Result<String, usbcommand::Error>
    {
        self.common.as_mut().unwrap().extrinsic_calibration()
    }
}

impl DepthMcuCommon {
    fn serialnum(&mut self) -> Result<String, usbcommand::Error> {
        let mut snbuffer: [u8; 128] = [0; 128];

        let command = protocol::DeviceCommands::DepthReadProductSN;

        let transferred =
            self.device
                .read(command.command_code(), Option::None, &mut snbuffer)?;

        let slice = &snbuffer[0..transferred];
        let vec = slice.to_vec();

        Ok(String::from_utf8(vec)?)
    }

    fn wait_is_ready(&mut self) -> Result<(), usbcommand::Error> {
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

    fn version(&mut self) -> Result<protocol::FirmwareVersions, usbcommand::Error> {
        let mut fwversions = protocol::FirmwareVersions::new();
        let command = protocol::DeviceCommands::ComponentVersionGet;
        let buffer = fwversions.as_mut_bytes();

        let transferred = self
            .device
            .read(command.command_code(), Option::None, buffer)?;

        assert_eq!(transferred, buffer.len());

        return Ok(fwversions);
    }

    fn extrinsic_calibration(&mut self) -> Result<String, usbcommand::Error> {
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

pub trait DepthMcuCommonOperations {
    fn serialnum(&mut self) -> Result<String, usbcommand::Error>;

    fn wait_is_ready(&mut self) -> Result<(), usbcommand::Error>;

    fn version(&mut self) -> Result<protocol::FirmwareVersions, usbcommand::Error>;

    fn extrinsic_calibration(&mut self) -> Result<String, usbcommand::Error>;
}
