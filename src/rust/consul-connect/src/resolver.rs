use std::str::FromStr;

use trust_dns_resolver::{
    error::ResolveError,
    proto::{
        error::ProtoError as ProtocolError,
        rr::rdata::SRV,
    },
    Name,
    TokioAsyncResolver,
};

#[derive(thiserror::Error, Debug)]
pub enum ConsulConnectResolveError {
    #[error("Failed to resolve name {name}")]
    EmptyResolution { name: String },
    #[error("Failed to resolve plugin {0}")]
    ResolveError(#[from] ResolveError),
    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
}

pub struct ResolvedConsulConnectService {
    pub domain: String,
    pub port: u16,
}

#[derive(Clone)]
pub struct ConsulConnectResolver {
    /// An in-process DNS resolver used for plugin service discovery
    resolver: TokioAsyncResolver,
}

impl ConsulConnectResolver {
    /// Given a Consul Connect service name, use SRV domain lookups to find out
    /// its domain and port.
    #[tracing::instrument(skip(self))]
    pub async fn resolve_service(
        &self,
        service_name: String,
    ) -> Result<ResolvedConsulConnectService, ConsulConnectResolveError> {
        let domain = format!("{}.service.consul.", &service_name);
        tracing::info!(
            message = "Resolving domain",
            domain = %domain,
        );
        let lowest_pri = self.resolve_lowest_pri(Name::from_str(&domain)?).await?;

        Ok(ResolvedConsulConnectService {
            domain: lowest_pri.target().to_string(),
            port: lowest_pri.port(),
        })
    }

    /// Performs the SRV record lookup, returning the record with the lowest priority
    async fn resolve_lowest_pri(&self, name: Name) -> Result<SRV, ConsulConnectResolveError> {
        let srvs = self.resolver.srv_lookup(name.clone()).await?;

        let lowest_priority = srvs.iter().min_by_key(|srv| srv.priority());

        match lowest_priority {
            None => Err(ConsulConnectResolveError::EmptyResolution {
                name: name.to_string(),
            }),
            Some(lowest_priority) => Ok((*lowest_priority).clone()),
        }
    }
}
