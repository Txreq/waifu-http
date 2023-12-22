use std::{thread, time::Duration};
// use async_std::task::sleep;
use waifu_http::*;

#[async_std::main]
async fn main() {
    let mut app = Server::bind(6969).await.expect("failed to bind listener");
    // standard
    app.get("/", |req, mut res| {
        Box::pin(async move {
            println!("{:#?}", req);
            res.send(Some("hello")).await;
        })
    })
    .await
    .unwrap();

    // sleep
    app.get("/sleep", |_req, mut res| {
        Box::pin(async move {
            thread::sleep(Duration::from_secs(10));
            res.send(Some("slept for 5 seconds")).await;
        })
    })
    .await
    .unwrap();

    app.listen().await.unwrap();
}
