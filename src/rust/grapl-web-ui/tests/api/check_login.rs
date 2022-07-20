#![cfg(feature = "integration_tests")]

#[actix_web::test]
async fn unauthenticated_check_login() -> Result<(), Box<dyn std::error::Error>> {
    let app = crate::test_app::TestApp::spawn()?;

    let res = app.post("auth/checkLogin").send().await?;

    assert_eq!(res.status(), actix_web::http::StatusCode::UNAUTHORIZED);

    Ok(())
}
