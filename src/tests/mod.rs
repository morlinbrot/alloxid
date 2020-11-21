//use crate::make_server;
//use assert_json_diff::assert_json_include;
//use http_types::{Method, Request, Url};
//use serde_json::{json, Value};

//use crate::settings::Settings;

//mod test_helpers;
//use test_helpers::*;

#[async_std::test]
async fn it_works() {
    //let config = Settings::new().unwrap();
    //let server = make_server().await;
    //let mut server = make_test_server(server).unwrap();

    //let req = Request::new(Method::Get, Url::parse("http://example.com/").unwrap());
    //let res = server.simulate(req).unwrap();
    //assert_eq!(res.status(), 200);

    //let body = res.body_string().await.unwrap();
    //let json: Value = serde_json::from_str(&body).unwrap();
    //assert_json_include!(actual: json, expected: json!([1, 2, 3]));
}
