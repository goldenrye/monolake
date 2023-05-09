mod servers;
mod tcp_server;
mod uds_server;
use std::future::Future;

use anyhow::{bail, Result};
pub use servers::Servers;

use self::{tcp_server::TcpServer, uds_server::UdsServer};

pub trait Server {
    type ServeFuture<'a>: Future<Output = Result<()>>
    where
        Self: 'a;
    type InitFuture<'a>: Future<Output = Result<()>>
    where
        Self: 'a;

    fn serve(&self) -> Self::ServeFuture<'_>;

    fn init(&mut self) -> Self::InitFuture<'_>;
}

#[derive(Debug, Clone)]
pub enum ServerWrapper {
    TcpServer(TcpServer),
    UdsServer(UdsServer),
    Unknown,
}

impl Server for ServerWrapper {
    type ServeFuture<'a> = impl Future<Output = Result<()>> + 'a
    where
        Self: 'a;
    type InitFuture<'a> = impl Future<Output = Result<()>> + 'a
        where
            Self: 'a;

    fn serve(&self) -> Self::ServeFuture<'_> {
        async move {
            match self {
                ServerWrapper::TcpServer(server) => server.serve().await,
                ServerWrapper::UdsServer(server) => server.serve().await,
                ServerWrapper::Unknown => bail!("unimplement!"),
            }
        }
    }

    fn init(&mut self) -> Self::InitFuture<'_> {
        async move {
            match self {
                ServerWrapper::TcpServer(server) => server.init().await,
                ServerWrapper::UdsServer(server) => server.init().await,
                ServerWrapper::Unknown => bail!("unimplement!"),
            }
        }
    }
}
