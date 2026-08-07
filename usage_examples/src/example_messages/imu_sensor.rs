use dust_dds::infrastructure::type_support::DdsType;
use serde::Deserialize;

#[derive(DdsType, Debug, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(DdsType, Debug, Deserialize)]
pub struct IMUData {
    accelerometer: Vector3,
    gyroscope: Vector3,
}
