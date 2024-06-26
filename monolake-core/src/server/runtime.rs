use std::future::Future;

#[cfg(target_os = "linux")]
use monoio::IoUringDriver;

#[cfg(target_os = "linux")]
const MIN_SQPOLL_IDLE_TIME: u32 = 1000;

use monoio::{time::TimeDriver, LegacyDriver, Runtime, RuntimeBuilder};

use crate::config::{RuntimeConfig, RuntimeType};

pub enum RuntimeWrapper {
    #[cfg(target_os = "linux")]
    IoUring(Runtime<TimeDriver<IoUringDriver>>),
    Legacy(Runtime<TimeDriver<LegacyDriver>>),
}

impl RuntimeWrapper {
    pub fn new(
        _config: &RuntimeConfig,
        thread_pool: Option<Box<dyn monoio::blocking::ThreadPool + Send + 'static>>,
    ) -> Self {
        #[cfg(target_os = "linux")]
        let runtime_type =
            if _config.runtime_type == RuntimeType::IoUring && monoio::utils::detect_uring() {
                RuntimeType::IoUring
            } else {
                RuntimeType::Legacy
            };
        #[cfg(not(target_os = "linux"))]
        let runtime_type = RuntimeType::Legacy;

        match runtime_type {
            #[cfg(target_os = "linux")]
            RuntimeType::IoUring => {
                let builder = match _config.sqpoll_idle {
                    Some(idle) => {
                        let builder = RuntimeBuilder::<monoio::IoUringDriver>::new();
                        let idle = MIN_SQPOLL_IDLE_TIME.max(idle);
                        let mut uring_builder = io_uring::IoUring::builder();
                        uring_builder.setup_sqpoll(idle);
                        builder.uring_builder(uring_builder)
                    }
                    None => RuntimeBuilder::<monoio::IoUringDriver>::new(),
                };
                let mut builder = builder.enable_timer().with_entries(_config.entries);
                if let Some(tp) = thread_pool {
                    builder = builder.attach_thread_pool(tp);
                }
                let runtime = builder.build().unwrap();
                RuntimeWrapper::IoUring(runtime)
            }
            RuntimeType::Legacy => {
                let mut builder = RuntimeBuilder::<monoio::LegacyDriver>::new().enable_timer();
                if let Some(tp) = thread_pool {
                    builder = builder.attach_thread_pool(tp);
                }
                let runtime = builder.build().unwrap();
                RuntimeWrapper::Legacy(runtime)
            }
        }
    }
}

impl RuntimeWrapper {
    pub fn block_on<F>(&mut self, future: F) -> F::Output
    where
        F: Future,
    {
        match self {
            #[cfg(target_os = "linux")]
            RuntimeWrapper::IoUring(driver) => driver.block_on(future),
            RuntimeWrapper::Legacy(driver) => driver.block_on(future),
        }
    }
}
