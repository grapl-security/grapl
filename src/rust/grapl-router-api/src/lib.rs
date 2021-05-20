// use grapl_router_api::model_plugin_deployer_router::delete::grapl_model_plugin_delete;

//
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

// use actix_web::HttpRequest;
// use std::thread::Builder;
// use actix_web::http::{Method, Uri};
use http::{Request, Response};


pub async fn make_request(path: &str) -> Response<()>{
    let request = Request::post(format!("http://localhost:8000/{}", path)) // replace with Colins Grpc
        .body(())
        .unwrap();

    // return request;

    match request.await {
        Ok(res) => res,
        Err(err) => err,
    }
}


pub mod model_plugin_deployer_router;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}