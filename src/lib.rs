use worker::*;

mod params;
mod resize;
mod cache;

#[event(fetch)]
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;

    let params = match params::ResizeParams::from_url(&url) {
        Ok(p) => p,
        Err(e) => return Response::error(e, 400),
    };

    // Check cache first
    let cache_key = cache::build_key(&params);
    if let Some(cached) = cache::get(&cache_key).await {
        return Ok(cached);
    }

    // Fetch source image
    let src_bytes = fetch_source(&params.src).await?;

    // Resize
    let output = resize::process(&src_bytes, &params)
        .map_err(|e| Error::RustError(e.to_string()))?;

    let mut response = Response::from_bytes(output)?;
    response.headers_mut().set("Content-Type", "image/jpeg")?;
    response.headers_mut().set("Cache-Control", "public, max-age=31536000, immutable")?;

    // Store in cache
    cache::put(&cache_key, &mut response).await;

    Ok(response)
}

async fn fetch_source(src_url: &str) -> Result<Vec<u8>> {
    let mut res = Fetch::Url(src_url.parse().map_err(|_| Error::RustError("invalid src url".into()))?)
        .send()
        .await?;

    if res.status_code() != 200 {
        return Err(Error::RustError(format!("upstream returned {}", res.status_code())));
    }

    let content_type = res.headers().get("Content-Type")?.unwrap_or_default();
    if !content_type.starts_with("image/") {
        return Err(Error::RustError("src is not an image".into()));
    }

    res.bytes().await
}
