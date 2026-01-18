use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicU32, Ordering},
};
use std::time::Duration;

use dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync;
use dust_dds::infrastructure::type_support::DdsType;
use mycelium_computing::core::module::Module;
use mycelium_computing::{consumes, provides};
use serial_test::serial;
use smol::Timer;

// =============================================================================
// SHARED DATA TYPES
// =============================================================================

#[derive(DdsType, Clone)]
pub struct SensorData {
    pub sensor_id: i32,
    pub value: f32,
}

#[derive(DdsType, Clone)]
pub struct MathRequest {
    pub a: i32,
    pub b: i32,
}

#[derive(DdsType, Clone)]
pub struct MathResponse {
    pub result: i32,
}

#[derive(DdsType, Clone)]
pub struct StatusInfo {
    pub status_code: i32,
    pub message: String,
}

// =============================================================================
// CONTINUOUS PROVIDER
// =============================================================================

#[provides([
    Continuous("sensor_stream", SensorData)
])]
struct SensorProvider;

// =============================================================================
// CONTINUOUS CONSUMER
// =============================================================================

static CONTINUOUS_RECEIVED_COUNT: AtomicI32 = AtomicI32::new(0);
static CONTINUOUS_TOTAL_VALUE: AtomicI32 = AtomicI32::new(0);

#[consumes([
    Continuous("sensor_stream", SensorData)
])]
struct SensorConsumer;

impl SensorConsumerContinuosTrait for SensorConsumer {
    async fn sensor_stream(data: SensorData) {
        CONTINUOUS_RECEIVED_COUNT.fetch_add(1, Ordering::SeqCst);
        CONTINUOUS_TOTAL_VALUE.fetch_add(data.value as i32, Ordering::SeqCst);
    }
}

// =============================================================================
// REQUEST-RESPONSE PROVIDER
// =============================================================================

#[provides([
    RequestResponse("multiply", MathRequest, MathResponse)
])]
struct MathProvider;

impl MathProviderProviderTrait for MathProvider {
    async fn multiply(request: MathRequest) -> MathResponse {
        MathResponse {
            result: request.a * request.b,
        }
    }
}

#[consumes([
    RequestResponse("multiply", MathRequest, MathResponse)
])]
struct MathConsumer;

// =============================================================================
// RESPONSE PROVIDER (no request input)
// =============================================================================

#[provides([
    Response("get_status", StatusInfo)
])]
struct StatusProvider;

impl StatusProviderProviderTrait for StatusProvider {
    async fn get_status() -> StatusInfo {
        StatusInfo {
            status_code: 200,
            message: "OK".to_string(),
        }
    }
}

#[consumes([
    Response("get_status", StatusInfo)
])]
struct StatusConsumer;

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    use super::*;
    use std::sync::Mutex;

    // These tests must run serially due to DDS discovery interference

    const DOMAIN_CONTINUOUS: u32 = 210;
    const DOMAIN_REQUEST_RESPONSE: u32 = 211;
    const DOMAIN_RESPONSE: u32 = 212;
    const DOMAIN_CONCURRENT: u32 = 213;
    const DOMAIN_SEQUENTIAL: u32 = 214;

    fn reset_continuous_counters() {
        CONTINUOUS_RECEIVED_COUNT.store(0, Ordering::SeqCst);
        CONTINUOUS_TOTAL_VALUE.store(0, Ordering::SeqCst);
    }

    // -------------------------------------------------------------------------
    // Continuous: Multiple consumers receiving from one provider
    // -------------------------------------------------------------------------

    #[test]
    #[serial]
    fn test_continuous_multiple_consumers() {
        reset_continuous_counters();

        let provider_handle = std::thread::spawn(|| {
            smol::block_on(async {
                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_CONTINUOUS, "sensor_provider", factory).await;
                let handle = app.register_provider::<SensorProvider>().await;

                // Wait for consumers to connect
                Timer::after(Duration::from_secs(3)).await;

                for i in 1..=3 {
                    handle
                        .sensor_stream(SensorData {
                            sensor_id: i,
                            value: (i * 10) as f32,
                        })
                        .await;
                    Timer::after(Duration::from_millis(100)).await;
                }

                Timer::after(Duration::from_secs(2)).await;
            });
        });

        let consumer1_handle = std::thread::spawn(|| {
            smol::block_on(async {
                Timer::after(Duration::from_millis(100)).await;
                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_CONTINUOUS, "sensor_consumer_1", factory).await;
                app.register_consumer::<SensorConsumer>().await;
                Timer::after(Duration::from_secs(6)).await;
            });
        });

        let consumer2_handle = std::thread::spawn(|| {
            smol::block_on(async {
                Timer::after(Duration::from_millis(200)).await;
                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_CONTINUOUS, "sensor_consumer_2", factory).await;
                app.register_consumer::<SensorConsumer>().await;
                Timer::after(Duration::from_secs(6)).await;
            });
        });

        let consumer3_handle = std::thread::spawn(|| {
            smol::block_on(async {
                Timer::after(Duration::from_millis(300)).await;
                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_CONTINUOUS, "sensor_consumer_3", factory).await;
                app.register_consumer::<SensorConsumer>().await;
                Timer::after(Duration::from_secs(6)).await;
            });
        });

        consumer1_handle.join().unwrap();
        consumer2_handle.join().unwrap();
        consumer3_handle.join().unwrap();
        provider_handle.join().unwrap();

        let total_count = CONTINUOUS_RECEIVED_COUNT.load(Ordering::SeqCst);
        let total_value = CONTINUOUS_TOTAL_VALUE.load(Ordering::SeqCst);

        // 3 consumers each receiving 3 messages = 9 total messages
        // Each message value: 10, 20, 30. Total per consumer = 60
        // But since all use the same static counter, we get the sum across all consumers
        assert!(
            total_count >= 3,
            "Should receive at least 3 messages total (got {})",
            total_count
        );
        assert!(
            total_value >= 60,
            "Total value should be at least 60 (got {})",
            total_value
        );
    }

    // -------------------------------------------------------------------------
    // RequestResponse: Multiple consumers calling one provider
    // -------------------------------------------------------------------------

    async fn test_consumer(name: String, a: i32, b: i32, delay_ms: u64) -> i32 {
        Timer::after(Duration::from_secs(2) + Duration::from_millis(delay_ms)).await;

        let factory = DomainParticipantFactoryAsync::get_instance();
        let mut app = Module::new(DOMAIN_REQUEST_RESPONSE, &name, factory).await;

        let consumer = app.register_consumer::<MathConsumer>().await;

        Timer::after(Duration::from_millis(500)).await;

        let request = MathRequest { a, b };

        println!("Sending request from consumer {} (a={}, b={})", name, a, b);

        let result = consumer
            .multiply(
                request,
                dust_dds::dcps::infrastructure::time::Duration::new(8, 0),
            )
            .await;

        match &result {
            Some(data) => println!("Consumer {} received response: {}", name, data.result),
            None => println!("Consumer {} received NO response (timeout)", name),
        }

        if let Some(data) = result {
            return data.result;
        }
        return -1;
    }

    #[test]
    #[serial]
    fn test_request_response_multiple_consumers() {
        let provider_handle = std::thread::spawn(|| {
            smol::block_on(async {
                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_REQUEST_RESPONSE, "math_provider", factory).await;
                app.register_provider::<MathProvider>().await;
                Timer::after(Duration::from_secs(20)).await;
            });
        });

        let values = (0..5)
            .into_par_iter()
            .map(|i| smol::block_on(test_consumer(format!("{}", i), i, i + 1, (i * 300) as u64)))
            .collect::<Vec<_>>();

        provider_handle.join().unwrap();

        for i in 0..5 {
            assert_eq!(values[i], i as i32 * (i as i32 + 1));
        }
    }

    // -------------------------------------------------------------------------
    // Response: Multiple consumers calling one provider (no request data)
    // -------------------------------------------------------------------------

    #[test]
    #[serial]
    fn test_response_multiple_consumers() {
        let provider_handle = std::thread::spawn(|| {
            smol::block_on(async {
                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_RESPONSE, "status_provider", factory).await;
                app.register_provider::<StatusProvider>().await;
                Timer::after(Duration::from_secs(8)).await;
            });
        });

        let results: Arc<Mutex<Vec<(&'static str, i32, String)>>> =
            Arc::new(Mutex::new(Vec::new()));

        let results1 = Arc::clone(&results);
        let consumer1_handle = std::thread::spawn(move || {
            smol::block_on(async {
                Timer::after(Duration::from_secs(1)).await;

                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_RESPONSE, "status_consumer_1", factory).await;
                let consumer = app.register_consumer::<StatusConsumer>().await;

                Timer::after(Duration::from_millis(500)).await;

                let timeout = dust_dds::dcps::infrastructure::time::Duration::new(5, 0);
                let response = consumer.get_status(timeout).await;

                if let Some(res) = response {
                    results1
                        .lock()
                        .unwrap()
                        .push(("consumer1", res.status_code, res.message));
                }
            });
        });

        let results2 = Arc::clone(&results);
        let consumer2_handle = std::thread::spawn(move || {
            smol::block_on(async {
                Timer::after(Duration::from_secs(1)).await;

                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_RESPONSE, "status_consumer_2", factory).await;
                let consumer = app.register_consumer::<StatusConsumer>().await;

                Timer::after(Duration::from_millis(600)).await;

                let timeout = dust_dds::dcps::infrastructure::time::Duration::new(5, 0);
                let response = consumer.get_status(timeout).await;

                if let Some(res) = response {
                    results2
                        .lock()
                        .unwrap()
                        .push(("consumer2", res.status_code, res.message));
                }
            });
        });

        let results3 = Arc::clone(&results);
        let consumer3_handle = std::thread::spawn(move || {
            smol::block_on(async {
                Timer::after(Duration::from_secs(1)).await;

                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_RESPONSE, "status_consumer_3", factory).await;
                let consumer = app.register_consumer::<StatusConsumer>().await;

                Timer::after(Duration::from_millis(700)).await;

                let timeout = dust_dds::dcps::infrastructure::time::Duration::new(5, 0);
                let response = consumer.get_status(timeout).await;

                if let Some(res) = response {
                    results3
                        .lock()
                        .unwrap()
                        .push(("consumer3", res.status_code, res.message));
                }
            });
        });

        consumer1_handle.join().unwrap();
        consumer2_handle.join().unwrap();
        consumer3_handle.join().unwrap();
        provider_handle.join().unwrap();

        let final_results = results.lock().unwrap();
        assert_eq!(
            final_results.len(),
            3,
            "All 3 consumers should get responses"
        );

        for (name, status_code, message) in final_results.iter() {
            assert_eq!(*status_code, 200, "{} should receive status 200", name);
            assert_eq!(message, "OK", "{} should receive message 'OK'", name);
        }
    }

    // -------------------------------------------------------------------------
    // Concurrent requests: Multiple instances sending requests simultaneously
    // -------------------------------------------------------------------------

    #[test]
    #[serial]
    fn test_request_response_concurrent_requests() {
        let provider_handle = std::thread::spawn(|| {
            smol::block_on(async {
                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_CONCURRENT, "concurrent_provider", factory).await;
                app.register_provider::<MathProvider>().await;
                Timer::after(Duration::from_secs(12)).await;
            });
        });

        let success_count = Arc::new(AtomicU32::new(0));
        let mut handles = Vec::new();

        for i in 0..5 {
            let count = Arc::clone(&success_count);
            let handle = std::thread::spawn(move || {
                smol::block_on(async {
                    Timer::after(Duration::from_secs(1) + Duration::from_millis((i * 200) as u64))
                        .await;

                    let factory = DomainParticipantFactoryAsync::get_instance();
                    let mut app = Module::new(
                        DOMAIN_CONCURRENT,
                        &format!("concurrent_consumer_{}", i),
                        factory,
                    )
                    .await;
                    let consumer = app.register_consumer::<MathConsumer>().await;

                    Timer::after(Duration::from_millis(500)).await;

                    let timeout = dust_dds::dcps::infrastructure::time::Duration::new(8, 0);
                    let a = (i + 1) as i32;
                    let b = (i + 2) as i32;
                    let expected = a * b;

                    let response = consumer.multiply(MathRequest { a, b }, timeout).await;

                    if let Some(res) = response {
                        if res.result == expected {
                            count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                });
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
        provider_handle.join().unwrap();

        let final_count = success_count.load(Ordering::SeqCst);
        assert_eq!(
            final_count, 5,
            "All 5 concurrent consumers should receive correct responses"
        );
    }

    // -------------------------------------------------------------------------
    // Sequential requests: One consumer sending multiple requests
    // -------------------------------------------------------------------------

    #[test]
    #[serial]
    fn test_request_response_sequential_requests() {
        let provider_handle = std::thread::spawn(|| {
            smol::block_on(async {
                let factory = DomainParticipantFactoryAsync::get_instance();
                let mut app = Module::new(DOMAIN_SEQUENTIAL, "sequential_provider", factory).await;
                app.register_provider::<MathProvider>().await;
                Timer::after(Duration::from_secs(15)).await;
            });
        });

        let results = smol::block_on(async {
            Timer::after(Duration::from_secs(1)).await;

            let factory = DomainParticipantFactoryAsync::get_instance();
            let mut app = Module::new(DOMAIN_SEQUENTIAL, "sequential_consumer", factory).await;
            let consumer = app.register_consumer::<MathConsumer>().await;

            Timer::after(Duration::from_millis(500)).await;

            let timeout = dust_dds::dcps::infrastructure::time::Duration::new(5, 0);

            let mut results = Vec::new();

            let res1 = consumer
                .multiply(MathRequest { a: 2, b: 3 }, timeout.clone())
                .await;
            if let Some(r) = res1 {
                results.push(r.result);
            }

            Timer::after(Duration::from_millis(100)).await;

            let res2 = consumer
                .multiply(MathRequest { a: 4, b: 5 }, timeout.clone())
                .await;
            if let Some(r) = res2 {
                results.push(r.result);
            }

            Timer::after(Duration::from_millis(100)).await;

            let res3 = consumer
                .multiply(MathRequest { a: 6, b: 7 }, timeout.clone())
                .await;
            if let Some(r) = res3 {
                results.push(r.result);
            }

            results
        });

        provider_handle.join().unwrap();

        assert_eq!(results.len(), 3, "Should receive 3 responses");
        assert_eq!(results[0], 6, "2 * 3 = 6");
        assert_eq!(results[1], 20, "4 * 5 = 20");
        assert_eq!(results[2], 42, "6 * 7 = 42");
    }
}
