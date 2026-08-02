use dust_dds::infrastructure::time::Duration;
use mycelium_computing::consumes;
use mycelium_computing::core::module::Module;
use mycelium_computing_std::StdRuntimeContext;

use crate::common::types::{MathRequest, MathResult};

#[path = "../common/mod.rs"]
mod common;

#[consumes(dust_dds::std_runtime::StdRuntime, [
    RequestResponse("sum_two_numbers", MathRequest, MathResult)
])]
struct Math;

async fn init_consumer() {
    let mut app = Module::new(0, "MathConsumer", StdRuntimeContext::new()).await;

    let consumer = app.register_consumer::<Math>().await;

    for _ in 0..100 {
        consumer
            .sum_two_numbers(
                MathRequest {
                    operand1: 10f32,
                    operand2: 20f32,
                },
                Duration::new(10, 0),
            )
            .await;
    }
}

fn main() {
    smol::block_on(init_consumer())
}
