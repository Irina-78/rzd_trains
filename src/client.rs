use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER, USER_AGENT};
use reqwest::{blocking::Response, cookie::Cookie};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::thread;
use std::time::Duration;
use std::{fmt, fmt::Display};

use crate::error::Error;
use crate::Result;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

const RZD_REFERER: &str = "rzd.ru";

// Some requests get a response with data at once
// and some get a response with identifier of the answer.
#[derive(PartialEq)]
#[non_exhaustive]
pub enum RzdQueryType {
    Simple,
    WithId,
}

// Identifier returned by the server in response to some requests.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RzdRequestId(u64);

impl RzdRequestId {
    pub fn new(id: u64) -> Self {
        RzdRequestId(id)
    }

    // Performs the conversion into digits.
    #[inline]
    pub fn to_uint(&self) -> u64 {
        self.0
    }
}

impl Default for RzdRequestId {
    fn default() -> Self {
        RzdRequestId(0)
    }
}

impl Display for RzdRequestId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait RzdClientInterface<T> {
    fn query_type(&self) -> RzdQueryType;

    fn request_id(&self) -> String;
    fn request_data(&self, id: RzdRequestId) -> String;

    fn deserialize_reply_id(&self, response: Response) -> Result<Option<RzdRequestId>>;
    fn deserialize_reply_data(&self, response: Response) -> Result<Option<T>>;
}

/// The client gets data from the server.
pub struct RzdClient<T> {
    _marker: PhantomData<T>,
}

impl<T> RzdClient<T> {
    /// Takes a search query and makes a request to the server.
    ///
    /// # Errors
    ///
    /// The method fails if there was an error while processing request
    /// or received data couldn't be deserialized.
    pub fn get<U>(search: &U) -> Result<Option<T>>
    where
        U: RzdClientInterface<T>,
    {
        match search.query_type() {
            RzdQueryType::Simple => RzdClient::simple_request(search),
            RzdQueryType::WithId => RzdClient::request_with_id(search),
        }
    }

    // Getting data with a single request to the server.
    fn simple_request<U>(search: &U) -> Result<Option<T>>
    where
        U: RzdClientInterface<T>,
    {
        let request = search.request_data(RzdRequestId::default());
        debug!("request: {}", request);

        let result = send_blocking_request(&request, request_headers_default())?;

        let result = match result {
            None => return Ok(None),
            Some(r) => r,
        };

        match search.deserialize_reply_data(result)? {
            Some(r) => Ok(Some(r)),
            None => Ok(None),
        }
    }

    // Getting data with a additional request to the server.
    fn request_with_id<U>(search: &U) -> Result<Option<T>>
    where
        U: RzdClientInterface<T>,
    {
        let (reply_id, cookies) = RzdClient::get_reply_id(search)?;

        let mut headers = request_headers_default();
        let cookie_header = HeaderValue::from_bytes(cookies.as_bytes())?;
        headers.insert(COOKIE, cookie_header);

        let request = search.request_data(reply_id);
        debug!("request: {}", request);

        for _ in 1..4 {
            // If server wasn't be on time to create an answer
            // then it sends a new `RzdRequestId`.
            thread::sleep(Duration::from_millis(1500));

            let result = send_blocking_request(request.clone().as_ref(), headers.clone())?;

            let result = match result {
                None => return Ok(None),
                Some(r) => r,
            };

            match search.deserialize_reply_data(result)? {
                Some(r) => return Ok(Some(r)),
                None => debug!("reply is incorrect"),
            }
        }

        Err(Error::RzdServerOverloaded)
    }

    fn get_reply_id<U>(search: &U) -> Result<(RzdRequestId, String)>
    where
        U: RzdClientInterface<T>,
    {
        let request = search.request_id();
        debug!("request: {}", request);

        let result = send_blocking_request(&request, request_headers_default())?;

        let result = match result {
            None => return Err(Error::FailRzdResponse),
            Some(r) => r,
        };

        let cookies = get_cookies_string(&mut result.cookies());

        let reply_id = match search.deserialize_reply_id(result)? {
            None => return Err(Error::FailRzdResponse),
            Some(r) => r,
        };

        Ok((reply_id, cookies))
    }
}

fn send_blocking_request(query: &str, headers: HeaderMap) -> Result<Option<Response>> {
    let request = reqwest::blocking::Client::new().get(query).headers(headers);

    let result = request.send()?;

    if !result.status().is_success() {
        error!("server returned {}", result.status());
        return Err(Error::FailRzdResponse);
    }

    if let Some(0) = result.content_length() {
        warn!("response body is empty");
        return Ok(None);
    }

    Ok(Some(result))
}

fn request_headers_default() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(APP_USER_AGENT));
    headers.insert(REFERER, HeaderValue::from_static(RZD_REFERER));
    headers
}

fn get_cookies_string(cookies_iter: &mut dyn Iterator<Item = Cookie>) -> String {
    let mut cookies: Vec<String> = cookies_iter
        .map(|c| format!("{}={}", c.name(), c.value()))
        .collect();
    cookies.sort();
    cookies.dedup();
    cookies.join("; ")
}
