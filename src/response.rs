use std::collections::HashMap;

use async_std::net::TcpStream;
use futures::{io::WriteHalf, AsyncWriteExt};

use chrono::Utc;

type Headers = HashMap<&'static str, &'static str>;

#[derive(Debug)]
pub struct Response {
    pub status: HttpCode,
    pub content: &'static str,
    pub headers: Headers,
    writer: WriteHalf<TcpStream>,
}

impl Response {
    pub fn new(writer: WriteHalf<TcpStream>) -> Self {
        Self {
            status: HttpCode::OK,
            content: "",
            headers: HashMap::new(),
            writer,
        }
    }

    pub fn status_line(&self) -> String {
        let (status_update, status_code) = self.status.get_raw();
        let status_line = format!("HTTP/1.1 {} {}", status_code, status_update).to_owned();

        status_line
    }

    pub fn parse_raw(&self) -> String {
        let status_line = self.status_line();
        let content = self.content;
        let content_len = content.len().to_string();

        let time = Utc::now();
        let date = &time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let mut headers = self.headers.clone();
        let mut message = String::new();

        message.push_str(&status_line);
        message.push_str("\r\n");

        headers.insert("Connection", "keep-alive");
        headers.insert("Content-Length", &content_len);
        headers.insert("Date", date);

        for (key, value) in headers {
            let raw_header = format!("{}: {}\r\n", key, value);
            message.push_str(&raw_header);
        }

        message.push_str("\r\n");
        message.push_str(content);

        message
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let bytes = self.parse_raw().as_bytes().to_owned();
        bytes
    }

    pub async fn send(&mut self, content: Option<&'static str>) {
        self.content = match content {
            Some(content) => content,
            None => self.content,
        };
        self.headers.insert("Content-Type", "text/plain");

        let raw_response = self.parse_raw();
        self.writer.write(raw_response.as_bytes()).await.unwrap();
        self.writer.flush().await.unwrap();
    }

    pub fn set_status(&mut self, http_code: HttpCode) {
        self.status = http_code;
    }

    pub fn set_content(&mut self, content: &'static str) {
        self.content = content;
    }

    pub fn render(&mut self) {}
}

#[derive(Debug, Clone)]
pub enum HttpCode {
    OK = 200,
    BadRequest = 400,
    NotFound = 404,
    ServerError = 500,
}

impl HttpCode {
    fn get_raw(&self) -> (&'static str, u16) {
        match self {
            HttpCode::OK => ("OK", HttpCode::OK as u16),
            HttpCode::BadRequest => ("Bad Request", HttpCode::BadRequest as u16),
            HttpCode::NotFound => ("Not Found", HttpCode::NotFound as u16),
            HttpCode::ServerError => ("Server Error", HttpCode::NotFound as u16),
        }
    }
}
