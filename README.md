# waifu-http

To be honest, I'm not sure what is this nor what is inside. Does it suck? Yes. Why? I don't know, figure that yourself. I did it to learn more about this hard a\*\* language: Rust; if you think my code sucks which I believe it is and you know how to fix it, please do and show me how did you do it.

## Why?

The answer is still: I DO NOT KNOW.

## Usage:

Obviously:

```rs
#[async_std::main]
async fn main() {
    let mut app = Server::bind(6969).await.expect("failed to bind listener");

    app.get("/", |req, mut res| {
        Box::pin(async move {
            res.send(Some("Hello, world!")).await;
        })
    })
    .await
    .unwrap();

    app.listen().await.unwrap();
}
```
