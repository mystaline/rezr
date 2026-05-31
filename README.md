# rezr

Rust-based serverless image resizer for Cloudflare Workers.

```
GET /?src=https://example.com/photo.jpg&w=400&q=80
```

---

## Query params

| Param | Required | Description | Default | Values |
|-------|----------|-------------|---------|--------|
| `src` | yes | Source image URL (http/https) | — | — |
| `w` | one of w/h | Output width in px | — | 1–8000 |
| `h` | one of w/h | Output height in px | — | 1–8000 |
| `q` | no | JPEG quality | 85 | 1–100 |
| `fit` | no | Resize mode | `contain` | `contain`, `exact`, `crop` |

- `fit=contain` (default) — fits within the given box, aspect ratio always preserved
- `fit=exact` — forces exact `w`×`h`, aspect ratio not preserved (warps image)
- `fit=crop` — scales to cover the box preserving AR, center-crops overflow
- Upscaling is skipped for `contain` and `exact` — if target > original, original is returned
- Output is always JPEG

---

## Deploy

```sh
npm install
npx wrangler deploy
```

Set a custom domain in Cloudflare dashboard → Workers → rezr → Custom Domains.

---

## Local dev

```sh
npm install
npx wrangler dev
```

---

## Testing

Unit tests cover param parsing and image resize logic (no Workers runtime required):

```sh
cargo test
```

Integration testing (cache, full request flow) requires `wrangler dev` running and hitting `http://localhost:8787` directly.

---

## Stack

- Rust → WASM via [`worker-rs`](https://github.com/cloudflare/workers-rs)
- Image processing: [`image`](https://crates.io/crates/image) (Lanczos3)
- Edge caching via Cloudflare Cache API
