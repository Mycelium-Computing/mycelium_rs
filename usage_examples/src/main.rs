// RUN THIS ON AN ANDROID DEVICE USING TERMUX
mod example_messages;

use example_messages::imu_sensor::IMUData;
use mycelium_computing::core::module::Module;
use mycelium_computing::runtimes::StdRuntimeContext;
use mycelium_computing::{consumes, provides};
use serde_json::Value;
use std::env;
use std::io::BufReader;
use std::process::{Command, Stdio};

#[provides([
    Continuous("imu", IMUData),
])]
struct SmartphoneSensor;

async fn provider() {
    let delay_ms = env::args().nth(1).unwrap_or_else(|| "10".to_string());

    let mut app = Module::new(0, "SmartphoneSensor", StdRuntimeContext::new()).await;

    let sensor_handle = app.register_provider::<SmartphoneSensor>().await;

    let mut sensor_process = Command::new("termux-sensor")
        .args(["-s", "icm456xy_acc,icm456xy_gyro,mmc5603", "-d", &delay_ms]) // Include the sensors of your interest
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let stdout = sensor_process
        .stdout
        .take()
        .ok_or("failed to capture termux-sensor stdout")
        .unwrap();

    let reader = BufReader::new(stdout);

    let stream = serde_json::Deserializer::from_reader(reader).into_iter::<Value>();

    for sample in stream {
        match sample {
            Ok(json) => {
                sensor_handle
                    .imu(serde_json::from_value(json).unwrap())
                    .await;
            }
            Err(error) => {
                eprintln!("JSON parse error: {error}");
            }
        }
    }

    let status = sensor_process.wait().unwrap();

    if !status.success() {
        eprintln!("termux-sensor exited with status: {status}");
    }

    app.run_forever().await;
}

#[consumes([
    Continuous("imu_data", IMUData)
])]
struct Smartphone;

impl SmartphoneContinuosTrait for Smartphone {
    async fn imu_data(data: IMUData) {
        println!("{:?}", data);
    }
}

async fn consumer() {
    let mut app = Module::new(0, "SomeAppInRobot", StdRuntimeContext::new()).await;

    let _ = app.register_consumer::<Smartphone>().await;

    app.run_forever().await;
}

async fn main_async() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Using as consumer");
        consumer().await;
    } else if args[1] == "provider" {
        println!("Using as provider");
        provider().await;
    }
}

fn main() {
    smol::block_on(main_async());
}
