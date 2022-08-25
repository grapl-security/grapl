
#[actix_web::test]
async fn plugin_create() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    app.login_with_test_user().await?;

    let plugin_artifact = reqwest::multipart::Part::bytes("junk".as_bytes());
    let form = reqwest::multipart::Form::new().part("test_plugin", plugin_artifact);

    let response = app
        .post("api/plugin/create")
        .multipart(form)
        .send()
        .await?;

    let status = response.status();

    eyre::ensure!(
        status == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        &response
    );

    Ok(())
}