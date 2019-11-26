/// Rounds a value up to an increment of size
///
/// # Example
/// Round a value up to the nearest 512
/// ```
/// let x = round_up!(y, 512);
/// ```
macro_rules! round_up {
    ($input:expr, $size:literal) => {
        ((($input) + (($size) - 1)) & !(($size) - 1))
    };
}

pub struct PayloadSize {
    pub meaningful_size: usize,
    pub padded_size: usize,
}

macro_rules! payload_size {
    ($input:expr) => {
        PayloadSize {
            meaningful_size: $input,
            padded_size: round_up!($input, 1024),
        }
    };
}

#[derive(Copy, Clone)]
pub enum SensorMode {
    PseudoCommon,
    LongThrowNative,
    MegaPixel,
    QuarterMegaPixel,
}

impl SensorMode {
    pub fn payload_size(&self) -> PayloadSize {
        match self {
            SensorMode::PseudoCommon => payload_size!(1678024),
            SensorMode::LongThrowNative => payload_size!(5310760),
            SensorMode::MegaPixel => payload_size!(9438664),
            SensorMode::QuarterMegaPixel => payload_size!(3777232),
        }
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        match self {
            SensorMode::PseudoCommon => 3u32.to_le_bytes(),
            SensorMode::LongThrowNative => 4u32.to_le_bytes(),
            SensorMode::MegaPixel => 5u32.to_le_bytes(),
            SensorMode::QuarterMegaPixel => 7u32.to_le_bytes(),
        }
    }
}

pub const CALIBRATION_DATA_SIZE: u32 = 2000000;

pub enum FPS {
    Fps5,
    Fps15,
    Fps30,
}

impl FPS {
    pub fn as_bytes(&self) -> [u8; 4] {
        match self {
            FPS::Fps5 => 5u32.to_le_bytes(),
            FPS::Fps15 => 15u32.to_le_bytes(),
            FPS::Fps30 => 30u32.to_le_bytes(),
        }
    }
}

pub struct PackageInfo {
    last_package: u8,
    package_size: u8,
}

pub enum DeviceCommands {
    Reset,
    VersionGet,
    DepthStart,
    DepthStop,
    NVDataGet,
    DepthModeSet,
    DepthPowerOff,
    DepthPowerOn,
    DepthStreamStart,
    DepthStreamStop,
    DepthFPSSet,
    DepthReadCalibrationData,
    DepthReadProductSN,
    ComponentVersionGet,
    DownloadFirmware,
    GetFirmwareUpdateStatus,
}

impl DeviceCommands {
    pub fn command_code(&self) -> u32 {
        match self {
            DeviceCommands::Reset => 0x00000000,
            DeviceCommands::VersionGet => 0x00000002,
            DeviceCommands::DepthStart => 0x00000009,
            DeviceCommands::DepthStop => 0x0000000A,
            DeviceCommands::NVDataGet => 0x00000022,
            DeviceCommands::DepthModeSet => 0x000000E1,
            DeviceCommands::DepthPowerOff => 0x000000EF,
            DeviceCommands::DepthPowerOn => 0x000000F0,
            DeviceCommands::DepthStreamStart => 0x000000F1,
            DeviceCommands::DepthStreamStop => 0x000000F2,
            DeviceCommands::DepthFPSSet => 0x00000103,
            DeviceCommands::DepthReadCalibrationData => 0x00000111,
            DeviceCommands::DepthReadProductSN => 0x00000115,
            DeviceCommands::ComponentVersionGet => 0x00000201,
            DeviceCommands::DownloadFirmware => 0x00000202,
            DeviceCommands::GetFirmwareUpdateStatus => 0x00000203,
        }
    }
}

pub enum NvTag {
    NoData,
    IRSensorCalibration,
}

impl NvTag {
    pub fn as_bytes(&self) -> [u8; 4] {
        match self {
            NvTag::NoData => 0u32.to_le_bytes(),
            NvTag::IRSensorCalibration => 2u32.to_le_bytes(),
        }
    }
}

#[repr(C, packed)]
pub struct Version {
    major: u8,
    minor: u8,
    build: u16,
}

#[repr(C, packed)]
pub struct FirmwareVersions {
    rgb: Version,
    depth: Version,
    audio: Version,
    depth_sensor_cfg_major: u16,
    depth_sensor_cfg_minor: u16,
    build_config: u8,
    signature_type: u8,
}

impl FirmwareVersions {
    pub fn new() -> Self {
        FirmwareVersions {
            rgb: Version {
                major: 0,
                minor: 0,
                build: 0,
            },
            depth: Version {
                major: 0,
                minor: 0,
                build: 0,
            },
            audio: Version {
                major: 0,
                minor: 0,
                build: 0,
            },
            depth_sensor_cfg_major: 0,
            depth_sensor_cfg_minor: 0,
            build_config: 0,
            signature_type: 0,
        }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            ::std::slice::from_raw_parts_mut(
                (self as *mut Self) as *mut u8,
                ::std::mem::size_of::<Self>(),
            )
        }
    }
}
