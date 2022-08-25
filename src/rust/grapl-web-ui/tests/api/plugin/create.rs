use grapl_web_ui::routes::api::plugin::{
    create::CreateResponse,
    get_metadata::GetPluginMetadataResponse,
};

#[actix_web::test]
async fn plugin_create() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";
    let create_metadata_body = serde_json::json!({
            "plugin_name": plugin_name,
            "plugin_type": "generator",
            "event_source_id": "00000000-0000-0000-0000-000000000000"
    });
    let form = reqwest::multipart::Form::new()
        .part(
            "metadata",
            reqwest::multipart::Part::text(create_metadata_body.to_string()),
        )
        .part(
            "plugin_artifact",
            reqwest::multipart::Part::bytes("junk".as_bytes()),
        );

    let response = app.post("api/plugin/create").multipart(form).send().await?;

    let status = response.status();

    eyre::ensure!(
        status == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    let response_body = response.json::<CreateResponse>().await?;

    println!("create response body: {:?}", response_body);

    let plugin_id = response_body.plugin_id;

    // ---------------------
    // Get Plugin metadata
    let response = app
        .get(format!("api/plugin/get_metadata?plugin_id={plugin_id}").as_str())
        .send()
        .await?;

    let status = response.status();

    eyre::ensure!(
        status == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    let response_body = response.json::<GetPluginMetadataResponse>().await?;

    println!("metadata response body: {:?}", response_body);

    eyre::ensure!(
        plugin_id == response_body.plugin_id,
        "unexpected plugin_id: {:?}",
        response_body.plugin_id
    );

    eyre::ensure!(
        plugin_name == response_body.display_name,
        "incorrect name: {}, should be {plugin_name}",
        response_body.display_name
    );

    Ok(())
}
