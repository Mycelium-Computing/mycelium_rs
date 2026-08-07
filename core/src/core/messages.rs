extern crate alloc;

use alloc::{string::String, vec::Vec};
use dust_dds::infrastructure::type_support::{DdsType, TypeSupport};

#[derive(DdsType, Debug, Clone)]
pub struct ProvidedFunctionality {
    pub name: String,
    pub input_type: String,
    pub output_type: String,
}

#[derive(DdsType, Debug, Clone)]
pub struct ProviderMessage {
    #[dust_dds(key)]
    pub provider_name: String,
    pub functionalities: Vec<ProvidedFunctionality>,
}

#[derive(DdsType, Debug, Clone)]
pub struct ConsumerDiscovery {
    #[dust_dds(key)]
    pub consumer_id: String,
    pub requested_functionality: ProvidedFunctionality,
}

#[derive(DdsType, Debug)]
pub struct EmptyMessage {
    pub _marker: u8,
}

impl Default for EmptyMessage {
    fn default() -> Self {
        Self { _marker: 0 }
    }
}

/// Identifies one request within the response topic.
///
/// The sequence number is only meaningful within the requester scope. A DDS
/// reader handle contains the participant GUID, so it remains unique across
/// processes while still being available without adding an operating-system
/// dependency to this `no_std` crate.
#[derive(DdsType, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestId {
    #[dust_dds(key)]
    pub requester_id: [u8; 16],
    #[dust_dds(key)]
    pub sequence: u32,
}

impl RequestId {
    pub const fn new(requester_id: [u8; 16], sequence: u32) -> Self {
        Self {
            requester_id,
            sequence,
        }
    }
}

#[derive(DdsType, Debug)]
pub struct ProviderExchange<T: TypeSupport + Send> {
    #[dust_dds(key)]
    pub id: RequestId,
    pub payload: T,
}

impl<T: TypeSupport + Send> ProviderExchange<T> {
    pub fn new(id: RequestId, payload: T) -> Self {
        Self { id, payload }
    }
}
