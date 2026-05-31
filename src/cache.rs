use worker::{Cache, Response};
use crate::params::ResizeParams;

pub fn build_key(params: &ResizeParams) -> String {
    format!(
        "https://rezr.cache/{}/{}",
        urlencoding::encode(&params.src),
        params.cache_key_suffix(),
    )
}

pub async fn get(key: &str) -> Option<Response> {
    let cache = Cache::default();
    cache.get(key, false).await.ok().flatten()
}

pub async fn put(key: &str, response: &mut Response) {
    let cache = Cache::default();
    if let Ok(cloned) = response.cloned() {
        let _ = cache.put(key, cloned).await;
    }
}
