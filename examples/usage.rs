// use std::{thread, time::Duration};
// use async_std::task::sleep;
use waifu_http::*;

#[async_std::main]
async fn main() {
    let mut app = Server::bind(6969).await.expect("failed to bind listener");
    // standard
    app.get("/", |req, res| Box::pin(hello(req, res)))
        .await
        .unwrap();

    app.listen().await.unwrap();
}

async fn hello(req: Request, mut res: Response) {
    res.render("index.html").await;
}
