// RUN THIS ON AN ANDROID DEVICE USING TERMUX
mod example_messages;

use example_messages::imu_sensor::{IMUData, Vector3};
use mycelium_computing::core::module::Module;
use mycelium_computing::runtimes::StdRuntimeContext;
use mycelium_computing::{consumes, provides};
use serde::Deserialize;
use std::env;
use std::io::BufReader;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
struct TermuxVector {
    values: [f32; 3],
}

impl From<TermuxVector> for Vector3 {
    fn from(vector: TermuxVector) -> Self {
        let [x, y, z] = vector.values;
        Self { x, y, z }
    }
}

#[derive(Deserialize)]
struct TermuxImuSample {
    #[serde(rename = "icm456xy_acc")]
    accelerometer: TermuxVector,
    #[serde(rename = "icm456xy_gyro")]
    gyroscope: TermuxVector,
}

impl From<TermuxImuSample> for IMUData {
    fn from(sample: TermuxImuSample) -> Self {
        Self::new(sample.accelerometer.into(), sample.gyroscope.into())
    }
}

#[provides([
    Continuous("imu", IMUData),
])]
struct SmartphoneSensor;

async fn provider() -> Result<(), Box<dyn std::error::Error>> {
    let delay_ms = env::args().nth(2).unwrap_or_else(|| "10".to_string());

    let mut app = Module::new(0, "SmartphoneSensor", StdRuntimeContext::new()).await;

    let sensor_handle = app.register_provider::<SmartphoneSensor>().await;

    let mut sensor_process = Command::new("termux-sensor")
        .args(["-s", "icm456xy_acc,icm456xy_gyro,mmc5603", "-d", &delay_ms]) // Include the sensors of your interest
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = sensor_process
        .stdout
        .take()
        .ok_or("failed to capture termux-sensor stdout")?;

    let reader = BufReader::new(stdout);

    let stream = serde_json::Deserializer::from_reader(reader).into_iter::<TermuxImuSample>();

    for sample in stream {
        match sample {
            Ok(sample) => {
                sensor_handle.imu(sample.into()).await;
            }
            Err(error) => {
                eprintln!("JSON parse error: {error}");
            }
        }
    }

    let status = sensor_process.wait()?;

    if !status.success() {
        eprintln!("termux-sensor exited with status: {status}");
    }

    app.run_forever().await;

    Ok(())
}

#[consumes([
    Continuous("imu", IMUData)
])]
struct Smartphone;

impl SmartphoneContinuosTrait for Smartphone {
    async fn imu(data: IMUData) {
        println!("{:?}", data);
    }
}

async fn consumer() {
    let mut app = Module::new(0, "SomeAppInRobot", StdRuntimeContext::new()).await;

    let _ = app.register_consumer::<Smartphone>().await;

    app.run_forever().await;
}

async fn main_async() {
    if env::args().nth(1).as_deref() == Some("provider") {
        println!("Using as provider");
        if let Err(error) = provider().await {
            eprintln!("Provider error: {error}");
        }
    } else {
        println!("Using as consumer");
        consumer().await;
    }
}

fn main() {
    smol::block_on(main_async());
}
