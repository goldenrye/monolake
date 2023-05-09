// pub mod handler;
mod rewrite;
use std::future::Future;

use http::{Request, Response};
pub use rewrite::Rewrite;

pub type HttpError = anyhow::Error;

pub trait HttpHandler: Clone {
    type Body;
    type Future<'a>: Future<Output = Result<Response<Self::Body>, HttpError>>
    where
        Self: 'a;

    fn handle(&self, request: Request<Self::Body>) -> Self::Future<'_>;
}
