use url::Url;

#[derive(Debug, Clone)]
pub struct ResizeParams {
    pub src: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: u8,
}

impl ResizeParams {
    pub fn from_url(url: &Url) -> std::result::Result<Self, &'static str> {
        let query: std::collections::HashMap<_, _> = url.query_pairs().collect();

        let src = query.get("src")
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .ok_or("missing required param: src")?;

        // Basic URL validation — must be http/https
        if !src.starts_with("http://") && !src.starts_with("https://") {
            return Err("src must be an http/https URL");
        }

        let width = query.get("w")
            .and_then(|v| v.parse::<u32>().ok())
            .map(|v| v.clamp(1, 8000));

        let height = query.get("h")
            .and_then(|v| v.parse::<u32>().ok())
            .map(|v| v.clamp(1, 8000));

        if width.is_none() && height.is_none() {
            return Err("at least one of ?w or ?h is required");
        }

        let quality = query.get("q")
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(85)
            .clamp(1, 100);

        Ok(Self { src, width, height, quality })
    }

    pub fn cache_key_suffix(&self) -> String {
        format!(
            "w{}_h{}_q{}",
            self.width.map_or("x".into(), |v| v.to_string()),
            self.height.map_or("x".into(), |v| v.to_string()),
            self.quality,
        )
    }
}
