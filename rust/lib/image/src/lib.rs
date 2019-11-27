#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut img1 = Image::new(ImageFormat::default, 100, 100, 100, Box::new([0u8; 30]));

        let mut img2 = Image::new(ImageFormat::default, 100, 100, 100, Box::new(vec![0; 40]));

        img1.set_iso_speed(100);

        {
            let mut buffer = img1.buffer().unwrap();

            let mut b = (**buffer).as_mut();

            b[0] = 3;
        }

        {
            let mut buffer = img1.buffer().unwrap();

            let b = (**buffer).as_mut();

            assert_eq!(b[0], 3);
        }
    }
}

use std::sync::{Arc, Mutex, RwLock};

#[derive(Copy, Clone)]
pub enum ImageFormat {
    default,
}

struct MutableImageState {
    device_timestamp_usec: u64,
    system_timestamp_nsec: u64,
    exposure_time_usec: u64,
    white_balance: u32,
    iso_speed: u32,
}

struct ImageState {
    mutable: RwLock<MutableImageState>,
    format: ImageFormat,
    width_pixels: i32,
    height_pixels: i32,
    stride_bytes: i32,
    buffer: Mutex<Box<dyn AsMut<[u8]>>>,
}

pub struct Image {
    state: Arc<ImageState>,
}

impl Image {
    pub fn new(
        format: ImageFormat,
        width_pixels: i32,
        height_pixels: i32,
        stride_bytes: i32,
        buffer: Box<dyn AsMut<[u8]>>,
    ) -> Self {
        Image {
            state: Arc::new(ImageState {
                mutable: RwLock::new(MutableImageState {
                    device_timestamp_usec: 0,
                    system_timestamp_nsec: 0,
                    exposure_time_usec: 0,
                    white_balance: 0,
                    iso_speed: 0,
                }),
                format: format,
                width_pixels: width_pixels,
                height_pixels: height_pixels,
                stride_bytes: stride_bytes,
                buffer: Mutex::new(buffer),
            }),
        }
    }

    fn format(&self) -> ImageFormat {
        self.state.format
    }
    fn width_pixels(&self) -> i32 {
        self.state.width_pixels
    }
    fn height_pixels(&self) -> i32 {
        self.state.height_pixels
    }
    fn stride_bytes(&self) -> i32 {
        self.state.stride_bytes
    }
    fn buffer(&mut self) -> std::sync::LockResult<std::sync::MutexGuard<Box<dyn AsMut<[u8]>>>> {
        self.state.buffer.lock()
    }
    fn device_timestamp_usec(&self) -> u64 {
        self.state.mutable.read().unwrap().device_timestamp_usec
    }
    fn system_timestamp_nsec(&self) -> u64 {
        self.state.mutable.read().unwrap().system_timestamp_nsec
    }
    fn exposure_time_usec(&self) -> u64 {
        self.state.mutable.read().unwrap().exposure_time_usec
    }
    fn white_balance(&self) -> u32 {
        self.state.mutable.read().unwrap().white_balance
    }
    fn iso_speed(&self) -> u32 {
        self.state.mutable.read().unwrap().iso_speed
    }
    fn set_device_timestamp_usec(&mut self, value: u64) {
        self.state.mutable.write().unwrap().device_timestamp_usec = value;
    }
    fn set_system_timestamp_nsec(&mut self, value: u64) {
        self.state.mutable.write().unwrap().system_timestamp_nsec = value;
    }
    fn set_exposure_time_usec(&mut self, value: u64) {
        self.state.mutable.write().unwrap().exposure_time_usec = value;
    }
    fn set_white_balance(&mut self, value: u32) {
        self.state.mutable.write().unwrap().white_balance = value;
    }
    fn set_iso_speed(&mut self, value: u32) {
        self.state.mutable.write().unwrap().iso_speed = value;
    }
}
