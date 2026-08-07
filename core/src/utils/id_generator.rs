use core::sync::atomic::{AtomicU32, Ordering};

use crate::core::messages::RequestId;
use dust_dds::infrastructure::instance::InstanceHandle;

/// A thread-safe atomic sequence generator.
///
/// The sequence is unique within a process.  A sequence alone must not be
/// used to correlate a distributed request; use [`next_request_id`] so the
/// sequence is scoped by the requester's DDS identity.
pub struct AtomicIdGenerator {
    counter: AtomicU32,
}

impl AtomicIdGenerator {
    /// Creates a new `AtomicIdGenerator` starting from 0.
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
        }
    }

    /// Creates a new `AtomicIdGenerator` starting from a specific value.
    pub const fn with_start(start: u32) -> Self {
        Self {
            counter: AtomicU32::new(start),
        }
    }

    /// Generates the next sequence value.
    ///
    /// This method is thread-safe and unique across all threads using this
    /// generator until overflow. It is not globally unique across processes.
    pub fn next_id(&self) -> u32 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Returns the current value without incrementing.
    pub fn current(&self) -> u32 {
        self.counter.load(Ordering::Relaxed)
    }

    /// Resets the counter to 0.
    pub fn reset(&self) {
        self.counter.store(0, Ordering::Relaxed);
    }
}

impl Default for AtomicIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Process-local sequence generator used as one component of a request ID.
static GLOBAL_REQUEST_ID_GENERATOR: AtomicIdGenerator = AtomicIdGenerator::new();

/// Generates a request ID scoped by the requester's DDS reader identity.
///
/// DDS reader handles include the participant GUID and are therefore unique
/// across processes. Combining that identity with a process-local sequence
/// prevents two consumers that both issue their first request from sharing a
/// correlation ID.
pub fn next_request_id(requester: InstanceHandle) -> RequestId {
    RequestId::new(requester.into(), GLOBAL_REQUEST_ID_GENERATOR.next_id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_ids() {
        let generator = AtomicIdGenerator::new();
        assert_eq!(generator.next_id(), 0);
        assert_eq!(generator.next_id(), 1);
        assert_eq!(generator.next_id(), 2);
    }

    #[test]
    fn test_with_start() {
        let generator = AtomicIdGenerator::with_start(100);
        assert_eq!(generator.next_id(), 100);
        assert_eq!(generator.next_id(), 101);
    }

    #[test]
    fn test_current() {
        let generator = AtomicIdGenerator::new();
        assert_eq!(generator.current(), 0);
        generator.next_id();
        assert_eq!(generator.current(), 1);
    }

    #[test]
    fn test_reset() {
        let generator = AtomicIdGenerator::new();
        generator.next_id();
        generator.next_id();
        generator.reset();
        assert_eq!(generator.current(), 0);
    }

    #[test]
    fn request_ids_are_scoped_by_requester() {
        let first = RequestId::new([1; 16], 0);
        let second = RequestId::new([2; 16], 0);

        assert_ne!(first, second);
        assert_eq!(first.sequence, second.sequence);
        assert_ne!(first.requester_id, second.requester_id);
    }

    #[test]
    fn next_request_id_preserves_reader_identity() {
        let requester = InstanceHandle::new([7; 16]);
        let request_id = next_request_id(requester);

        assert_eq!(request_id.requester_id, [7; 16]);
    }
}
