extern crate alloc;

use crate::core::messages::ProviderMessage;
use crate::utils::storage::ExecutionObjects;
use alloc::string::String;
use dust_dds::dds_async::domain_participant::DomainParticipantAsync;
use dust_dds::dds_async::publisher::PublisherAsync;
use dust_dds::dds_async::subscriber::SubscriberAsync;

/// A marker type for providers that don't have continuous functionalities.
/// This type is used as the default `ContinuousHandle` when no continuous
/// methods are defined.
pub struct NoContinuousHandle;

pub trait ProviderTrait {
    /// The handle type that provides access to continuous functionality writers.
    /// For providers without continuous functionalities, this should be `NoContinuousHandle`.
    type ContinuousHandle;

    fn get_functionalities() -> ProviderMessage;

    fn create_execution_objects(
        functionality_name: String,
        participant: &DomainParticipantAsync,
        publisher: &PublisherAsync,
        subscriber: &SubscriberAsync,
        storage: &mut ExecutionObjects,
    ) -> impl Future<Output = ()>;

    /// Creates the continuous handle containing writers for all continuous functionalities.
    /// This handle should be stored and used to publish continuous data throughout
    /// the provider's lifetime.
    ///
    /// For providers without continuous functionalities, this returns `NoContinuousHandle`.
    fn create_continuous_handle(
        participant: &DomainParticipantAsync,
        publisher: &PublisherAsync,
    ) -> impl Future<Output = Self::ContinuousHandle>;
}
