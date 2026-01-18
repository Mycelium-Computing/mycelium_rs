use dust_dds::infrastructure::type_support::DdsType;

#[derive(DdsType, Debug)]
pub struct MathRequest {
    pub operand1: f32,
    pub operand2: f32,
}

#[derive(DdsType, Debug)]
pub struct MathResult {
    pub result: f32,
}
