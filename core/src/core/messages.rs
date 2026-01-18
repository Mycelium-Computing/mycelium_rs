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

#[derive(DdsType, Debug)]
pub struct ProviderExchange<T: TypeSupport + Send> {
    pub id: u32,
    pub payload: T,
}

impl<T: TypeSupport + Send> ProviderExchange<T> {
    pub fn new(id: u32, payload: T) -> Self {
        Self { id, payload }
    }
}
