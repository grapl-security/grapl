use grapl_web_ui::routes::api::plugin::{
    create::CreateResponse,
    get_deployment::PluginDeploymentResponse,
    get_metadata::GetPluginMetadataResponse,
};

use crate::test_app::TestApp;

#[actix_web::test]
async fn plugin_lifecycle() -> eyre::Result<()> {
    let app = TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";

    let create_response = create_plugin(&app, plugin_name).await?;

    let plugin_metadata = get_plugin_metadata(&app, &create_response.plugin_id).await?;

    eyre::ensure!(
        create_response.plugin_id == plugin_metadata.plugin_id,
        "unexpected plugin_id: {:?}",
        plugin_metadata.plugin_id
    );

    let plugin_id = create_response.plugin_id;

    eyre::ensure!(
        plugin_name == plugin_metadata.display_name,
        "get_plugin_metadata returned unexpected name: {}, should be {plugin_name}",
        plugin_metadata.display_name
    );

    deploy_plugin(&app, &plugin_id).await?;

    let deployment_status = get_deployment(&app, &plugin_id).await?;

    eyre::ensure!(
        deployment_status.deployed,
        "plugin not deployed :("
    );

    eyre::ensure!(
        deployment_status.status == "success",
        "plugin deployment not successful"
    );

    // cool, now tear it down
    tear_down(&app, &plugin_id).await?;

    eyre::ensure!(
        !deployment_status.deployed,
        "plugin still deployed after tear_down :("
    );

    Ok(())
}

async fn create_plugin(app: &TestApp, plugin_name: &str) -> eyre::Result<CreateResponse> {
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

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    let response_body = response.json::<CreateResponse>().await?;

    println!("create response body: {:?}", response_body);

    Ok(response_body)
}

async fn get_plugin_metadata(
    app: &TestApp,
    plugin_id: &uuid::Uuid,
) -> eyre::Result<GetPluginMetadataResponse> {
    let response = app
        .get(format!("api/plugin/get_metadata?plugin_id={plugin_id}").as_str())
        .send()
        .await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    let response_body = response.json::<GetPluginMetadataResponse>().await?;

    println!("metadata response body: {:?}", response_body);

    Ok(response_body)
}

async fn deploy_plugin(app: &TestApp, plugin_id: &uuid::Uuid) -> eyre::Result<()> {
    let body = serde_json::json!({
            "plugin_id": plugin_id,
    });

    let response = app.post("api/plugin/deploy").json(&body).send().await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
}

async fn get_deployment(
    app: &TestApp,
    plugin_id: &uuid::Uuid,
) -> eyre::Result<PluginDeploymentResponse> {
    let response = app
        .get(format!("api/plugin/get_deployment?plugin_id={plugin_id}").as_str())
        .send()
        .await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    let response_body = response.json::<PluginDeploymentResponse>().await?;

    println!("metadata response body: {:?}", response_body);

    Ok(response_body)
}

async fn tear_down(app: &TestApp,
    plugin_id: &uuid::Uuid,) -> eyre::Result<()> {
        let body = serde_json::json!({
            "plugin_id": plugin_id,
    });

    let response = app.post("api/plugin/tear_down").json(&body).send().await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
    }