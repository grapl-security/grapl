use std::ops::Deref;
use actix_web::{
    post,
    web,
    HttpRequest,
    HttpResponse,
    Result,
};
use crate::authn::AuthenticatedUser;

// We have a new type for this to differentiate between the URL for this backend service and that
// for others
#[derive(Clone, Debug)]
pub(crate) struct PluginRegistryEndpointUrl(url::Url);

impl From<url::Url> for PluginRegistryEndpointUrl {
    fn from(u: url::Url) -> Self {
        Self(u)
    }
}

impl Deref for PluginRegistryEndpointUrl {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(handler);
}

// TODO: use 'tail' do rebuild the path to backend (drop outer scope path)
#[post("/{tail:.*}")]
pub(crate) async fn handler(
    req: HttpRequest,
    payload: web::Payload,
    backend_url: web::Data<PluginRegistryEndpointUrl>,
    client: web::Data<awc::Client>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let url = backend_url.get_ref().deref().clone();

    super::fwd_request_to_backend_service(req, payload, url, client.get_ref().clone()).await
}


// pub(crate) fn config(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::resource("/plugin_registry")
//             .route(web::post().to(create_plugin_post))
//             .guard(guard::Post())
//             .guard(guard::Header("content-type", "application/json")), // .guard(guard::Header("X-Requested-With", "XMLHttpRequest")),
//     );
// }

// async fn create_plugin_post<T>(
//     req: HttpRequest,
//     payload: web::Payload,
//     // backend_url: web::Data<ModelPluginDeployerEndpoint>,
//     client: web::Data<PluginRegistryServiceClient<T>>,
//     _user: AuthenticatedUser,
// ) -> Result<HttpResponse>
//     // T:
// {
//     tracing::debug!(
//         env=?std::env::args(),
//     );

//     let mut client = PluginRegistryServiceClient::from_env().await?;
//     let tenant_id = uuid::Uuid::new_v4();

//     let request = CreatePluginRequest {
//         plugin_artifact: b"???????".to_vec(),
//         tenant_id, // todo(AP - Add Tenant ID)
//         display_name: "test_for_now".to_owned(),
//         plugin_type: PluginType::Generator,
//     };

//     let response = client
//         .create_plugin(request)
//         .timeout(std::time::Duration::from_secs(5))
//         .await??;

//     let plugin_id = response.plugin_id;

//     let get_response: GetPluginResponse = client
//         .get_plugin(GetPluginRequest {
//             plugin_id,
//             tenant_id,
//         })
//         .timeout(std::time::Duration::from_secs(5))
//         .await??;

//     todo!()
// }
