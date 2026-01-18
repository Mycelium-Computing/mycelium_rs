#[path = "../common/mod.rs"]
mod common;

use std::time::Duration;

use dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync;
use mycelium_computing::core::application::Application;
use mycelium_computing::provides;
use smol::Timer;

use crate::common::types::{MathRequest, MathResult};

/// Provider implementation for benchmarking
#[provides([
    RequestResponse("sum_two_numbers", MathRequest, MathResult)
])]
struct Math;

impl MathProviderTrait for Math {
    async fn sum_two_numbers(input: MathRequest) -> MathResult {
        MathResult {
            result: input.operand1 + input.operand2,
        }
    }
}

async fn run_provider(domain_id: u32) {
    let factory = DomainParticipantFactoryAsync::get_instance();
    let mut app = Application::new(domain_id, "BenchmarkProvider", factory).await;
    app.register_provider::<Math>().await;

    Timer::after(Duration::from_secs(10)).await;
}

fn main() {
    let domain_id = 0;

    smol::block_on(run_provider(domain_id));
}
