extern crate alloc;

use crate::core::messages::ProviderExchange;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use dust_dds::dcps::channels::oneshot::OneshotSender;
use dust_dds::dds_async::data_reader::DataReaderAsync;
use dust_dds::dds_async::data_reader_listener::DataReaderListener;
use dust_dds::dds_async::data_writer::DataWriterAsync;
use dust_dds::dds_async::data_writer_listener::DataWriterListener;
use dust_dds::dds_async::publisher_listener::PublisherListener;
use dust_dds::dds_async::subscriber_listener::SubscriberListener;
use dust_dds::dds_async::topic_listener::TopicListener;
use dust_dds::domain::domain_participant_listener::DomainParticipantListener;
use dust_dds::infrastructure::sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE};
use dust_dds::infrastructure::type_support::TypeSupport;

pub struct NoOpParticipantListener;
impl DomainParticipantListener for NoOpParticipantListener {}

pub struct NoOpTopicListener;
impl TopicListener for NoOpTopicListener {}

pub struct NoOpPublisherListener;
impl PublisherListener for NoOpPublisherListener {}

pub struct NoOpSubscriberListener;
impl SubscriberListener for NoOpSubscriberListener {}

pub struct NoOpDataWriterListener;
impl<T: TypeSupport + 'static> DataWriterListener<T> for NoOpDataWriterListener {}

pub struct NoOpDataReaderListener;
impl<T: TypeSupport + 'static> DataReaderListener<T> for NoOpDataReaderListener {}

pub struct RequestListener<I: TypeSupport, O: TypeSupport> {
    pub writer: DataWriterAsync<O>,
    pub implementation: Box<dyn Fn(I) -> Pin<Box<dyn Future<Output = O> + Send>> + Send + Sync>,
}

impl<I, O> DataReaderListener<I> for RequestListener<I, O>
where
    I: TypeSupport + Send + Sync + 'static,
    O: TypeSupport + Send + Sync + 'static,
{
    async fn on_data_available(&mut self, reader: DataReaderAsync<I>) {
        let samples = reader
            .take(100, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .await;

        if let Ok(data) = samples {
            for sample in data {
                if let Some(d) = sample.data {
                    let result = (self.implementation)(d).await;
                    self.writer.write(result, None).await.unwrap();
                }
            }
        }
    }
}

pub struct ProviderResponseListener<T: Send> {
    pub expected_id: u32,
    pub response_sender: Option<OneshotSender<T>>,
}

impl<T> DataReaderListener<ProviderExchange<T>> for ProviderResponseListener<T>
where
    T: TypeSupport + Send + Sync + 'static,
{
    async fn on_data_available(&mut self, reader: DataReaderAsync<ProviderExchange<T>>) {
        let samples = reader
            .take(100, ANY_SAMPLE_STATE, ANY_VIEW_STATE, ANY_INSTANCE_STATE)
            .await;

        if let Err(_) = samples {
            return;
        }

        let samples = samples.unwrap();

        let found = samples
            .into_iter()
            .filter_map(|sample| {
                if let Some(data) = sample.data {
                    if data.id == self.expected_id {
                        return Some(data);
                    }
                }
                None
            })
            .next();

        if let Some(data) = found {
            if let Some(sender) = self.response_sender.take() {
                sender.send(data.payload);
            }
        };
    }
}
