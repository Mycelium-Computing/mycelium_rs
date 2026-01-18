pub mod consumer;
pub mod provider;

extern crate alloc;

use crate::core::listener::{
    NoOpDataReaderListener, NoOpDataWriterListener, NoOpParticipantListener, NoOpPublisherListener,
    NoOpSubscriberListener, NoOpTopicListener,
};
use crate::core::messages::{ConsumerDiscovery, ProviderMessage};
use crate::core::module::consumer::ConsumerTrait;
use crate::core::module::provider::ProviderTrait;
use crate::core::qos::{reliable_reader_qos, reliable_writer_qos};
use crate::utils::storage::ExecutionObjects;
use alloc::string::{String, ToString};
use core::time::Duration as StdDuration;
use dust_dds::dds_async::data_reader::DataReaderAsync;
use dust_dds::dds_async::data_writer::DataWriterAsync;
use dust_dds::dds_async::domain_participant::DomainParticipantAsync;
use dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync;
use dust_dds::dds_async::publisher::PublisherAsync;
use dust_dds::dds_async::subscriber::SubscriberAsync;
use dust_dds::infrastructure::qos::QosKind;
use dust_dds::infrastructure::status::{NO_STATUS, StatusKind};
use dust_dds::infrastructure::time::Duration;
use dust_dds::runtime::DdsRuntime;

pub struct Module {
    name: String,
    pub participant: DomainParticipantAsync,
    pub publisher: PublisherAsync,
    subscriber: SubscriberAsync,
    provider_registration_writer: DataWriterAsync<ProviderMessage>,
    provider_registration_reader: DataReaderAsync<ProviderMessage>,
    consumer_discovery_writer: DataWriterAsync<ConsumerDiscovery>,
    consumer_discovery_reader: DataReaderAsync<ConsumerDiscovery>,
    objects_storage: ExecutionObjects,
}

impl Module {
    pub async fn run_forever(&self) {
        core::future::pending::<()>().await;
    }

    pub fn provider_registration_reader(&self) -> &DataReaderAsync<ProviderMessage> {
        &self.provider_registration_reader
    }

    pub fn consumer_discovery_reader(&self) -> &DataReaderAsync<ConsumerDiscovery> {
        &self.consumer_discovery_reader
    }

    /// Waits until at least one provider is discovered on the ProviderRegistration topic.
    /// This ensures the SEDP handshake has completed and data can flow.
    pub async fn wait_for_providers(&self) {
        loop {
            let status = self
                .provider_registration_reader
                .get_subscription_matched_status()
                .await;

            if let Ok(status) = status {
                if status.current_count > 0 {
                    break;
                }
            }

            futures_timer::Delay::new(StdDuration::from_millis(10)).await;
        }

        self.provider_registration_reader
            .wait_for_historical_data(Duration::new(5, 0))
            .await
            .ok();
    }

    /// Waits until at least one consumer is discovered on the ConsumerDiscovery topic.
    /// This ensures the SEDP handshake has completed and data can flow.
    pub async fn wait_for_consumers(&self) {
        loop {
            let status = self
                .consumer_discovery_reader
                .get_subscription_matched_status()
                .await;

            if let Ok(status) = status {
                if status.current_count > 0 {
                    break;
                }
            }

            futures_timer::Delay::new(StdDuration::from_millis(10)).await;
        }

        self.consumer_discovery_reader
            .wait_for_historical_data(Duration::new(5, 0))
            .await
            .ok();
    }

    /// Registers a provider and returns its ContinuousHandle for publishing continuous data.
    ///
    /// The returned handle contains writers for all continuous functionalities and should
    /// be stored for the lifetime of the provider to publish data without reinstantiation.
    ///
    /// For providers without continuous functionalities, this returns `NoContinuousHandle`.
    pub async fn register_provider<P: ProviderTrait>(&mut self) -> P::ContinuousHandle {
        let functionalities = P::get_functionalities();

        self.provider_registration_writer
            .write(functionalities.clone(), None)
            .await
            .unwrap();

        for functionality in functionalities.functionalities {
            P::create_execution_objects(
                functionality.name,
                &self.participant,
                &self.publisher,
                &self.subscriber,
                &mut self.objects_storage,
            )
            .await;
        }

        P::create_continuous_handle(&self.participant, &self.publisher).await
    }

    pub async fn register_consumer<C: ConsumerTrait>(&mut self) -> C::Handle {
        let consumer_id = C::get_consumer_id();
        let functionalities = C::get_requested_functionalities();

        for functionality in functionalities {
            self.consumer_discovery_writer
                .write(
                    ConsumerDiscovery {
                        consumer_id: consumer_id.clone(),
                        requested_functionality: functionality,
                    },
                    None,
                )
                .await
                .unwrap();
        }

        C::create_handle(&self.participant, &self.publisher, &self.subscriber).await
    }

    pub async fn new<R: DdsRuntime>(
        domain_id: u32,
        name: &str,
        participant_factory: &'static DomainParticipantFactoryAsync<R>,
    ) -> Self {
        let participant = participant_factory
            .create_participant(
                domain_id as i32,
                QosKind::Default,
                None::<NoOpParticipantListener>,
                NO_STATUS,
            )
            .await
            .unwrap();

        let provider_registration_topic = participant
            .create_topic::<ProviderMessage>(
                "ProviderRegistration",
                "ProviderRegistration",
                QosKind::Default,
                None::<NoOpTopicListener>,
                NO_STATUS,
            )
            .await
            .unwrap();

        let consumer_discovery_topic = participant
            .create_topic::<ConsumerDiscovery>(
                "ConsumerDiscovery",
                "ConsumerDiscovery",
                QosKind::Default,
                None::<NoOpTopicListener>,
                NO_STATUS,
            )
            .await
            .unwrap();

        let publisher = participant
            .create_publisher(QosKind::Default, None::<NoOpPublisherListener>, NO_STATUS)
            .await
            .unwrap();

        let subscriber = participant
            .create_subscriber(QosKind::Default, None::<NoOpSubscriberListener>, NO_STATUS)
            .await
            .unwrap();

        let provider_registration_writer = publisher
            .create_datawriter::<ProviderMessage>(
                &provider_registration_topic,
                QosKind::Specific(reliable_writer_qos()),
                None::<NoOpDataWriterListener>,
                NO_STATUS,
            )
            .await
            .unwrap();

        let provider_registration_reader = subscriber
            .create_datareader::<ProviderMessage>(
                &provider_registration_topic,
                QosKind::Specific(reliable_reader_qos()),
                None::<NoOpDataReaderListener>,
                &[StatusKind::DataAvailable],
            )
            .await
            .unwrap();

        let consumer_discovery_writer = publisher
            .create_datawriter::<ConsumerDiscovery>(
                &consumer_discovery_topic,
                QosKind::Specific(reliable_writer_qos()),
                None::<NoOpDataWriterListener>,
                NO_STATUS,
            )
            .await
            .unwrap();

        let consumer_discovery_reader = subscriber
            .create_datareader::<ConsumerDiscovery>(
                &consumer_discovery_topic,
                QosKind::Specific(reliable_reader_qos()),
                None::<NoOpDataReaderListener>,
                &[StatusKind::DataAvailable],
            )
            .await
            .unwrap();

        let objects_storage = ExecutionObjects::new();

        Module {
            name: name.to_string(),
            participant,
            publisher,
            subscriber,
            provider_registration_writer,
            provider_registration_reader,
            consumer_discovery_writer,
            consumer_discovery_reader,
            objects_storage,
        }
    }
}
