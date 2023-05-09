use std::{future::Future, rc::Rc};

use http::{Response, StatusCode};
use log::debug;
use matchit::Router;
use monoio_http::h1::payload::Payload;
use monolake_core::{
    config::Route,
    http::{HttpError, HttpHandler, Rewrite},
};
use rand::RngCore;
use tower_layer::{layer_fn, Layer};

use crate::http::generate_response;

#[derive(Clone)]
pub struct RewriteHandler<H> {
    inner: H,
    router: Router<Route>,
}

impl<H> RewriteHandler<H> {
    pub fn layer(routes: Rc<Vec<Route>>) -> impl Layer<H, Service = RewriteHandler<H>> {
        let mut router: Router<Route> = Router::new();
        for route in routes.iter() {
            router.insert(route.path.clone(), route.clone()).unwrap();
        }
        layer_fn(move |inner| RewriteHandler {
            inner,
            router: router.clone(),
        })
    }
}

impl<H> HttpHandler for RewriteHandler<H>
where
    H: HttpHandler<Body = Payload> + 'static,
{
    type Body = Payload;
    type Future<'a> = impl Future<Output = Result<Response<Self::Body>, HttpError>> + 'a;
    fn handle(&self, mut request: http::Request<Self::Body>) -> Self::Future<'_> {
        async move {
            let req_path = request.uri().path();
            log::info!("request path: {}", req_path);

            match self.router.at(req_path) {
                Ok(route) => {
                    let route = route.value;
                    log::info!("the route id: {}", route.id);
                    let upstreams = &route.upstreams;
                    let mut rng = rand::thread_rng();
                    let next = rng.next_u32() as usize % upstreams.len();
                    let upstream: &monolake_core::config::Upstream = &upstreams[next];

                    Rewrite::rewrite_request(&mut request, &upstream.endpoint);

                    self.inner.handle(request).await
                }
                Err(e) => {
                    debug!("match request uri: {} with error: {}", request.uri(), e);
                    Ok(generate_response(StatusCode::NOT_FOUND))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use matchit::Router;
    use monolake_core::config::{Endpoint, Route, Upstream, Uri};

    fn iterate_match<'a>(req_path: &str, routes: &'a Vec<Route>) -> Option<&'a Route> {
        let mut target_route = None;
        let mut route_len = 0;
        for route in routes.iter() {
            let route_path = &route.path;
            let route_path_len = route_path.len();
            if req_path.starts_with(route_path) && route_path_len > route_len {
                target_route = Some(route);
                route_len = route_path_len;
            }
        }
        target_route
    }

    fn create_routes() -> impl Iterator<Item = Route> {
        let total_routes = 1024 * 100;
        (0..total_routes).into_iter().map(|n| Route {
            id: "testroute".to_string(),
            path: format!("/{}", n),
            upstreams: Vec::from([Upstream {
                endpoint: Endpoint::Uri(Uri {
                    uri: format!("http://test{}.endpoint", n).parse().unwrap(),
                }),
                weight: Default::default(),
            }]),
        })
    }

    #[test]
    fn test_iterate_match() {
        let mut router: Router<Route> = Router::new();
        create_routes().for_each(|route| router.insert(route.path.clone(), route.clone()).unwrap());
        let routes: Vec<Route> = create_routes().collect();
        let target_path = "/1024";

        let current = SystemTime::now();
        let iterate_route = iterate_match(target_path, &routes).unwrap();
        let iterate_match_elapsed = current.elapsed().unwrap().as_micros();

        let current = SystemTime::now();
        let matchit_route = router.at(target_path).unwrap().value;
        let matchit_match_elapsed = current.elapsed().unwrap().as_micros();

        assert_eq!(
            format!("{:?}", iterate_route),
            format!("{:?}", matchit_route)
        );
        println!("{:?}", iterate_route);
        assert!(matchit_match_elapsed < (iterate_match_elapsed / 100));
    }
}
