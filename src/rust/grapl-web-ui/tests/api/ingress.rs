#![cfg(feature = "integration_tests")]

use eyre::ContextCompat;

use crate::test_app::TestApp;

#[actix_web::test]
async fn publish_log() -> eyre::Result<()> {
    let app = TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_name = "Test Plugin";

    let create_response = crate::plugin::create_plugin(&app, plugin_name).await?;

    //TODO: this shouldn't be necessary, but we're seeing 500 errors without it.
    // I'll file a task to look into it for now so we can unblock frontend work.
    std::thread::sleep(std::time::Duration::from_secs(5));

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
