use axum::body::Body;
use axum::response::{IntoResponse, Response};
use axum_macros::debug_handler;
use tracing::debug;

use crate::error::ServiceError;
use crate::StateExtension;

use alloxid_grpc::hello::greeter_client::GreeterClient;
use alloxid_grpc::hello::HelloRequest;

#[debug_handler]
pub(crate) async fn hello(state: StateExtension) -> Result<impl IntoResponse, ServiceError> {
    let _pool = state.db_pool.clone();
    let settings = state.settings.clone();

    debug!(
        "grpc/hello called, port={} db_name={}",
        settings.app.port, settings.database.name,
    );

    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".to_string(),
    });

    let response = client
        .say_hello(request)
        .await
        .expect("Failed to parse gRPC response");

    Ok(Response::new(Body::from(format!(
        "Message from the grpc server: {:?}",
        response.get_ref().message
    ))))
}
