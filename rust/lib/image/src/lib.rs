#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use std::sync::{Arc, RwLock};

#[derive(Copy, Clone)]
pub enum ImageFormat {

}

struct MutableImageState {
    device_timestamp_usec: u64,
    system_timestamp_nsec: u64,
    exposure_time_usec: u64,
    white_balance: u32,
    iso_speed: u32,
}

struct ImageState<T>
    where T : AsMut<[u8]>
{
    mutable: RwLock<MutableImageState>,
    buffer: T,
    format: ImageFormat,
    width_pixels: i32,
    height_pixels: i32,
    stride_bytes: i32,
}

pub struct ImageImpl<T>
    where T : AsMut<[u8]>
{
    state: Arc<ImageState<T>>
}

impl<T> ImageImpl<T>
    where T : AsMut<[u8]> {

    pub fn new(
        format: ImageFormat,
        width_pixels: i32,
        height_pixels: i32,
        stride_bytes: i32,
        buffer: T
    ) 
    -> Self {
        ImageImpl {
            state: Arc::new(
                ImageState {
                    mutable: RwLock::new(
                        MutableImageState {
                            device_timestamp_usec: 0,
                            system_timestamp_nsec: 0,
                            exposure_time_usec: 0,
                            white_balance: 0,
                            iso_speed: 0,
                        }
                    ),
                    format: format,
                    width_pixels: width_pixels,
                    height_pixels: height_pixels,
                    stride_bytes: stride_bytes,
                    buffer: buffer
                }
            )
        }
    }
}


trait Image {
    fn format(&self) -> ImageFormat;
    fn width_pixels(&self) -> i32;
    fn height_pixels(&self) -> i32;
    fn stride_bytes(&self) -> i32;
    fn buffer(&mut self) -> &mut [u8];
    fn device_timestamp_usec(&self) -> u64;
    fn system_timestamp_nsec(&self) -> u64;
    fn exposure_time_usec(&self) -> u64;
    fn white_balance(&self) -> u32;
    fn iso_speed(&self) -> u32;
    fn set_device_timestamp_usec(&mut self, value: u64);
    fn set_system_timestamp_nsec(&mut self, value: u64);
    fn set_exposure_time_usec(&mut self, value: u64);
    fn set_white_balance(&mut self, value: u32);
    fn set_iso_speed(&mut self, value: u32);
}



impl<T> Image for ImageImpl<T> 
where T : AsMut<[u8]>
{
    fn format(&self) -> ImageFormat
    {
        self.state.format
    }
    fn width_pixels(&self) -> i32
    {
        self.state.width_pixels
    }
    fn height_pixels(&self) -> i32
    {
        self.state.height_pixels
    }
    fn stride_bytes(&self) -> i32
    {
        self.state.stride_bytes
    }
    fn buffer(&mut self) -> &mut [u8]
    {
        panic!("Don't know how to implement this yet")

        // I think I need to implement an RAII guard pattern of some sort
        // here, and I'm not sure if I can do it in safe code.

        // The caller needs to get a pointer the mut [u8], but to be safe, they
        // need to be holding a lock while they have it
        
    }
    fn device_timestamp_usec(&self) -> u64
    {
        self.state.mutable.read().unwrap().device_timestamp_usec
    }
    fn system_timestamp_nsec(&self) -> u64
    {
        self.state.mutable.read().unwrap().system_timestamp_nsec
    }
    fn exposure_time_usec(&self) -> u64
    {
        self.state.mutable.read().unwrap().exposure_time_usec
    }
    fn white_balance(&self) -> u32
    {
        self.state.mutable.read().unwrap().white_balance
    }
    fn iso_speed(&self) -> u32
    {
        self.state.mutable.read().unwrap().iso_speed
    }
    fn set_device_timestamp_usec(&mut self, value: u64)
    {
        self.state.mutable.write().unwrap().device_timestamp_usec = value;
    }
    fn set_system_timestamp_nsec(&mut self, value: u64)
    {
        self.state.mutable.write().unwrap().system_timestamp_nsec = value;
    }
    fn set_exposure_time_usec(&mut self, value: u64)
    {
        self.state.mutable.write().unwrap().exposure_time_usec = value;
    }
    fn set_white_balance(&mut self, value: u32)
    {
        self.state.mutable.write().unwrap().white_balance = value;
    }
    fn set_iso_speed(&mut self, value: u32)
    {
        self.state.mutable.write().unwrap().iso_speed = value;
    }
}
