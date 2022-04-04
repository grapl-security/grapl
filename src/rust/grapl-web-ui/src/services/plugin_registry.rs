use actix_web::{guard, web, Result, HttpResponse, HttpRequest};
use grapl_utils::future_ext::GraplFutureExt;
use plugin_registry::client::PluginRegistryServiceClient;
use rust_proto::plugin_registry::{CreatePluginRequest, GetPluginRequest, GetPluginResponse, PluginType};
use crate::authn::AuthenticatedUser;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/plugin_registry")
            .route(web::post().to(create_plugin_post))
            .guard(guard::Post())
            .guard(guard::Header("content-type", "application/json")), // .guard(guard::Header("X-Requested-With", "XMLHttpRequest")),
    );
}

async fn create_plugin_post<T>(
    req: HttpRequest,
    payload: web::Payload,
    // backend_url: web::Data<ModelPluginDeployerEndpoint>,
    client: web::Data<PluginRegistryServiceClient<T>>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse>
    // T:
{
    tracing::debug!(
        env=?std::env::args(),
    );

    let mut client = PluginRegistryServiceClient::from_env().await?;
    let tenant_id = uuid::Uuid::new_v4();

    let request = CreatePluginRequest {
        plugin_artifact: b"???????".to_vec(),
        tenant_id, // todo(AP - Add Tenant ID)
        display_name: "test_for_now".to_owned(),
        plugin_type: PluginType::Generator,
    };

    let response = client
        .create_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let plugin_id = response.plugin_id;

    let get_response: GetPluginResponse = client
        .get_plugin(GetPluginRequest {
            plugin_id,
            tenant_id,
        })
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    todo!()
}
