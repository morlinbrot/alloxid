use tokio::time::{sleep, Duration};
use tonic::{transport::Server, Request, Response, Status};

use alloxid_grpc::hello::{
    self,
    greeter_client::GreeterClient,
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> {
        // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = hello::HelloReply {
            message: format!("Hello, {}!", request.into_inner().name).into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

pub async fn spawn_test_server() -> std::net::SocketAddr {
    let greeter = MyGreeter::default();

    let addr = "[::1]:50051".parse().expect("Failed to parse addr");
    // println!("\ngRPC server listening on {}", addr);

    tokio::spawn(async move {
        Server::builder()
            .add_service(GreeterServer::new(greeter))
            .serve(addr)
            .await
            .unwrap();
    });

    sleep(Duration::from_millis(100)).await;

    addr
}

#[tokio::test]
async fn hello() {
    let addr = spawn_test_server().await;

    let mut client = GreeterClient::connect(format!("http://{}", addr))
        .await
        .expect("Failed to connect client");

    let request = tonic::Request::new(HelloRequest {
        name: "Test".to_string(),
    });

    let res = client
        .say_hello(request)
        .await
        .expect("Failed to parse gRPC response");

    println!("{:?}", res.metadata());
    assert_eq!(res.get_ref().message, "Hello, Test!");
}
