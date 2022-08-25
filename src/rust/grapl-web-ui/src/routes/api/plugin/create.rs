mod error;
use std::sync::Mutex;

use actix_multipart::{
    Multipart,
};
use actix_web::web;
use error::WebResponseError;
use futures::{
    StreamExt,
    TryStreamExt,
};
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    PluginMetadata,
    PluginRegistryServiceClient,
    PluginType,
};

// #[derive(serde::Deserialize)]
// pub(super) struct CreateParameters {
//     plugin_name: String,
// }

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct CreateResponse {
    pub plugin_id: String,
}

#[tracing::instrument(skip(plugin_registry_client, user, payload), fields(
    username = tracing::field::Empty
))]
pub(super) async fn create(
    plugin_registry_client: web::Data<Mutex<PluginRegistryServiceClient>>,
    user: crate::authn::AuthenticatedUser,
    // data: web::Json<CreateParameters>,
    mut payload: Multipart,
) -> Result<impl actix_web::Responder, WebResponseError> {

    let tenant_id = user.get_organization_id();

    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field.content_disposition();

        let plugin_name = content_disposition
            .get_name()
            .ok_or(WebResponseError::MissingName)?;

        tracing::debug!(
            message = "creating plugin",
            username = user.get_username(),
            plugin_name,
        );

        // Hack, take this as a field
        let event_source_id = Some(uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000")?);

        let metadata = PluginMetadata::new(tenant_id.to_owned(), plugin_name.to_string(), PluginType::Generator, event_source_id);

        //TODO: because PluginRegistryServiceClient::create_plugin doesn't provide us a way
        // to abort if our multipart stream fails, we need can't just pass the stream along,
        // we need to fully get the plugin artifact before we can't start sending upstream.
        // Note: create_plugin takes `Stream<Item = Bytes>`. If it took a Item=<Result<Bytes, ...>>
        // then we could at least map our Err type and pass the stream along without needing
        // fully terminate our client stream before starting this next stream.

        // Gah fuck, writing to disk won't even work, we need the whole thing in memory.
        // Instead of working around this I'm just going to only send the first chunk we
        // get and stop after that for now. We can fix this when upstream supports aborting.
        // let temp_file = tempfile::tempfile()?;
        // while let Some(chunk) = field.next().await {
        //     let data = chunk?;
        //     temp_file.write_all(&data)?;
        // }
        if let Some(first_chunk) = field.next().await {
            let first_chunk = first_chunk?;

            let plugin_artifact_stream = futures::stream::once(async move { first_chunk });

            let mut plugin_registry_client = plugin_registry_client.lock().unwrap();
            let response = plugin_registry_client
                .create_plugin(metadata, plugin_artifact_stream)
                .await?;

            let plugin_id = response.plugin_id();

            return Ok(actix_web::HttpResponse::Ok().json(CreateResponse {
                plugin_id: plugin_id.to_string(),
            }));
        }

        // At this point we have all file contents from the user

        // Field in turn is stream of *Bytes* object
        // while let Some(chunk) = field.try_next().await? {
        //     // filesystem operations are blocking, we have to use threadpool
        //     f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        // // }
        // let mut plugin_registry_client = plugin_registry_client.lock().unwrap();

        // use futures::stream;
        // let plugin_artifact = field.map_ok(|a| stream::iter(a));
        // // let plugin_artifact = field.map(Result::unwrap);
        // // let plugin_artifact = to_chunks(field);

        // let response = plugin_registry_client
        //     .create_plugin(metadata, plugin_artifact)
        //     .await?;

        // let plugin_id = response.plugin_id();

        // return Ok(actix_web::HttpResponse::Ok().json(CreateResponse {
        //     plugin_id: plugin_id.to_string(),
        // }));
    }

    Err(WebResponseError::MissingName)
}
