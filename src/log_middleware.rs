use futures::Future;
use std::pin::Pin;
use tide::{Next, Request};

#[derive(Debug)]
pub(crate) struct LogMiddleware;

use super::*;

type BoxedTideResult<'a> = Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>>;

pub fn logger<'a>(req: Request<State>, next: Next<'a, State>) -> BoxedTideResult {
    println!(
        "log: Incoming {} request on url {}",
        req.method(),
        req.url()
    );
    Box::pin(async {
        let res = next.run(req).await;
        println!("log: Outgoing response with status {}", res.status());
        Ok(res)
    })
}
