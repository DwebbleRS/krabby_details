//! Types to represent a problem detail error response.
//!
//! See [RFC 9457](https://www.rfc-editor.org/rfc/rfc9457.html) for more details.
use std::borrow::Cow;

use bytes::{BufMut, BytesMut};
use http::{header::CONTENT_TYPE, HeaderName, HeaderValue, StatusCode};

#[derive(serde::Serialize, Debug)]
pub struct ProblemDetails<Extension> {
    #[serde(rename = "type")]
    pub type_: Cow<'static, str>,
    pub status: u16,
    pub title: Cow<'static, str>,
    pub detail: Cow<'static, str>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extension>,
}

#[derive(serde::Serialize)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

#[derive(serde::Serialize)]
pub struct ValidationError {
    pub detail: String,
    #[serde(flatten)]
    pub source: Source,
}

/// The request part where the problem occurred.
#[derive(serde::Serialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum Source {
    Body {
        /// A [JSON pointer](https://www.rfc-editor.org/info/rfc6901) targeted
        /// at the problematic body property.
        pointer: Option<String>,
    },
    Header {
        /// The name of the problematic header.
        name: Cow<'static, str>,
    },
}

impl<Extension> axum_core::response::IntoResponse for ProblemDetails<Extension>
where
    Extension: serde::Serialize,
{
    fn into_response(self) -> axum_core::response::Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_json::to_writer(&mut buf, &self) {
            Ok(()) => (
                [(CONTENT_TYPE, APPLICATION_PROBLEM_JSON)],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(_) => INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

pub const APPLICATION_PROBLEM_JSON: HeaderValue =
    HeaderValue::from_static("application/problem+json");

pub const INTERNAL_SERVER_ERROR: (StatusCode, [(HeaderName, HeaderValue); 1], &[u8]) = (
    StatusCode::INTERNAL_SERVER_ERROR,
    [(CONTENT_TYPE, APPLICATION_PROBLEM_JSON)],
    INTERNAL_SERVER_ERROR_PROBLEM,
);

pub const INTERNAL_SERVER_ERROR_PROBLEM: &[u8] = br#"{
    "type": "internal_server_error",
    "title": "Internal Server Error",
    "detail": "Something went wrong when processing your request. Please try again later."
    "status": 500
}"#;
