//! Client for the [Moondream](https://moondream.ai/) vision API.
//!
//! Provides a simple wrapper around the Moondream HTTP endpoints. It is used to 
//! detect objects in images, generate captions and answer visual questions. Examples 
//! are available in the `examples` directory.

use derive_new::new;
use derive_setters::Setters;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

/// Errors returned by the [`MoonDream`] client when performing HTTP requests.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wrapper around [`reqwest::Error`].
    #[error("MoonDream Error: {0}")]
    PointError(#[from] reqwest::Error),
}

/// Client for interacting with the [Moondream API](https://moondream.ai/).
///
/// Use [`MoonDream::remote`] when you have an API key or [`MoonDream::local`]
/// for unauthenticated local deployments. The client exposes helper methods for
/// the `/point`, `/detect`, `/caption` and `/query` endpoints.
#[derive(Debug, new, Setters, Clone)]
#[setters(prefix = "with_", into, strip_option)]
pub struct MoonDream {
    #[setters(skip)]
    token: String,

    #[new(value = "String::from(\"https://api.moondream.ai/v1\")")]
    endpoint: String,

    #[new(default)]
    headers: Vec<(String, String)>,

    #[new(value = "Duration::from_secs(5)")]
    timeout: Duration,

    #[new(value = "reqwest::Client::new()")]
    client: reqwest::Client,
}

/// Response returned by the `/point` endpoint.
///
/// Contains the request identifier, a list of centre [`Point`]s for each
/// detected object and an optional count of how many were found.
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct PointsResponse {
    /// Unique request identifier returned by the API.
    pub request_id: Option<String>,
    /// List of centre coordinates for each detected object.
    pub points: Vec<Point>,
    /// Number of points returned by the API.
    pub count: Option<usize>,
}

/// Response returned by the `/detect` endpoint.
///
/// Includes the request id and the bounding boxes for all detected objects.
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct DetectResponse {
    /// Unique request identifier returned by the API.
    pub request_id: Option<String>,
    /// Bounding boxes for each detected object.
    pub objects: Vec<DetectionObject>,
}

/// Bounding box coordinates for a detected object.
///
/// Values are normalized to the image dimensions (0-1). To convert them to
/// pixels multiply by the width and height of the source image.
#[derive(Debug, Deserialize, PartialOrd, PartialEq, Clone)]
pub struct DetectionObject {
    /// Left boundary of the box (normalized 0-1).
    pub x_min: f64,
    /// Top boundary of the box (normalized 0-1).
    pub y_min: f64,
    /// Right boundary of the box (normalized 0-1).
    pub x_max: f64,
    /// Bottom boundary of the box (normalized 0-1).
    pub y_max: f64,
}

/// Centre point coordinates returned by the `/point` endpoint.
///
/// Values are normalized to the image dimensions (0-1). To convert them to
/// pixels multiply by the width and height of the source image.
#[derive(Debug, Deserialize, PartialOrd, PartialEq, Clone)]
pub struct Point {
    /// Normalized X coordinate.
    pub x: f64,
    /// Normalized Y coordinate.
    pub y: f64,
}

/// Response from the `/query` endpoint (Visual Question Answering).
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct QueryResponse {
    /// Unique request identifier returned by the API.
    pub request_id: Option<String>,
    /// Answer returned for the asked question.
    pub answer: String,
}

impl MoonDream {
    /// Create a [`MoonDream`] instance for a local service.
    ///
    /// Use this when the API does not require authentication and you want to
    /// specify the service endpoint directly.
    pub fn local(endpoint: impl Into<String>) -> Self {
        MoonDream::new(String::new()).with_endpoint(endpoint)
    }

    /// Create a [`MoonDream`] instance for the hosted service.
    ///
    /// Provide the authentication token returned by the remote provider.
    pub fn remote(token: impl Into<String>) -> Self {
        MoonDream::new(token.into())
    }

    pub async fn points(
        &self,
        image: impl Into<String>,
        object: impl Into<String>,
    ) -> Result<PointsResponse, Error> {
        let object = object.into();
        let image = image.into();

        let result = self
            .client
            .post(format!("{}/point", self.endpoint))
            .header("X-Moondream-Auth", &self.token)
            .timeout(self.timeout.clone())
            .json(&json!({
                "image_url": image,
                "object": object,
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(result.json().await?)
    }

    pub async fn detect(
        &self,
        image: impl Into<String>,
        object: impl Into<String>,
    ) -> Result<DetectResponse, Error> {
        let object = object.into();
        let image = image.into();

        let result = self
            .client
            .post(format!("{}/detect", self.endpoint))
            .header("X-Moondream-Auth", &self.token)
            .timeout(self.timeout.clone())
            .json(&json!({
                "image_url": image,
                "object": object,
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(result.json().await?)
    }

    pub async fn caption(
        &self,
        image: impl Into<String>,
        length: Option<CaptionLength>,
    ) -> Result<CaptionResponse, Error> {
        let image = image.into();
        let length = length.unwrap_or(CaptionLength::Normal);

        let result = self
            .client
            .post(format!("{}/caption", self.endpoint))
            .header("X-Moondream-Auth", &self.token)
            .timeout(self.timeout.clone())
            .json(&json!({
                "image_url": image,
                "length": length.as_str(),
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(result.json().await?)
    }

    pub async fn query(
        &self,
        image: impl Into<String>,
        question: impl Into<String>,
    ) -> Result<QueryResponse, Error> {
        let image = image.into();
        let question = question.into();

        let result = self
            .client
            .post(format!("{}/query", self.endpoint))
            .header("X-Moondream-Auth", &self.token)
            .timeout(self.timeout.clone())
            .json(&json!({
                "image_url": image,
                "question": question,
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(result.json().await?)
    }
}

/// Controls the length of the caption returned by [`MoonDream::caption`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionLength {
    /// A brief caption.
    Short,
    /// A normal length caption.
    Normal,
}

impl CaptionLength {
    fn as_str(&self) -> &'static str {
        match self {
            CaptionLength::Short => "short",
            CaptionLength::Normal => "normal",
        }
    }
}

/// Response from the `/caption` endpoint.
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct CaptionResponse {
    /// Unique request identifier returned by the API.
    pub request_id: Option<String>,
    /// The generated caption text.
    pub caption: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_points_response_deserialization() {
        let json = r#"{
            "request_id": "abc",
            "points": [{"x": 0.1, "y": 0.2}],
            "count": 1
        }"#;

        let resp: PointsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.request_id, Some("abc".to_string()));
        assert_eq!(resp.points, vec![Point { x: 0.1, y: 0.2 }]);
        assert_eq!(resp.count, Some(1));
    }

    #[tokio::test]
    async fn test_points_functional() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "request_id": "abc",
            "points": [{"x": 0.5, "y": 0.5}],
            "count": 1
        });

        Mock::given(method("POST"))
            .and(path("/point"))
            .and(header("x-moondream-auth", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let md = MoonDream::new("token".to_string()).with_endpoint(server.uri());

        let resp = md
            .points("data:image/png;base64,AAA", "object")
            .await
            .unwrap();

        assert_eq!(
            resp,
            PointsResponse {
                request_id: Some("abc".to_string()),
                points: vec![Point { x: 0.5, y: 0.5 }],
                count: Some(1),
            }
        );
    }

    #[tokio::test]
    async fn test_detect_response_deserialization() {
        let json = r#"{
            "request_id": "req1",
            "objects": [{"x_min": 0.1, "y_min": 0.2, "x_max": 0.3, "y_max": 0.4}]
        }"#;

        let resp: DetectResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.request_id, Some("req1".to_string()));
        assert_eq!(
            resp.objects,
            vec![DetectionObject {
                x_min: 0.1,
                y_min: 0.2,
                x_max: 0.3,
                y_max: 0.4
            }]
        );
    }

    #[tokio::test]
    async fn test_detect_functional() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "request_id": "req1",
            "objects": [{"x_min": 0.1, "y_min": 0.2, "x_max": 0.3, "y_max": 0.4}]
        });

        Mock::given(method("POST"))
            .and(path("/detect"))
            .and(header("x-moondream-auth", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let md = MoonDream::new("token".to_string()).with_endpoint(server.uri());

        let resp = md
            .detect("data:image/png;base64,AAA", "object")
            .await
            .unwrap();

        assert_eq!(
            resp,
            DetectResponse {
                request_id: Some("req1".to_string()),
                objects: vec![DetectionObject {
                    x_min: 0.1,
                    y_min: 0.2,
                    x_max: 0.3,
                    y_max: 0.4
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_caption_response_deserialization() {
        let json = r#"{
            "request_id": "req2",
            "caption": "a cat on a mat"
        }"#;

        let resp: CaptionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.request_id, Some("req2".to_string()));
        assert_eq!(resp.caption, "a cat on a mat".to_string());
    }

    #[tokio::test]
    async fn test_caption_functional() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "request_id": "req2",
            "caption": "a cat on a mat"
        });

        Mock::given(method("POST"))
            .and(path("/caption"))
            .and(header("x-moondream-auth", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let md = MoonDream::new("token".to_string()).with_endpoint(server.uri());

        let resp = md
            .caption("data:image/png;base64,AAA", Some(CaptionLength::Normal))
            .await
            .unwrap();

        assert_eq!(
            resp,
            CaptionResponse {
                request_id: Some("req2".to_string()),
                caption: "a cat on a mat".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_query_response_deserialization() {
        let json = r#"{
            "request_id": "req3",
            "answer": "It is a cat"
        }"#;

        let resp: QueryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.request_id, Some("req3".to_string()));
        assert_eq!(resp.answer, "It is a cat".to_string());
    }

    #[tokio::test]
    async fn test_query_functional() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "request_id": "req3",
            "answer": "It is a cat"
        });

        Mock::given(method("POST"))
            .and(path("/query"))
            .and(header("x-moondream-auth", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let md = MoonDream::new("token".to_string()).with_endpoint(server.uri());

        let resp = md
            .query("data:image/png;base64,AAA", "What is this?")
            .await
            .unwrap();

        assert_eq!(
            resp,
            QueryResponse {
                request_id: Some("req3".to_string()),
                answer: "It is a cat".to_string(),
            }
        );
    }

    #[test]
    fn test_caption_length_as_str() {
        assert_eq!(CaptionLength::Short.as_str(), "short");
        assert_eq!(CaptionLength::Normal.as_str(), "normal");
    }

    #[test]
    fn test_constructors_and_setters() {
        let md_local = MoonDream::local("http://localhost:8080");
        assert_eq!(md_local.token, "");
        assert_eq!(md_local.endpoint, "http://localhost:8080".to_string());

        let md_remote = MoonDream::remote("secret");
        assert_eq!(md_remote.token, "secret".to_string());
        assert_eq!(
            md_remote.endpoint,
            "https://api.moondream.ai/v1".to_string()
        );

        let md_timeout = md_remote.clone().with_timeout(Duration::from_secs(10));
        assert_eq!(md_timeout.timeout, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_query_remote_functional() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "request_id": "req4",
            "answer": "Remote answer",
        });

        Mock::given(method("POST"))
            .and(path("/query"))
            .and(header("x-moondream-auth", "token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let md = MoonDream::remote("token").with_endpoint(server.uri());

        let resp = md
            .query("data:image/png;base64,AAA", "What is this?")
            .await
            .unwrap();

        assert_eq!(
            resp,
            QueryResponse {
                request_id: Some("req4".to_string()),
                answer: "Remote answer".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_points_local_functional() {
        let server = MockServer::start().await;

        let body = serde_json::json!({
            "request_id": "abc",
            "points": [{"x": 0.5, "y": 0.5}],
            "count": 1
        });

        Mock::given(method("POST"))
            .and(path("/point"))
            .and(header("x-moondream-auth", ""))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;

        let md = MoonDream::local(server.uri());

        let resp = md
            .points("data:image/png;base64,AAA", "object")
            .await
            .unwrap();

        assert_eq!(
            resp,
            PointsResponse {
                request_id: Some("abc".to_string()),
                points: vec![Point { x: 0.5, y: 0.5 }],
                count: Some(1),
            }
        );
    }
}
