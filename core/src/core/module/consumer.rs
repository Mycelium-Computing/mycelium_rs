extern crate alloc;

use crate::core::messages::ProvidedFunctionality;
use crate::runtime_context::RuntimeContext;
use alloc::{string::String, vec::Vec};
use core::future::Future;
use dust_dds::dds_async::domain_participant::DomainParticipantAsync;
use dust_dds::dds_async::publisher::PublisherAsync;
use dust_dds::dds_async::subscriber::SubscriberAsync;

pub trait ConsumerTrait<C: RuntimeContext> {
    type Handle;

    fn get_consumer_id() -> String;

    fn get_requested_functionalities() -> Vec<ProvidedFunctionality>;

    fn create_handle(
        participant: &DomainParticipantAsync,
        publisher: &PublisherAsync,
        subscriber: &SubscriberAsync,
        context: &C,
    ) -> impl Future<Output = Self::Handle>;
}
