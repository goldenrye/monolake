#![feature(impl_trait_in_assoc_type)]

pub mod config;
pub mod http;
pub mod service;
pub mod tls;
pub mod util;

use std::num::NonZeroUsize;

use figlet_rs::FIGfont;

pub const MAX_CONFIG_SIZE_LIMIT: usize = 8072;
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
pub const MIN_SQPOLL_IDLE_TIME: u32 = 1000; // 1s idle time.

pub trait Builder<Config> {
    fn build_with_config(config: Config) -> Self;
}

pub fn print_logo() {
    let standard_font = FIGfont::standard().unwrap();
    if let Some(figure) = standard_font.convert("Monolake") {
        println!("{}", figure);
    }
}

pub fn max_parallel_count() -> NonZeroUsize {
    std::thread::available_parallelism().unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) })
}
