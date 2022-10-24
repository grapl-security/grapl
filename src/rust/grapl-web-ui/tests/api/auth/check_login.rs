#[actix_web::test]
async fn auth_unauthenticated_check_login() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    let request = app.post("api/auth/checkLogin");
    let response = app.send_with_retries(request).await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::UNAUTHORIZED,
        "unexpected response: {:?}",
        response
    );

    Ok(())
}

#[actix_web::test]
async fn auth_authenticated_check_login() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    app.login_with_test_user().await?;

    let request = app.post("api/auth/checkLogin");
    let response = app.send_with_retries(request).await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        response
    );

    Ok(())
}
