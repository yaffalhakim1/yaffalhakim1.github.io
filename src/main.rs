mod content;
mod models;
mod render;
mod routes;
mod state;

use crate::state::AppState;
use axum::Router;
use axum::routing::{get, get_service};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;

fn site_config() -> models::SiteConfig {
    models::SiteConfig {
        title: "Yafi Alhakim".into(),
        description: "Fullstack Engineer building pragmatic tools for the web and desktop."
            .into(),
        base_url: "https://yaffalhakim1.github.io".into(),
        author: "Muhammad Yafi Alhakim".into(),
        github_url: "https://github.com/yaffalhakim1".into(),
        projects: vec![
            models::Project {
                name: "habit-terminal".into(),
                description: "Terminal-themed habit tracker with gamified RPG progression."
                    .into(),
                tags: vec!["React", "TypeScript", "Vite", "TanStack Query", "CSS"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                source: Some("https://github.com/yaffalhakim1/habit-terminal".into()),
                demo: Some("https://yaffalhakim1.github.io/habit-terminal/".into()),
            },
            models::Project {
                name: "NowOnTaskbar".into(),
                description:
                    "Show what's playing on your Windows taskbar (Spotify, YouTube, Chrome, Edge, VLC)."
                        .into(),
                tags: vec!["C#", "WinRT", "Win32", "GDI"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                source: Some("https://github.com/yaffalhakim1/nowplaying-windows".into()),
                demo: None,
            },
            models::Project {
                name: "Kerenzikov".into(),
                description:
                    "A native app for all your coding agents. GPUI-based, Windows and Android, forked from waku and rebranded."
                        .into(),
                tags: vec!["Rust", "GPUI"].into_iter().map(Into::into).collect(),
                source: Some("https://github.com/yaffalhakim1/Kerenzikov-app".into()),
                demo: None,
            },
            models::Project {
                name: "Codex Discord Bridge".into(),
                description:
                    "Lightweight Rust bridge connecting Codex to Discord for remote monitoring and approvals."
                        .into(),
                tags: vec!["Rust", "Discord"].into_iter().map(Into::into).collect(),
                source: Some("https://github.com/yaffalhakim1/codex-discord-bridge".into()),
                demo: None,
            },
            models::Project {
                name: "Simple E-commerce".into(),
                description:
                    "E-commerce with Next.js App Router + React Server Components.".into(),
                tags: vec!["Next.js", "TypeScript", "RSC"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                source: None,
                demo: Some("https://simple-ecommerce-appdir.vercel.app/".into()),
            },
            models::Project {
                name: "Sumz".into(),
                description: "Summarize any article with AI in seconds.".into(),
                tags: vec!["React", "TypeScript", "OpenAI API"]
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                source: None,
                demo: Some("https://summarizer-drab.vercel.app/".into()),
            },
        ],
    }
}

fn export_dir() -> PathBuf {
    std::env::var("EXPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("dist"))
}

fn export_site(out_dir: &PathBuf) {
    let config = site_config();
    let posts = content::load_posts();
    let state = AppState { config, posts };

    if out_dir.exists() {
        std::fs::remove_dir_all(out_dir).expect("failed to clear export dir");
    }
    std::fs::create_dir_all(out_dir).expect("failed to create export dir");

    write_file(
        &out_dir.join("index.html"),
        &render::render_index(&state.config, &state.posts),
    );
    write_file(
        &out_dir.join("projects/index.html"),
        &render::render_projects(&state.config),
    );
    write_file(
        &out_dir.join("blog/index.html"),
        &render::render_blog_index(&state.config, &state.posts),
    );

    let total_pages = routes::blog::page_count(state.posts.len(), routes::blog::PAGE_SIZE);
    for page in 1..=total_pages {
        let html = render::render_blog_page(&state.config, &state.posts, page)
            .expect("failed to render pagination page");
        write_file(&out_dir.join(format!("blog/page/{page}/index.html")), &html);
    }

    for post in &state.posts {
        let html = render::render_blog_post(&state.config, &state.posts, &post.slug)
            .unwrap_or_else(|| panic!("failed to render post {}", post.slug));
        write_file(
            &out_dir.join(format!("blog/{}/index.html", post.slug)),
            &html,
        );
    }

    write_file(&out_dir.join("404.html"), &render::render_not_found());

    let rss = routes::feeds::rss_string(&state.config, &state.posts);
    let sitemap = routes::feeds::sitemap_string(&state.config, &state.posts);
    write_file(&out_dir.join("rss.xml"), &rss);
    write_file(&out_dir.join("sitemap.xml"), &sitemap);

    copy_dir("static", &out_dir.join("static"));

    eprintln!("Exported site to {}", out_dir.display());
}

fn write_file(path: &PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    std::fs::write(path, contents).expect("failed to write export file");
}

fn copy_dir(src: &str, dst: &PathBuf) {
    std::fs::create_dir_all(dst).expect("failed to create destination directory");
    for entry in std::fs::read_dir(src).expect("failed to read static source") {
        let entry = entry.expect("failed to read static entry");
        let file_type = entry.file_type().expect("failed to get file type");
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir(entry.path().to_str().unwrap(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("failed to copy static file");
        }
    }
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);

    if let Some(arg) = args.next() {
        if arg == "--export" {
            let dir = args.next().map(PathBuf::from).unwrap_or_else(export_dir);
            export_site(&dir);
            return;
        }
        eprintln!("Unknown argument: {arg}");
        std::process::exit(2);
    }

    let config = site_config();
    let posts = content::load_posts();

    eprintln!(
        "Loaded {} posts from {}",
        posts.len(),
        content::posts_dir().display()
    );

    let state = AppState { config, posts };

    let static_service = get_service(ServeDir::new("static"));

    let app = Router::new()
        .route("/", get(routes::home::home))
        .route("/projects", get(routes::projects::projects))
        .route("/blog", get(routes::blog::blog))
        .route("/blog/page/{page}", get(routes::blog::blog_page))
        .route("/blog/{slug}", get(routes::blog::blog_post))
        .route("/rss.xml", get(routes::feeds::rss))
        .route("/sitemap.xml", get(routes::feeds::sitemap))
        .nest_service("/static", static_service)
        .fallback(routes::blog::not_found_handler)
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    eprintln!("listening on http://{addr}");

    axum::serve(listener, app).await.expect("server error");
}
