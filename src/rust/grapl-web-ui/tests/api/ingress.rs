#![cfg(feature = "integration_tests")]

use eyre::ContextCompat;
use grapl_web_ui::routes::api::plugin::create::CreateResponse;

use crate::test_app::TestApp;

#[actix_web::test]
async fn publish_log1() -> eyre::Result<()> {
    let app = TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";

    let create_response = create_plugin(&app, plugin_name).await?;

    let plugin_metadata =
        crate::plugin::get_plugin_metadata(&app, &create_response.plugin_id).await?;

    let event_source_id = plugin_metadata
        .event_source_id
        .wrap_err("new plugin is missing event_source_id")?;

    let response = app
        .post(format!("api/ingress/publish/{event_source_id}").as_str())
        .body("junk data")
        .send()
        .await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
}

#[actix_web::test]
async fn publish_log2() -> eyre::Result<()> {
    let app = TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";

    let create_response = create_plugin(&app, plugin_name).await?;

    let plugin_metadata =
        crate::plugin::get_plugin_metadata(&app, &create_response.plugin_id).await?;

    let event_source_id = plugin_metadata
        .event_source_id
        .wrap_err("new plugin is missing event_source_id")?;

    let response = app
        .post(format!("api/ingress/publish/{event_source_id}").as_str())
        .body("junk data")
        .send()
        .await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
}

#[actix_web::test]
async fn publish_log3() -> eyre::Result<()> {
    let app = TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";

    let create_response = create_plugin(&app, plugin_name).await?;

    let plugin_metadata =
        crate::plugin::get_plugin_metadata(&app, &create_response.plugin_id).await?;

    let event_source_id = plugin_metadata
        .event_source_id
        .wrap_err("new plugin is missing event_source_id")?;

    let response = app
        .post(format!("api/ingress/publish/{event_source_id}").as_str())
        .body("junk data")
        .send()
        .await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
}

#[actix_web::test]
async fn publish_log4() -> eyre::Result<()> {
    let app = TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";

    let create_response = create_plugin(&app, plugin_name).await?;

    let plugin_metadata =
        crate::plugin::get_plugin_metadata(&app, &create_response.plugin_id).await?;

    let event_source_id = plugin_metadata
        .event_source_id
        .wrap_err("new plugin is missing event_source_id")?;

    let response = app
        .post(format!("api/ingress/publish/{event_source_id}").as_str())
        .body("junk data")
        .send()
        .await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
}

#[actix_web::test]
async fn publish_log5() -> eyre::Result<()> {
    let app = TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";

    let create_response = create_plugin(&app, plugin_name).await?;

    let plugin_metadata =
        crate::plugin::get_plugin_metadata(&app, &create_response.plugin_id).await?;

    let event_source_id = plugin_metadata
        .event_source_id
        .wrap_err("new plugin is missing event_source_id")?;

    let response = app
        .post(format!("api/ingress/publish/{event_source_id}").as_str())
        .body("junk data")
        .send()
        .await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
}

#[actix_web::test]
async fn publish_log6() -> eyre::Result<()> {
    let app = TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";

    let create_response = create_plugin(&app, plugin_name).await?;

    let plugin_metadata =
        crate::plugin::get_plugin_metadata(&app, &create_response.plugin_id).await?;

    let event_source_id = plugin_metadata
        .event_source_id
        .wrap_err("new plugin is missing event_source_id")?;

    let response = app
        .post(format!("api/ingress/publish/{event_source_id}").as_str())
        .body("junk data")
        .send()
        .await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
}

pub async fn create_plugin(app: &TestApp, plugin_name: &str) -> eyre::Result<CreateResponse> {
    let create_metadata_body = serde_json::json!({
            "plugin_name": plugin_name,
            "plugin_type": "generator",
            "event_source_id": uuid::Uuid::new_v4()
    });

    let generator_bytes = "junk".as_bytes();

    let form = reqwest::multipart::Form::new()
        .part(
            "metadata",
            reqwest::multipart::Part::text(create_metadata_body.to_string()),
        )
        .part(
            "plugin_artifact",
            reqwest::multipart::Part::bytes(generator_bytes.to_vec()),
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
