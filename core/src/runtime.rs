//! Runtime policies and primitive contracts used by the framework.
//!
//! [`RuntimeContext`] deliberately remains coupled to DustDDS. A context chooses the
//! [`dust_dds::runtime::DdsRuntime`] used by the participant factory and supplies the
//! framework primitives that DustDDS does not expose publicly. The concrete
//! standard-runtime implementation is added separately; this module currently defines
//! factory access plus the timer, mutex, and future-selection contracts needed by core.

use core::future::Future;
use core::ops::DerefMut;

use dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync;
use dust_dds::runtime::DdsRuntime;

/// The timer handle selected by a [`RuntimeContext`]'s DustDDS runtime.
///
/// The handle implements [`dust_dds::runtime::Timer`]. Its `delay` method
/// accepts [`core::time::Duration`], regardless of the underlying timer
/// implementation.
pub type TimerHandleOf<C> = <<C as RuntimeContext>::DdsRuntime as DdsRuntime>::TimerHandle;

/// The mutex implementation selected by a [`RuntimeContext`] for `T`.
pub type MutexOf<C, T> = <C as RuntimeContext>::Mutex<T>;

/// The result of racing two futures.
///
/// Unlike the result returned by a particular future-combinator crate, this enum
/// contains only the value produced by the winning future. The losing future is
/// owned by the selection operation and is dropped before the operation resolves.
pub enum SelectResult<A, B> {
    /// The first future completed.
    First(A),
    /// The second future completed.
    Second(B),
}

/// Contract for an asynchronous mutex supplied by a runtime context.
///
/// The guard and lock future are generic over the borrow of the mutex so a guard
/// can remain alive across an await point. Implementations must ensure that the
/// guard releases the lock when dropped.
pub trait RuntimeMutex<T>: Send + Sync + 'static
where
    T: Send + 'static,
{
    /// The guard returned after the mutex is acquired.
    type Guard<'a>: DerefMut<Target = T> + Send + 'a
    where
        Self: 'a;

    /// Asynchronously acquire the mutex.
    fn lock(&self) -> impl Future<Output = Self::Guard<'_>> + Send;
}

/// A framework runtime policy tied to a DustDDS runtime.
pub trait RuntimeContext: Send + Sync + 'static {
    /// The DustDDS runtime used by the participant factory for this context.
    type DdsRuntime: DdsRuntime;

    /// The mutex family supplied by this context.
    type Mutex<T>: RuntimeMutex<T>
    where
        T: Send + 'static;

    /// Obtain the DDS participant factory selected by this context.
    fn get_dds_factory(&self) -> &DomainParticipantFactoryAsync<Self::DdsRuntime>;

    /// Obtain a timer handle from this context.
    fn timer(&self) -> TimerHandleOf<Self>;

    /// Construct a mutex containing `value`.
    fn mutex<T>(&self, value: T) -> MutexOf<Self, T>
    where
        T: Send + 'static;

    /// Race two futures using this context's selection policy.
    fn select<A, B>(
        &self,
        first: A,
        second: B,
    ) -> impl Future<Output = SelectResult<A::Output, B::Output>> + Send
    where
        A: Future + Send,
        B: Future + Send,
        A::Output: Send,
        B::Output: Send;
}
