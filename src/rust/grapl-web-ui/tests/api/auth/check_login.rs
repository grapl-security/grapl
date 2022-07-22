#[actix_web::test]
async fn auth_unauthenticated_check_login() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    let response = app.post("api/auth/checkLogin").send().await?;

    println!("{:?}", response);

    assert_eq!(response.status(), actix_web::http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[actix_web::test]
async fn auth_authenticated_check_login() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    app.login_with_test_user().await?;

    let response = app.post("api/auth/checkLogin").send().await?;

    println!("{:?}", response);

    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    Ok(())
}
