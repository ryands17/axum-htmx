use bytes::Bytes;
use http::StatusCode;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct TestClient {
  client: reqwest::Client,
  addr: SocketAddr,
}

impl TestClient {
  pub async fn new(svc: axum::Router) -> Self {
    let listener = TcpListener::bind("127.0.0.1:0")
      .await
      .expect("Could not bind ephemeral socket");
    let addr = listener.local_addr().unwrap();

    #[cfg(feature = "withtrace")]
    println!("Listening on {}", addr);

    tokio::spawn(async move {
      let server = axum::serve(listener, svc);
      server.await.expect("server error");
    });

    #[cfg(feature = "cookies")]
    let client = reqwest::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .cookie_store(true)
      .build()
      .unwrap();

    #[cfg(not(feature = "cookies"))]
    let client = reqwest::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .build()
      .unwrap();

    TestClient { client, addr }
  }

  /// returns the base URL (http://ip:port) for this TestClient
  ///
  /// this is useful when trying to check if Location headers in responses
  /// are generated correctly as Location contains an absolute URL
  pub fn base_url(&self) -> String {
    format!("http://{}", self.addr)
  }

  pub fn get(&self, url: &str) -> RequestBuilder {
    RequestBuilder {
      builder: self.client.get(format!("http://{}{}", self.addr, url)),
    }
  }

  pub fn head(&self, url: &str) -> RequestBuilder {
    RequestBuilder {
      builder: self.client.head(format!("http://{}{}", self.addr, url)),
    }
  }

  pub fn post(&self, url: &str) -> RequestBuilder {
    RequestBuilder {
      builder: self.client.post(format!("http://{}{}", self.addr, url)),
    }
  }

  pub fn put(&self, url: &str) -> RequestBuilder {
    RequestBuilder {
      builder: self.client.put(format!("http://{}{}", self.addr, url)),
    }
  }

  pub fn patch(&self, url: &str) -> RequestBuilder {
    RequestBuilder {
      builder: self.client.patch(format!("http://{}{}", self.addr, url)),
    }
  }

  pub fn delete(&self, url: &str) -> RequestBuilder {
    RequestBuilder {
      builder: self.client.delete(format!("http://{}{}", self.addr, url)),
    }
  }
}

pub struct RequestBuilder {
  builder: reqwest::RequestBuilder,
}

impl RequestBuilder {
  pub async fn send(self) -> TestResponse {
    TestResponse {
      response: self.builder.send().await.unwrap(),
    }
  }

  pub fn body(mut self, body: impl Into<reqwest::Body>) -> Self {
    self.builder = self.builder.body(body);
    self
  }

  pub fn form<T: serde::Serialize + ?Sized>(mut self, form: &T) -> Self {
    self.builder = self.builder.form(&form);
    self
  }

  pub fn json<T>(mut self, json: &T) -> Self
  where
    T: serde::Serialize,
  {
    self.builder = self.builder.json(json);
    self
  }

  pub fn header(mut self, key: &str, value: &str) -> Self {
    self.builder = self.builder.header(key, value);
    self
  }

  pub fn multipart(mut self, form: reqwest::multipart::Form) -> Self {
    self.builder = self.builder.multipart(form);
    self
  }
}

/// A wrapper around [`reqwest::Response`] that provides common methods with internal `unwrap()`s.
///
/// This is conventient for tests where panics are what you want. For access to
/// non-panicking versions or the complete `Response` API use `into_inner()` or
/// `as_ref()`.
pub struct TestResponse {
  response: reqwest::Response,
}

impl TestResponse {
  pub async fn text(self) -> String {
    self.response.text().await.unwrap()
  }

  #[allow(dead_code)]
  pub async fn bytes(self) -> Bytes {
    self.response.bytes().await.unwrap()
  }

  pub async fn json<T>(self) -> T
  where
    T: serde::de::DeserializeOwned,
  {
    self.response.json().await.unwrap()
  }

  pub fn status(&self) -> StatusCode {
    StatusCode::from_u16(self.response.status().as_u16()).unwrap()
  }

  pub fn headers(&self) -> &reqwest::header::HeaderMap {
    self.response.headers()
  }

  pub async fn chunk(&mut self) -> Option<Bytes> {
    self.response.chunk().await.unwrap()
  }

  pub async fn chunk_text(&mut self) -> Option<String> {
    let chunk = self.chunk().await?;
    Some(String::from_utf8(chunk.to_vec()).unwrap())
  }

  /// Get the inner [`reqwest::Response`] for less convenient but more complete access.
  pub fn into_inner(self) -> reqwest::Response {
    self.response
  }
}

impl AsRef<reqwest::Response> for TestResponse {
  fn as_ref(&self) -> &reqwest::Response {
    &self.response
  }
}
