extern crate depth_mcu;
extern crate usbcommand;

use depth_mcu::DepthMcuCommonOperations;
use usbcommand::DeviceType;
use usbcommand::Usbcommand;

fn main() {
    println!("Hello, world!");

    let cmd = Usbcommand::open(DeviceType::DepthProcessor, 0).unwrap();

    println!("PID: {}", cmd.pid());
    println!("Serial Number: {}", cmd.serial_number());

    let sn_descriptor = cmd.serial_number().clone();

    let mut mcu = depth_mcu::DepthMcu::new(cmd);

    let sn = mcu.serialnum().unwrap();

    println!("sn: {}", sn);

    assert_eq!(sn, sn_descriptor);

    let mut running_mcu = mcu
        .set_capture_mode(depth_mcu::CaptureMode::PassiveIR)
        .unwrap();

    let cal_data = running_mcu.calibration().unwrap();
    println!("cal result len: {}", cal_data.len());

    println!(
        "extrinsic result: {}",
        running_mcu.extrinsic_calibration().unwrap()
    );

    let mut streaming_mcu = running_mcu.start_streaming(depth_mcu::FPS::Fps15).unwrap();

    std::thread::sleep_ms(20);


    let mut mcu = streaming_mcu.stop_streaming().unwrap();

    println!("sn again: {}", mcu.serialnum().unwrap());
}
