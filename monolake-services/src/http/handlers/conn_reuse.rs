use std::future::Future;

use http::{HeaderValue, Response, StatusCode, Version};
use log::debug;
use monoio_http::h1::payload::Payload;
use monolake_core::{
    http::{HttpError, HttpHandler},
    service::ServiceLayer,
};
use tower_layer::{layer_fn, Layer};

use crate::http::{generate_response, is_conn_reuse, CONN_CLOSE, CONN_KEEP_ALIVE};

#[derive(Clone)]
pub struct ConnReuseHandler<H> {
    inner: H,
}

impl<H> ServiceLayer<H> for ConnReuseHandler<H> {
    type Param = ();
    type Layer = impl Layer<H, Service = ConnReuseHandler<H>>;
    fn layer(_: Self::Param) -> Self::Layer {
        layer_fn(move |inner| ConnReuseHandler { inner })
    }
}

impl<H> HttpHandler for ConnReuseHandler<H>
where
    H: HttpHandler<Body = Payload> + 'static,
{
    type Body = Payload;
    type Future<'a> = impl Future<Output = Result<Response<Self::Body>, HttpError>> + 'a;
    fn handle(&self, mut request: http::Request<Self::Body>) -> Self::Future<'_> {
        async move {
            let version = request.version();
            let reuse_conn = is_conn_reuse(request.headers(), version);
            debug!("frontend conn reuse {:?}", reuse_conn);
            // update the http request to make sure the backend conn reuse
            *request.version_mut() = Version::HTTP_11;
            let _ = request.headers_mut().remove(http::header::CONNECTION);

            match self.inner.handle(request).await {
                Ok(mut response) => {
                    let header_value = if reuse_conn {
                        unsafe { HeaderValue::from_maybe_shared_unchecked(CONN_KEEP_ALIVE) }
                    } else {
                        unsafe { HeaderValue::from_maybe_shared_unchecked(CONN_CLOSE) }
                    };
                    response
                        .headers_mut()
                        .insert(http::header::CONNECTION, header_value);
                    *response.version_mut() = version;
                    Ok(response)
                }
                Err(_e) => Ok(generate_response(StatusCode::INTERNAL_SERVER_ERROR)),
            }
        }
    }
}
