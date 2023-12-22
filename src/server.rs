use std::sync::Arc;

use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::sync::Mutex;

use async_std::task;
use futures::io::BufReader;
use futures::stream::StreamExt;
use futures::AsyncReadExt;

use log::error;
use log::info;
use log::warn;

use crate::App;
use crate::HttpCode;
use crate::Request;
use crate::Response;
use crate::Router;

pub struct Server {
    pub port: u16,
    pub router: Arc<Mutex<Router>>,
    listener: TcpListener,
}

impl Server {
    pub async fn bind(port: u16) -> Result<Self, String> {
        env_logger::init();
        let router = Arc::new(Mutex::new(Router::new()));

        match TcpListener::bind(format!("127.0.0.1:{}", &port)).await {
            Ok(listener) => Ok(Self {
                port,
                listener,
                router,
            }),
            Err(_) => Err(String::from(format!(
                "failed to bind listener to {}",
                &port
            ))),
        }
    }

    pub async fn listen(&mut self) -> Result<(), String> {
        info!("local: http://127.0.0.1:{}", self.port);

        let mut incoming = self.listener.incoming();
        while let Some(stream) = incoming.next().await {
            let router = self.router.clone();
            task::spawn(handle_stream(stream, router));
        }

        Ok(())
    }
}

impl App for Server {
    fn get_router(&mut self) -> &Arc<Mutex<Router>> {
        &self.router
    }
}

// handling stream
async fn handle_stream<T>(stream: Result<TcpStream, T>, router: Arc<Mutex<Router>>) {
    match stream {
        Ok(stream) => {
            let (reader, writer) = stream.split();
            let mut reader = BufReader::new(reader);
            match Request::parse_raw(&mut reader).await {
                Ok(request) => {
                    let writer = writer;
                    let router = router.lock().await;
                    router.handle(request, writer).await;
                }
                Err(err) => {
                    warn!("{}", err);
                    let mut response = Response::new(writer);
                    response.set_status(HttpCode::BadRequest);
                    response.set_content("Invalid request");
                    response.send(None).await;
                }
            };
        }
        Err(_) => {
            error!("");
        }
    }
}
