#[actix_web::test]
async fn auth_password_incorrect_password() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    let request = app
        .post("api/auth/sign_in_with_password")
        .json(&serde_json::json!({
            "username": app.test_user.username,
            "password": "nope",
        }));
    let response = app.send_with_retries(request).await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::UNAUTHORIZED,
        "unexpected response: {:?}",
        response
    );

    Ok(())
}

#[actix_web::test]
async fn auth_password_nonexistent_user() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    let request = app
        .post("api/auth/sign_in_with_password")
        .json(&serde_json::json!({
            "username": "nope",
            "password": "nope",
        }));
    let response = app.send_with_retries(request).await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::UNAUTHORIZED,
        "unexpected response: {:?}",
        response
    );

    Ok(())
}

#[actix_web::test]
async fn auth_password_empty_creds() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    let request = app
        .post("api/auth/sign_in_with_password")
        .json(&serde_json::json!({
            "username": "",
            "password": "",
        }));
    let response = app.send_with_retries(request).await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::UNAUTHORIZED,
        "unexpected response: {:?}",
        response
    );

    Ok(())
}

#[actix_web::test]
async fn auth_password_success() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    let request = app
        .post("api/auth/sign_in_with_password")
        .json(&serde_json::json!({
            "username": app.test_user.username,
            "password": app.test_user.password,
        }));
    let response = app.send_with_retries(request).await?;

    eyre::ensure!(
        response.status() == actix_web::http::StatusCode::OK,
        "unexpected response: {:?}",
        response
    );

    Ok(())
}
