
// pub fn get_path(_path: String) -> String {
//     // if path == "/modelPluginDeployer"{
//     //     deploy();
//     // }
//     //
//     // if path == "/deleteModelPlugin"{
//     //     delete();
//     // }
//     //
//     // if path == "list" {
//     //     list();
//     // }
//     return "test".to_string();
//
// }


use::reqwest;
// use actix_web::body::Body;
use actix_web::HttpResponse;


pub async fn make_request(path: &str, body: HttpResponse) ->  Result<(), Box<dyn std::error::Error>>{
    let client = reqwest::Client::new();
    let response = client.post(format!("http://localhost:8000/{}", path))
        .body(body)
        .send()
        .await?;

    Ok(())
}

pub mod model_plugin_deployer_router;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}