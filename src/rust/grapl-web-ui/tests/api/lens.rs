#[actix_web::test]
async fn lens_subscribe() -> eyre::Result<()> {
    let app = crate::test_app::TestApp::init().await?;

    app.login_with_test_user().await?;

    let response = app.post("api/lens/subscribe").send().await?;

    println!("{:?}", response);

    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    Ok(())
}
