use std::sync::Arc;

#[derive(Copy, Clone)]
pub enum ImageFormat {
    Default,
}

struct MutableImageState {
    device_timestamp_usec: u64,
    system_timestamp_nsec: u64,
    exposure_time_usec: u64,
    white_balance: u32,
    iso_speed: u32,
}

pub struct Image {
    mutable: MutableImageState,
    format: ImageFormat,
    width_pixels: i32,
    height_pixels: i32,
    stride_bytes: i32,
    buffer: Box<dyn Buffer>,
}

pub struct BufferWrapper<T> where
    T: AsMut<[u8]> + Send
{
    content: T,
}

pub trait Buffer : Send {
    fn as_mut(&mut self) -> &mut [u8];
}

impl<T> Buffer for BufferWrapper<T> where T: AsMut<[u8]> + Send {
    fn as_mut(&mut self) -> &mut [u8] {
        self.content.as_mut()
    }
}

impl<T> BufferWrapper<T> where
    T: AsMut<[u8]> + Send {
    pub fn new(content: T) -> BufferWrapper<T> {
        BufferWrapper { content: content }
    }
}

impl Image {
    pub fn new(
        format: ImageFormat,
        width_pixels: i32,
        height_pixels: i32,
        stride_bytes: i32,
        buffer: Box<dyn Buffer>,
    ) -> Self {
        Image {
            mutable: MutableImageState {
                device_timestamp_usec: 0,
                system_timestamp_nsec: 0,
                exposure_time_usec: 0,
                white_balance: 0,
                iso_speed: 0,
            },
            format: format,
            width_pixels: width_pixels,
            height_pixels: height_pixels,
            stride_bytes: stride_bytes,
            buffer: buffer,
        }
    }

    pub fn format(&self) -> ImageFormat {
        self.format
    }
    pub fn width_pixels(&self) -> i32 {
        self.width_pixels
    }
    pub fn height_pixels(&self) -> i32 {
        self.height_pixels
    }
    pub fn stride_bytes(&self) -> i32 {
        self.stride_bytes
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        (*self.buffer).as_mut()
    }

    pub fn device_timestamp_usec(&self) -> &u64 {
        &self.mutable.device_timestamp_usec
    }
    pub fn system_timestamp_nsec(&self) -> &u64 {
        &self.mutable.system_timestamp_nsec
    }
    pub fn exposure_time_usec(&self) -> &u64 {
        &self.mutable.exposure_time_usec
    }
    pub fn white_balance(&self) -> &u32 {
        &self.mutable.white_balance
    }
    pub fn iso_speed(&self) -> &u32 {
        &self.mutable.iso_speed
    }

    pub fn device_timestamp_usec_mut(&mut self) -> &mut u64 {
        &mut self.mutable.device_timestamp_usec
    }
    pub fn system_timestamp_nsec_mut(&mut self) -> &mut u64 {
        &mut self.mutable.system_timestamp_nsec
    }
    pub fn exposure_time_usec_mut(&mut self) -> &mut u64 {
        &mut self.mutable.exposure_time_usec
    }
    pub fn white_balance_mut(&mut self) -> &mut u32 {
        &mut self.mutable.white_balance
    }
    pub fn iso_speed_mut(&mut self) -> &mut u32 {
        &mut self.mutable.iso_speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_image() {
        let mut img1 = Image::new(ImageFormat::Default, 100, 100, 100, Box::new(BufferWrapper::new([0u8; 30])));

        let _ = Image::new(ImageFormat::Default, 100, 100, 100, Box::new(BufferWrapper::new(vec![0; 40])));

        *img1.iso_speed_mut() = 100;

        {
            let buffer = img1.buffer_mut();

            buffer[0] = 3;
        }

        {
            let buffer = img1.buffer_mut();

            assert_eq!(buffer[0], 3);
        }
    }
}
