use std::marker::PhantomData;

use poem::{
    error::Error,
    http::{header, StatusCode},
    Endpoint, Request, Response,
};
use rust_embed::RustEmbed;

/// An endpoint that wraps a `rust-embed` bundle and serves files from it.
/// It serves 'index.html' if the path is empty or requested file is not found.
/// This is useful for serving a single-page application (SPA) from a binary.
pub struct EmbeddedSPAEndpoint<E: RustEmbed + Send + Sync> {
    _embed: PhantomData<E>,
}

impl<E: RustEmbed + Sync + Send> Default for EmbeddedSPAEndpoint<E> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<E: RustEmbed + Send + Sync> EmbeddedSPAEndpoint<E> {
    /// Create a new `EmbeddedFilesEndpoint` from a `rust-embed` bundle.
    pub fn new() -> Self {
        EmbeddedSPAEndpoint {
            _embed: PhantomData,
        }
    }

    async fn get_content(&self, path: &str, req: &Request) -> Result<Option<Response>, Error> {
        match E::get(path) {
            Some(content) => {
                // If the file is found, check if the etag matches
                let hash = hex::encode(content.metadata.sha256_hash());
                if req
                    .headers()
                    .get(header::IF_NONE_MATCH)
                    .map(|etag| etag.to_str().unwrap_or("000000").eq(&hash))
                    .unwrap_or(false)
                {
                    // Returns 304 if the etag matches
                    return Err(StatusCode::NOT_MODIFIED.into());
                }

                // otherwise, return 200 with etag hash
                let body: Vec<u8> = content.data.into();
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Ok(Some(
                    Response::builder()
                        .header(header::CONTENT_TYPE, mime.as_ref())
                        .header(header::ETAG, hash)
                        .body(body),
                ))
            }
            None => Ok(None),
        }
    }
}

impl<E: RustEmbed + Send + Sync> Endpoint for EmbeddedSPAEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output, Error> {
        let mut path = req
            .uri()
            .path()
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string();
        // Default to index.html if path is empty
        if path.is_empty() {
            path = "index.html".to_string();
        }
        let path = path.as_ref();
        match self.get_content(path, &req).await? {
            Some(response) => Ok(response),
            None => {
                // If the path is not found, try to serve index.html
                let path = "index.html";
                match self.get_content(path, &req).await? {
                    Some(response) => Ok(response),
                    // If index.html is not found, return 404
                    None => Err(StatusCode::NOT_FOUND.into()),
                }
            }
        }
    }
}
