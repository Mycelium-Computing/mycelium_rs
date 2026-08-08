// RUN THIS ON AN ANDROID DEVICE USING TERMUX
mod example_messages;

use example_messages::imu_sensor::{IMUData, Vector3};
use mycelium::core::module::Module;
use mycelium::runtimes::StdRuntimeContext;
use mycelium::{consumes, provides};
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
    #[serde(rename = "icm456xy_acc", default)]
    accelerometer: Option<TermuxVector>,
    #[serde(rename = "icm456xy_gyro", default)]
    gyroscope: Option<TermuxVector>,
}

impl TermuxImuSample {
    fn into_imu_data(self) -> Option<IMUData> {
        Some(IMUData::new(
            self.accelerometer?.into(),
            self.gyroscope?.into(),
        ))
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
        .args(["-s", "icm456xy_acc,icm456xy_gyro", "-d", &delay_ms])
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
                if let Some(data) = sample.into_imu_data() {
                    sensor_handle.imu(data).await;
                }
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
