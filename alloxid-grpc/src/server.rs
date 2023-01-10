use tonic::{transport::Server, Request, Response, Status};

use hello::greeter_server::{Greeter, GreeterServer};
use hello::{HelloReply, HelloRequest};

pub mod hello {
    tonic::include_proto!("hello");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Request received: {:?}", request);

        let reply = hello::HelloReply {
            // We must use .into_inner() as the fields of gRPC requests and responses are private
            message: format!("Hello, {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    println!("\nGreeterServer listening on {}", addr);
    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
