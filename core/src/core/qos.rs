use crate::runtime_context::{RuntimeContext, SelectResult, TimerHandleOf};
use core::time::Duration;
use dust_dds::dds_async::data_reader::DataReaderAsync;
use dust_dds::dds_async::data_writer::DataWriterAsync;
use dust_dds::infrastructure::qos::{DataReaderQos, DataWriterQos};
use dust_dds::infrastructure::qos_policy::{
    DurabilityQosPolicy, DurabilityQosPolicyKind, HistoryQosPolicy, HistoryQosPolicyKind,
    ReliabilityQosPolicy, ReliabilityQosPolicyKind,
};
use dust_dds::infrastructure::time::DurationKind;
use dust_dds::infrastructure::type_support::TypeSupport;
use dust_dds::runtime::Timer;

pub fn reliable_writer_qos() -> DataWriterQos {
    DataWriterQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Infinite,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(100),
        },
        ..Default::default()
    }
}

pub fn reliable_reader_qos() -> DataReaderQos {
    DataReaderQos {
        durability: DurabilityQosPolicy {
            kind: DurabilityQosPolicyKind::TransientLocal,
        },
        reliability: ReliabilityQosPolicy {
            kind: ReliabilityQosPolicyKind::Reliable,
            max_blocking_time: DurationKind::Infinite,
        },
        history: HistoryQosPolicy {
            kind: HistoryQosPolicyKind::KeepLast(100),
        },
        ..Default::default()
    }
}

pub async fn wait_for_writer_match<C, T>(
    writer: &DataWriterAsync<T>,
    timeout: Duration,
    timer: TimerHandleOf<C>,
) -> bool
where
    C: RuntimeContext,
    T: TypeSupport + Send + Sync,
    TimerHandleOf<C>: Timer + Clone + Send + Sync + 'static,
{
    let writer = writer.clone();
    let mut match_timer = timer.clone();
    let match_check = async move {
        loop {
            let status = writer.get_publication_matched_status().await;

            if let Ok(status) = status {
                if status.current_count > 0 {
                    return true;
                }
            }

            match_timer.delay(Duration::from_millis(10)).await;
        }
    };

    let mut timeout_timer = timer;
    let timeout_future = timeout_timer.delay(timeout);

    match C::select(match_check, timeout_future).await {
        SelectResult::First(result) => result,
        SelectResult::Second(_) => false,
    }
}

pub async fn wait_for_reader_match<C, T>(
    reader: &DataReaderAsync<T>,
    timeout: Duration,
    timer: TimerHandleOf<C>,
) -> bool
where
    C: RuntimeContext,
    T: TypeSupport + Send + Sync,
    TimerHandleOf<C>: Timer + Clone + Send + Sync + 'static,
{
    let reader = reader.clone();
    let mut match_timer = timer.clone();
    let match_check = async move {
        loop {
            let status = reader.get_subscription_matched_status().await;

            if let Ok(status) = status {
                if status.current_count > 0 {
                    return true;
                }
            }

            match_timer.delay(Duration::from_millis(10)).await;
        }
    };

    let mut timeout_timer = timer;
    let timeout_future = timeout_timer.delay(timeout);

    match C::select(match_check, timeout_future).await {
        SelectResult::First(result) => result,
        SelectResult::Second(_) => false,
    }
}
