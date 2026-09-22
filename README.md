# Yafi Alhakim — Portfolio + Blog

A minimal portfolio and blog engine built with Rust, Axum, and Askama templates.

## Stack

- **Axum 0.8** for the HTTP server
- **Askama** templates (with `askama_web` for Axum integration)
- **pulldown-cmark** for Markdown → HTML
- **syntect** for class-based syntax highlighting in code blocks
- **serde_yaml** for YAML frontmatter parsing
- **tower-http** for static file serving

## Run

```bash
cargo run
# → http://0.0.0.0:3000
```

### Port override

```bash
PORT=8080 cargo run
# → http://0.0.0.0:8080
```

## Routes

| Route              | Description                                   |
|--------------------|-----------------------------------------------|
| `GET /`            | Home — hero, featured projects, recent posts  |
| `GET /projects`    | All projects                                  |
| `GET /blog`        | Paginated blog list (6 per page)              |
| `GET /blog?page=N` | Blog page N (backward-compatible)             |
| `GET /blog/page/N` | Blog page N (page 1 redirects to `/blog`)     |
| `GET /blog/:slug`  | Single post with prev/next navigation         |
| `GET /static/*`    | Static assets                                 |
| `GET /rss.xml`     | RSS 2.0 feed                                  |
| `GET /sitemap.xml` | XML sitemap                                   |
| (anything else)    | 404 fallback                                  |

## Adding a post

Create a new `.md` file in `content/posts/` with YAML frontmatter:

```yaml
---
title: "My Post Title"
date: 2026-05-01
slug: my-post-title
description: "A one-liner for the post listing and RSS feed."
tags: [rust, tutorial]
---

Your markdown body here.

```rust
fn main() {
    println!("Hello from Rust!");
}
```
```

The slug is used as the URL path (`/blog/my-post-title`).

## Docker

```bash
docker build -t portfolio .
docker run -p 3000:3000 portfolio
```

## Testing

```bash
cargo test
```

## Defaults Chosen

- **Design system**: Kami, an editorial paper-and-ink system. It uses a
  parchment page (`#F5F4ED`), ivory surfaces (`#FAF9F5`), warm sand secondary
  surfaces (`#E8E6DC`), solid tag tints (`#E4ECF5` / `#EEF2F7`), warm neutral
  hairlines (`#E3E1D6`), and ink blue as the only chromatic accent (`#1B365D`
  light / `#2D5A8A` on dark surfaces). Text is near-black `#141413` for
  headings and warm dark gray for body copy. The dark theme uses `#141413` as
  the page base and warm charcoal `#30302E` for cards/code. Typography is a
  single serif family for headings and body, with JetBrains Mono for metadata,
  tags, dates, nav, and code.
- **Color theme**: follows `prefers-color-scheme` by default. The nav toggle
  explicitly applies and persists `light` or `dark` in `localStorage` under the
  key `theme`; an inline head script applies that stored choice before first
  paint to avoid a flash of the wrong theme.

- **Syntax highlighting**: syntect with class-based output (`ClassStyle::Spaced`).
  The highlighted classes are mapped to the Kami palette in `static/style.css`,
  avoiding inlined colors while keeping the HTML clean.
- **Base URL**: `https://yaffalhakim1.github.io` in `src/main.rs`.
  If you deploy elsewhere, update `base_url` in `site_config()`.
- **RSS dates**: formatted as RFC 2822 with a fixed midnight UTC time (posts
  carry only a date, not a datetime).
- **Pagination**: 6 posts per page. Out-of-range pages return 404.
- **Static assets**: no fingerprinting; content is served from `./static`
  relative to the binary's working directory.
- **Templating**: Askama with owned data passed into templates (cloned per
  request). For a site of this size the allocation cost is negligible and it
  avoids lifetime gymnastics.
- **404 handling**: Axum's router fallback renders a friendly 404 page for
  unmatched routes.
## Deploy to GitHub Pages

This site is configured as a **user-site repo** at
`yaffalhakim1.github.io`. The GitHub Actions workflow in
`.github/workflows/deploy.yml` handles static export and deployment.

### Static export

```bash
cargo run -- --export dist
```

This writes the pre-rendered site to `./dist`. You can also specify another
directory:

```bash
cargo run -- --export ./some-other-dir
```

Or use the environment variable:

```bash
EXPORT_DIR=./some-other-dir cargo run -- --export
```

The export creates:

- `index.html`, `projects/index.html`
- `blog/index.html` and `blog/page/N/index.html` for pages 2..last
- one `blog/<slug>/index.html` for every post
- `404.html`, `rss.xml`, `sitemap.xml`
- `static/` copied as-is

### Workflow

On every push to `main`, the workflow:

1. checks out the repository,
2. installs the stable Rust toolchain with a minimal profile and cache,
3. runs `cargo run -- --export dist`,
4. uploads `dist/` as the GitHub Pages artifact, and
5. deploys that artifact to GitHub Pages.
