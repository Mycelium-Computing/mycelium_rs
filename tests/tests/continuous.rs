use std::sync::{LazyLock, Mutex};

use dust_dds::infrastructure::type_support::DdsType;
use mycelium_computing::{consumes, provides};

#[derive(DdsType)]
struct Number {
    value: i32,
}

#[provides([
    Continuous("integer", Number)
])]
struct NumberGenerator;

#[consumes([
    Continuous("integer", Number)
])]
struct NumberReceiver;

struct TestState {
    calls: i32,
    total_sum: i32,
}

static STATE_INSTANCE: LazyLock<Mutex<TestState>> = LazyLock::new(|| {
    Mutex::new(TestState {
        calls: 0,
        total_sum: 0,
    })
});

impl NumberReceiverContinuosTrait for NumberReceiver {
    async fn integer(data: Number) {
        STATE_INSTANCE.lock().unwrap().calls += 1;
        STATE_INSTANCE.lock().unwrap().total_sum += data.value;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync;
    use mycelium_computing::core::module::Module;
    use smol::Timer;

    use crate::{Number, NumberGenerator, NumberReceiver, STATE_INSTANCE};

    async fn provider_application() {
        let domain_participant_factory = DomainParticipantFactoryAsync::get_instance();
        let mut application =
            Module::new(150, "test_application", domain_participant_factory).await;

        let continuous_handle = application.register_provider::<NumberGenerator>().await;

        // This delay is intended as the continuous data should be discarded if not listened.
        Timer::after(Duration::from_millis(500)).await;

        continuous_handle.integer(Number { value: 1 }).await;
        continuous_handle.integer(Number { value: 2 }).await;
        continuous_handle.integer(Number { value: 3 }).await;

        Timer::after(Duration::from_secs(2)).await;
    }

    async fn consumer_application() {
        let factory = DomainParticipantFactoryAsync::get_instance();
        let mut app = Module::new(150, "test_consumer", factory).await;

        let _ = app.register_consumer::<NumberReceiver>().await;
        Timer::after(Duration::from_secs(2)).await;
    }

    async fn test_number_generator_and_receiver() {
        let consumer = std::thread::spawn(|| {
            smol::block_on(consumer_application());
        });
        let provider = std::thread::spawn(move || {
            smol::block_on(provider_application());
        });

        provider.join().unwrap();
        consumer.join().unwrap();

        assert_eq!(STATE_INSTANCE.lock().unwrap().total_sum, 6);
        assert_eq!(STATE_INSTANCE.lock().unwrap().calls, 3);
    }

    #[test]
    fn test_number_generator_and_receiver_wrapper() {
        smol::block_on(test_number_generator_and_receiver());
    }
}
