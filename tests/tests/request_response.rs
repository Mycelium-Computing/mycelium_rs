use dust_dds::infrastructure::type_support::DdsType;
use mycelium::{consumes, provides};

#[derive(DdsType)]
struct ArithmeticRequest {
    a: f32,
    b: f32,
}

#[derive(DdsType)]
struct Number {
    value: f32,
}

#[provides([
    RequestResponse("add_two_ints", ArithmeticRequest, Number)
])]
struct CalculatorProvider;

impl CalculatorProviderProviderTrait for CalculatorProvider {
    async fn add_two_ints(request: ArithmeticRequest) -> Number {
        println!("Adding {} and {}", request.a, request.b);
        Number {
            value: request.a + request.b,
        }
    }
}

#[consumes([
    RequestResponse("add_two_ints", ArithmeticRequest, Number),
])]
struct CalculatorConsumer;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::FutureExt;
    use mycelium::core::module::Module;
    use mycelium::runtimes::StdRuntimeContext;
    use smol::Timer;

    use crate::{
        ArithmeticRequest, CalculatorConsumer, CalculatorConsumerResponseTrait, CalculatorProvider,
    };

    #[test]
    fn test_function() {
        let handle = std::thread::spawn(|| {
            smol::block_on(async {
                let mut app = Module::new(150, "test_provider", StdRuntimeContext::new()).await;
                app.register_provider::<CalculatorProvider>().await;

                Timer::after(Duration::new(2, 0)).await;
            });
        });

        let expected_result = 3.0;

        async fn test_consumer() -> f32 {
            let mut app = Module::new(150, "test_consumer", StdRuntimeContext::new()).await;

            let consumer = app.register_consumer::<CalculatorConsumer>().await;

            let request = ArithmeticRequest { a: 1.0, b: 2.0 };

            let result = consumer
                .add_two_ints(
                    request,
                    dust_dds::dcps::infrastructure::time::Duration::new(2, 0),
                )
                .await;

            if let Some(data) = result {
                return data.value;
            }
            return -1.0;
        }

        let value = smol::block_on(async {
            futures::select! {
                res = test_consumer().fuse() => res,
                _ = Timer::after(Duration::new(1, 0)).fuse() => -1.0,
            }
        });

        assert_eq!(value, expected_result);

        handle.join().unwrap();
    }
}
