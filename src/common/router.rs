use axum::{Router, extract::Host};
use axum_connect::prelude::*;

use proto::hello::{HelloWorldService, HelloRequest, HelloResponse};

mod proto {
    // Include the generated code in a `proto` module.
    //
    // Note: I'm not super happy with this pattern. I hope to add support to `protoc-gen-prost` in
    // the near-ish future instead see:
    // https://github.com/neoeinstein/protoc-gen-prost/issues/82#issuecomment-1877107220 That will
    // better align with Buf.build's philosophy. This is how it works for now though.
    pub mod hello {
        include!(concat!(env!("OUT_DIR"), "/hello.rs"));
    }
}

pub async fn new() -> Router {
    Router::new() 
        .rpc(HelloWorldService::say_hello(say_hello_unary))
}

/// The bread-and-butter of Connect-Web, a Unary request handler.
///
/// Just to demo error handling, I've chose to return a `Result` here. If your method is
/// infallible, you could just as easily return a `HellResponse` directly. The error type I'm using
/// is defined in `error.rs` and is worth taking a quick peak at.
///
/// Like Axum, both the request AND response types just need to implement RpcFromRequestParts` and
/// `RpcIntoResponse` respectively. This allows for a ton of flexibility in what your handlers
/// actually accept/return. This is a concept very core to Axum, so I won't go too deep into the
/// ideology here.
async fn say_hello_unary(Host(host): Host, request: HelloRequest) -> Result<HelloResponse, Error> {
    Ok(HelloResponse {
        message: format!(
            "Hello {}! You're addressing the hostname: {}.",
            request.name.clone(),
            host.clone()
        ),
    })
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_connect::error::{RpcError, RpcErrorCode, RpcIntoError};

// This is an example Error type, to demo impls needed for `axum-connect`. It uses `thiserror` to
// wrap various error types as syntactic sugar, but you could just as easily write this out by hand.
#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Returns `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Returns `404 Not Found`
    #[error("request path not found")]
    NotFound,

    /// Returns `500 Internal Server Error`
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

/// Allows the error type to be returned from RPC handlers.
///
/// This trait is distinct from `IntoResponse` because RPCs cannot return arbitrary HTML responses.
/// Error codes are well-defined in connect-web (which mirrors gRPC), streaming errors don't effect
/// HTTP status codes, and so on.
impl RpcIntoError for Error {
    fn rpc_into_error(self) -> axum_connect::prelude::RpcError {
        println!("{:#?}", self);

        // Each response is a tuple of well-defined (per the Connect-Web) codes, along with a
        // message.
        match self {
            Self::Forbidden => {
                RpcError::new(RpcErrorCode::PermissionDenied, "Forbidden".to_string())
            }
            Self::NotFound => RpcError::new(RpcErrorCode::NotFound, "Not Found".to_string()),
            Self::Anyhow(_) => {
                RpcError::new(RpcErrorCode::Internal, "Internal Server Error".to_string())
            }
        }
    }
}

// This is a normal `IntoResponse` impl, which is used by Axum to convert errors into HTTP
// responses.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("{:#?}", self);
        match self {
            Self::Forbidden => (StatusCode::FORBIDDEN, "Forbidden").into_response(),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            Self::Anyhow(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}