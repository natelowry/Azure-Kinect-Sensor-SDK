mod bindings;
mod error;

pub use bindings::k4a_plugin_version_t as Version;
pub use error::Error;

use libloading::{Library, Symbol};
use std::sync::mpsc::{SyncSender, Receiver, sync_channel};

enum InputMessage {
    Terminate,
    ProcessImage(crate::image::Image)
}

pub struct DepthEngine {
    lib: Library,
    plugin: bindings::_k4a_plugin_t,
    thread_join_handle: Option<std::thread::JoinHandle<()>>,
    input_tx: SyncSender<InputMessage>
}

const EXPECTED_VERSION : Version = Version { major: 2, minor: 1, patch: 0 };

impl bindings::_k4a_plugin_t {
    pub fn validate(&self) -> Result<(), Error> {
        if EXPECTED_VERSION.major != self.version.major {
            return Err(Error::Version(crate::error::Mismatch::new(EXPECTED_VERSION, self.version)));
        }
        self.depth_engine_create_and_initialize.ok_or(Error::IncompatibleInterface)?;
        self.depth_engine_process_frame.ok_or(Error::IncompatibleInterface)?;
        self.depth_engine_get_output_frame_size.ok_or(Error::IncompatibleInterface)?;
        self.depth_engine_destroy.ok_or(Error::IncompatibleInterface)?;
        self.transform_engine_create_and_initialize.ok_or(Error::IncompatibleInterface)?;
        self.transform_engine_process_frame.ok_or(Error::IncompatibleInterface)?;
        self.transform_engine_get_output_frame_size.ok_or(Error::IncompatibleInterface)?;
        self.transform_engine_destroy.ok_or(Error::IncompatibleInterface)?;

        Ok(())
    }
}

impl DepthEngine {
    pub fn new() -> Result<Self, Error> {

        let path = "C:/Program Files/Azure Kinect SDK v1.3.0/sdk/windows-desktop/amd64/release/bin/depthengine_2_0.dll";
        let lib = Library::new(path)?;

        let func: Symbol<bindings::k4a_register_plugin_fn>;
        
        // Get the pointer to the entry point
        unsafe {
            func = lib.get(b"k4a_register_plugin")?;
        }

        let result;
        let mut plugin: bindings::_k4a_plugin_t;

        // Call the k4a_register_plugin entry point to get the plugin data
        unsafe {
            plugin = std::mem::zeroed();
            result = func.ok_or(Error::IncompatibleInterface)?(&mut plugin);
        }

        // If the function returns false, the interface is not compatible
        if !result {
            return Err(Error::IncompatibleInterface);
        }
        
        plugin.validate()?;

        println!("plugin version {:?}", plugin.version);

        
        let (input_tx, input_rx) = std::sync::mpsc::sync_channel::<InputMessage>(5);

        let join_handle = std::thread::spawn(move || {
            let rx = input_rx;

            loop {
                match rx.recv().unwrap() {
                    InputMessage::Terminate => {
                        break;
                    }
                    InputMessage::ProcessImage(image) => {

                    }
                }
            }
        });
        
        Ok(DepthEngine {
            lib: lib,
            plugin: plugin,
            thread_join_handle: Option::Some(join_handle),
            input_tx: input_tx,
        })
    }

} 

impl Drop for DepthEngine {

    
    fn drop(&mut self) {
        if let Some(join) = self.thread_join_handle.take() {
            self.input_tx.send(InputMessage::Terminate).unwrap();
            join.join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dewrapper_sanity() {
        let _ = super::DepthEngine::new().unwrap();
    }
}
