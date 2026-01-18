pub mod consumer;
pub mod provider;

use crate::core::application::provider::ProviderTrait;
use crate::core::listener::{
    NoOpDataReaderListener, NoOpDataWriterListener, NoOpParticipantListener, NoOpPublisherListener,
    NoOpSubscriberListener, NoOpTopicListener,
};
use crate::core::messages::{ConsumerDiscovery, ProviderMessage};
use crate::utils::storage::ExecutionObjects;
use dust_dds::dds_async::data_reader::DataReaderAsync;
use dust_dds::dds_async::data_writer::DataWriterAsync;
use dust_dds::dds_async::domain_participant::DomainParticipantAsync;
use dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync;
use dust_dds::dds_async::publisher::PublisherAsync;
use dust_dds::dds_async::subscriber::SubscriberAsync;
use dust_dds::infrastructure::qos::QosKind;
use dust_dds::infrastructure::status::{NO_STATUS, StatusKind};
use dust_dds::runtime::DdsRuntime;

pub struct Application {
    name: String,
    pub participant: DomainParticipantAsync,
    pub publisher: PublisherAsync,
    subscriber: SubscriberAsync,
    pub consumer_request_reader: DataReaderAsync<ConsumerDiscovery>,
    provider_registration_writer: DataWriterAsync<ProviderMessage>,
    pub providers: Vec<ProviderMessage>,
    objects_storage: ExecutionObjects,
}

impl Application {
    pub async fn run_forever(&self) {
        println!("{} is waiting forever", self.name);
        core::future::pending::<()>().await;
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

        // Create and return the continuous handle for this provider
        P::create_continuous_handle(&self.participant, &self.publisher).await
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

        let consumer_request_topic = participant
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

        // TODO: Define the specific QoS configuration for the writer
        let provider_registration_writer = publisher
            .create_datawriter::<ProviderMessage>(
                &provider_registration_topic,
                QosKind::Default,
                None::<NoOpDataWriterListener>,
                NO_STATUS,
            )
            .await
            .unwrap();

        let providers = Vec::new();

        // TODO: Define the specific QoS configuration for the reader
        let consumer_request_reader = subscriber
            .create_datareader::<ConsumerDiscovery>(
                &consumer_request_topic,
                QosKind::Default,
                None::<NoOpDataReaderListener>,
                &[StatusKind::DataAvailable],
            )
            .await
            .unwrap();

        let objects_storage = ExecutionObjects::new();

        Application {
            name: name.to_string(),
            participant,
            publisher,
            subscriber,
            consumer_request_reader,
            provider_registration_writer,
            providers,
            objects_storage,
        }
    }
}
