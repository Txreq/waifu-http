use std::collections::HashMap;

use futures::{AsyncBufRead, AsyncBufReadExt};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Method {
    Get,
    Post,
    Patch,
    Delete,
}

impl TryFrom<&str> for Method {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Get" => Ok(Self::Get),
            _ => Err("unsupported method"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub pathname: String,
    pub headers: HashMap<String, String>,
    pub size: usize,
}

impl Request {
    pub async fn parse_raw(mut stream: impl AsyncBufRead + Unpin) -> Result<Self, &'static str> {
        let mut size = 0 as usize;
        let mut line_buffer = String::new();
        let bytes = stream.read_line(&mut line_buffer).await.unwrap();

        if line_buffer.is_empty() {
            return Err("empty request");
        }

        size += bytes;

        let mut headers: HashMap<String, String> = HashMap::new();
        let req_line = line_buffer
            .split_whitespace()
            .into_iter()
            .collect::<Vec<&str>>();

        let method = match Method::try_from(req_line[0]) {
            Ok(method) => method,
            Err(_) => Method::Get,
        };
        let pathname = req_line[1].to_string();

        loop {
            line_buffer.clear();
            let bytes = stream.read_line(&mut line_buffer).await.unwrap();

            if line_buffer.is_empty() || line_buffer == "\n" || line_buffer == "\r\n" {
                break;
            }

            size += bytes;
            let mut parts = line_buffer.splitn(2, ": ");
            if let (Some(header_name), Some(header_value)) = (parts.next(), parts.next()) {
                headers.insert(
                    header_name.trim().to_string(),
                    header_value.trim().to_string(),
                );
            }
        }

        Ok(Self {
            method,
            pathname,
            headers,
            size,
        })
    }
}
