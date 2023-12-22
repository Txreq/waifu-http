use async_std::net::TcpStream;
use futures::{io::WriteHalf, AsyncWriteExt};
use log::error;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::Utc;

type Headers = HashMap<&'static str, String>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Response {
    pub status: HttpCode,
    pub content: String,
    pub headers: Headers,

    views: PathBuf,
    writer: WriteHalf<TcpStream>,
}

impl Response {
    pub fn new(writer: WriteHalf<TcpStream>, views: PathBuf) -> Self {
        Self {
            status: HttpCode::OK,
            content: String::new(),
            headers: HashMap::new(),
            writer,
            views,
        }
    }

    pub fn status_line(&self) -> String {
        let (status_update, status_code) = self.status.get_raw();
        let status_line = format!("HTTP/1.1 {} {}", status_code, status_update).to_owned();

        status_line
    }

    pub fn parse_raw(&self) -> String {
        let status_line = self.status_line();
        let content = &self.content;
        let content_len = content.len();

        let time = Utc::now();
        let date = &time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let mut headers = self.headers.clone();
        let mut message = String::new();

        message.push_str(&status_line);
        message.push_str("\r\n");

        headers.insert("Connection", "keep-alive".to_string());
        headers.insert("Content-Length", content_len.to_string());
        headers.insert("Date", date.to_string());

        if let None = headers.get("Content-Type") {
            headers.insert("Content-Type", MimeType::TextPlain.to_string());
        }

        for (key, value) in headers {
            let raw_header = format!("{}: {}\r\n", key, value);
            message.push_str(&raw_header);
        }

        message.push_str("\r\n");
        message.push_str(&content);

        message
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let bytes = self.parse_raw().as_bytes().to_owned();
        bytes
    }

    pub async fn send(&mut self, content: &str) {
        self.content = content.to_string();

        let raw_response = self.parse_raw();
        self.writer.write(raw_response.as_bytes()).await.unwrap();
        self.writer.flush().await.unwrap();
    }

    pub fn status(&mut self, http_code: HttpCode) {
        self.status = http_code;
    }

    pub async fn render(&mut self, file: &'static str) {
        let path = Path::join(&self.views, file);

        match File::open(path) {
            Ok(mut fd) => {
                let mut content = String::new();
                if let Ok(size) = fd.read_to_string(&mut content) {
                    self.headers
                        .insert("Content-Type", MimeType::TextHtml.to_string());
                    self.headers.insert("Content-Length", size.to_string());
                    self.send(&content).await;
                } else {
                    let filename = format!("{} NOT FOUND", file);
                    let filename = filename.as_str();
                    self.status(HttpCode::NotFound);
                    self.send(filename).await;
                };
            }
            Err(err) => {
                error!("{}", err.to_string());
                self.status(HttpCode::ServerError);
                self.send("Internal server exception").await;
            }
        }
    }
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

#[derive(Debug)]
#[allow(dead_code)]
enum MimeType {
    TextPlain,
    TextHtml,
    ApplicationJson,
    ImageJpeg,
    ImagePng,
}

impl MimeType {
    fn to_string(&self) -> String {
        match self {
            MimeType::TextPlain => String::from("text/plain"),
            MimeType::TextHtml => String::from("text/html"),
            MimeType::ApplicationJson => String::from("application/json"),
            MimeType::ImageJpeg => String::from("image/jpeg"),
            MimeType::ImagePng => String::from("image/png"),
        }
    }
}
