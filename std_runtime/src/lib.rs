#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::future::Future;

use mycelium_computing::runtime::RuntimeMutex;

#[cfg(feature = "std")]
use dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync;
#[cfg(feature = "std")]
use mycelium_computing::runtime::{MutexOf, RuntimeContext, SelectResult, TimerHandleOf};

/// An asynchronous mutex adapter for the standard runtime package.
pub struct StdMutex<T>(async_lock::Mutex<T>);

impl<T> StdMutex<T> {
    /// Creates a mutex containing `value`.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(async_lock::Mutex::new(value))
    }

    /// Consumes the adapter and returns the contained value.
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

impl<T> From<T> for StdMutex<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> RuntimeMutex<T> for StdMutex<T>
where
    T: Send + 'static,
{
    type Guard<'a>
        = async_lock::MutexGuard<'a, T>
    where
        Self: 'a;

    fn lock(&self) -> impl Future<Output = Self::Guard<'_>> + Send {
        self.0.lock()
    }
}

/// A [`RuntimeContext`] backed by DustDDS's standard runtime.
///
/// The context uses DustDDS's standard participant-factory singleton and owns a timer driver
/// for framework-level delays. The factory singleton owns its own internal DustDDS runtime;
/// DustDDS exposes the runtime as a type parameter rather than exposing that singleton's
/// runtime instance. Consequently, this context guarantees type compatibility with the
/// factory, while its framework timer is a separately owned standard timer driver.
#[cfg(feature = "std")]
pub struct StdRuntimeContext {
    factory: &'static DomainParticipantFactoryAsync<dust_dds::std_runtime::StdRuntime>,
    timer_driver: dust_dds::std_runtime::timer::TimerDriver,
}

#[cfg(feature = "std")]
impl StdRuntimeContext {
    /// Creates a standard runtime context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            factory: DomainParticipantFactoryAsync::get_instance(),
            timer_driver: dust_dds::std_runtime::timer::TimerDriver::new(),
        }
    }
}

#[cfg(feature = "std")]
impl Default for StdRuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl RuntimeContext for StdRuntimeContext {
    type DdsRuntime = dust_dds::std_runtime::StdRuntime;
    type Mutex<T: Send + 'static> = StdMutex<T>;

    fn get_dds_factory(&self) -> &DomainParticipantFactoryAsync<Self::DdsRuntime> {
        self.factory
    }

    fn timer(&self) -> TimerHandleOf<Self> {
        self.timer_driver.handle()
    }

    fn mutex<T>(&self, value: T) -> MutexOf<Self, T>
    where
        T: Send + 'static,
    {
        StdMutex::new(value)
    }

    fn select<A, B>(
        &self,
        first: A,
        second: B,
    ) -> impl Future<Output = SelectResult<A::Output, B::Output>> + Send
    where
        A: Future + Send,
        B: Future + Send,
        A::Output: Send,
        B::Output: Send,
    {
        async move {
            futures::pin_mut!(first, second);

            match futures::future::select(first, second).await {
                futures::future::Either::Left((output, _)) => SelectResult::First(output),
                futures::future::Either::Right((output, _)) => SelectResult::Second(output),
            }
        }
    }
}
