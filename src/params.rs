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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(query: &str) -> Result<ResizeParams, &'static str> {
        let url = Url::parse(&format!("https://example.com/?{query}")).unwrap();
        ResizeParams::from_url(&url)
    }

    #[test]
    fn width_only_is_valid() {
        let p = parse("src=https://img.example.com/a.jpg&w=400").unwrap();
        assert_eq!(p.width, Some(400));
        assert_eq!(p.height, None);
    }

    #[test]
    fn height_only_is_valid() {
        let p = parse("src=https://img.example.com/a.jpg&h=300").unwrap();
        assert_eq!(p.width, None);
        assert_eq!(p.height, Some(300));
    }

    #[test]
    fn both_dims_valid() {
        let p = parse("src=https://img.example.com/a.jpg&w=800&h=600").unwrap();
        assert_eq!(p.width, Some(800));
        assert_eq!(p.height, Some(600));
    }

    #[test]
    fn missing_src_returns_error() {
        assert!(parse("w=100").is_err());
    }

    #[test]
    fn non_http_src_returns_error() {
        assert!(parse("src=ftp://bad.example.com/a.jpg&w=100").is_err());
    }

    #[test]
    fn missing_both_dims_returns_error() {
        assert!(parse("src=https://img.example.com/a.jpg").is_err());
    }

    #[test]
    fn quality_defaults_to_85() {
        let p = parse("src=https://img.example.com/a.jpg&w=100").unwrap();
        assert_eq!(p.quality, 85);
    }

    #[test]
    fn quality_clamped_to_range() {
        let p = parse("src=https://img.example.com/a.jpg&w=100&q=200").unwrap();
        assert_eq!(p.quality, 100);
    }

    #[test]
    fn width_clamped_to_8000() {
        let p = parse("src=https://img.example.com/a.jpg&w=99999").unwrap();
        assert_eq!(p.width, Some(8000));
    }

    #[test]
    fn cache_key_suffix_both_dims() {
        let p = parse("src=https://img.example.com/a.jpg&w=400&h=300&q=80").unwrap();
        assert_eq!(p.cache_key_suffix(), "w400_h300_q80");
    }

    #[test]
    fn cache_key_suffix_width_only() {
        let p = parse("src=https://img.example.com/a.jpg&w=400").unwrap();
        assert_eq!(p.cache_key_suffix(), "w400_hx_q85");
    }
}
