use std::{future::Future, rc::Rc};

use http::{Response, StatusCode};
use monoio_http::h1::payload::Payload;
use monoio_http_client::Client;
use monolake_core::http::{HttpError, HttpHandler};

use crate::http::generate_response;

#[derive(Clone)]
pub struct ProxyHandler {
    client: Rc<Client>,
}

impl ProxyHandler {
    pub fn new(client: Rc<Client>) -> Self {
        ProxyHandler { client }
    }
}

impl HttpHandler for ProxyHandler {
    type Body = Payload;
    type Future<'a> = impl Future<Output = Result<Response<Self::Body>, HttpError>> + 'a;
    fn handle(&self, request: http::Request<Self::Body>) -> Self::Future<'_> {
        async move {
            match self.client.send(request).await {
                Ok(resp) => Ok(resp),
                Err(_e) => Ok(generate_response(StatusCode::INTERNAL_SERVER_ERROR)),
            }
        }
    }
}
