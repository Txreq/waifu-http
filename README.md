# waifu-http

To be honest, I'm not sure what is this nor what is inside. Does it suck? Yes. Why? I don't know, figure that yourself. I did it to learn more about this hard a\*\* language: Rust; if you think my code sucks which I believe it is and you know how to fix it, please do and show me how did you do it.

## Why?

The answer is still: I DO NOT KNOW.

## Usage:

Obviously:

```rs
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
```
