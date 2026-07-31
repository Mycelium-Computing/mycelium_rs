extern crate alloc;

use crate::core::messages::ProvidedFunctionality;
use alloc::{string::String, vec::Vec};
use core::future::Future;
use dust_dds::dds_async::domain_participant::DomainParticipantAsync;
use dust_dds::dds_async::publisher::PublisherAsync;
use dust_dds::dds_async::subscriber::SubscriberAsync;
use dust_dds::runtime::DdsRuntime;

pub trait ConsumerTrait {
    /// Runtime selected by the `consumes` macro.
    type Runtime: DdsRuntime;
    type Handle;

    fn get_consumer_id() -> String;

    fn get_requested_functionalities() -> Vec<ProvidedFunctionality>;

    fn create_handle(
        participant: &DomainParticipantAsync,
        publisher: &PublisherAsync,
        subscriber: &SubscriberAsync,
        timer: <Self::Runtime as DdsRuntime>::TimerHandle,
    ) -> impl Future<Output = Self::Handle>;
}
