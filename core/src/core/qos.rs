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

pub async fn wait_for_writer_match<T: TypeSupport>(
    writer: &DataWriterAsync<T>,
    timeout: Duration,
) -> bool {
    use futures::FutureExt;

    let match_check = async {
        loop {
            let status = writer.get_publication_matched_status().await;

            if let Ok(status) = status {
                if status.current_count > 0 {
                    return true;
                }
            }

            futures_timer::Delay::new(Duration::from_millis(10)).await;
        }
    }
    .fuse();

    let timeout_future = futures_timer::Delay::new(timeout).fuse();

    futures::pin_mut!(match_check);
    futures::pin_mut!(timeout_future);

    futures::select! {
        result = match_check => result,
        _ = timeout_future => false,
    }
}

pub async fn wait_for_reader_match<T: TypeSupport>(
    reader: &DataReaderAsync<T>,
    timeout: Duration,
) -> bool {
    use futures::FutureExt;

    let match_check = async {
        loop {
            let status = reader.get_subscription_matched_status().await;

            if let Ok(status) = status {
                if status.current_count > 0 {
                    return true;
                }
            }

            futures_timer::Delay::new(Duration::from_millis(10)).await;
        }
    }
    .fuse();

    let timeout_future = futures_timer::Delay::new(timeout).fuse();

    futures::pin_mut!(match_check);
    futures::pin_mut!(timeout_future);

    futures::select! {
        result = match_check => result,
        _ = timeout_future => false,
    }
}
