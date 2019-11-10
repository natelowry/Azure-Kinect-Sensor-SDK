
macro_rules! round_up {
    ($input:expr) => {
        ((($input) + ((1024)-1)) & !((1024)- 1))
    }
}

pub struct PayloadSize
{
    pub meaningful_size: usize,
    pub padded_size: usize,
}

macro_rules! payload_size {
    ($input:expr) => {
        PayloadSize {
            meaningful_size: $input,
            padded_size: round_up!($input)
        }
    }
}

const SensorModeLongThrowNativeSize : usize = 5310760;
const SensorModeLongThrowNativeSizePadded : usize = round_up!(SensorModeLongThrowNativeSize);

pub const SensorModeLongThrowNative : PayloadSize = payload_size!(5310760);

pub enum SensorMode
{
    PseudoCommon = 3,
    LongThrowNative = 4,
    MegaPixel = 5,
    QuarterMegaPixel = 7,

}
