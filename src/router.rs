use async_std::sync::Mutex;
use async_std::task;
use futures::Future;

use std::collections::HashMap;
use std::marker::Send;
use std::pin::Pin;
use std::sync::Arc;

use crate::Method;
use crate::Request;
use crate::Response;

use log::info;
pub type Handler = Box<dyn Fn(Request, Response) -> Pin<Box<dyn Future<Output = ()> + Send>>>;
pub type Routes = HashMap<String, HashMap<Method, Handler>>;

// #[derive(Debug, Clone)]
pub struct Router {
    pub routes: Routes,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn register<F>(
        &mut self,
        pathname: &str,
        method: Method,
        handler: F,
    ) -> Result<(), &'static str>
    where
        F: Fn(Request, Response) -> Pin<Box<dyn Future<Output = ()> + Send>> + 'static,
    {
        let mut pathname = pathname.to_string();
        if !pathname.starts_with("/") {
            pathname = format!("/{}", pathname);
        }

        let path = self.routes.entry(pathname).or_insert(HashMap::new());

        if path.contains_key(&method) {
            Err("path already exists")
        } else {
            path.insert(method, Box::new(handler));
            Ok(())
        }
    }

    pub async fn handle(
        &self,
        request: Request,
        writer: futures::io::WriteHalf<async_std::net::TcpStream>,
    ) {
        let pathname = &request.pathname;
        let method = &request.method;

        if let Some(path) = self.routes.get(pathname) {
            if let Some(handler) = path.get(&method) {
                info!("called handler on {}", pathname);
                let response = Response::new(writer);
                task::spawn(handler(request, response));
                // handler(request, response).await;
            }
        }
    }
}

pub trait App {
    fn get_router(&mut self) -> &Arc<Mutex<Router>>;

    fn get<F>(&mut self, pathname: &str, handler: F) -> impl Future<Output = Result<(), &str>>
    where
        F: Fn(Request, Response) -> Pin<Box<dyn Future<Output = ()> + Send>> + 'static,
        Self: Send,
    {
        async {
            let mut router = self.get_router().lock().await;
            router.register(pathname, Method::Get, handler)
        }
    }

    fn post<F>(&mut self, pathname: &str, handler: F) -> impl Future<Output = Result<(), &str>>
    where
        F: Fn(Request, Response) -> Pin<Box<dyn Future<Output = ()> + Send>> + 'static,
        Self: Send,
    {
        async {
            let mut router = self.get_router().lock().await;
            router.register(pathname, Method::Post, handler)
        }
    }
}

unsafe impl Send for Router {}
unsafe impl Sync for Router {}
