mod content;
mod models;
mod render;
mod routes;
mod state;
mod steam;

use crate::state::AppState;
use axum::Router;
use axum::routing::{get, get_service};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::services::ServeDir;

fn site_config() -> models::SiteConfig {
    models::SiteConfig {
        title: "Yafi Alhakim".into(),
        description: concat!(
            "Fullstack Engineer from Indonesia, currently at MySkill.id. I build for the web ",
            "and the desktop — fast pages, honest APIs, and interfaces that stay out of the way."
        )
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
        role: "Frontend Engineer".into(),
        location: "Jakarta, Indonesia".into(),
        bio: concat!(
            "I build for the web and the desktop, and I care most about the seam between them: ",
            "fast pages, honest APIs, and interfaces that stay out of the way. When not debugging ",
            "or sketching API routes, I write up the journey — a tutorial, a deployment log, ",
            "or an aha moment."
        )
        .into(),
        experience: experience(),
        tools: tools(),
        links: links(),
        hero_cards: hero_cards(),
        static_games: static_games(),
    }
}

fn static_games() -> Vec<models::StaticGame> {
    vec![
        models::StaticGame {
            name: "Assassin's Creed Shadows".into(),
            icon_url: "https://media.steampowered.com/steamcommunity/public/images/apps/3159330/1f8f5a64b174f9cc07d5b277e7b08219c262511f.jpg".into(),
            playtime_hours: 100,
            unlocked_achievements: 100,
            total_achievements: 100,
            store_url: Some("https://store.steampowered.com/app/3159330".into()),
        },
        models::StaticGame {
            name: "Ghost of Tsushima".into(),
            icon_url: "https://media.steampowered.com/steamcommunity/public/images/apps/2215430/e87b8cbe31f7bc5f40ee6ed94ccfa18f59f04fbc.jpg".into(),
            playtime_hours: 80,
            unlocked_achievements: 34,
            total_achievements: 52,
            store_url: Some("https://store.steampowered.com/app/2215430".into()),
        },
        models::StaticGame {
            name: "Marvel's Spider-Man Remastered".into(),
            icon_url: "https://media.steampowered.com/steamcommunity/public/images/apps/1817070/346333cb340139ad8b697005e5c79a3162c387b0.jpg".into(),
            playtime_hours: 70,
            unlocked_achievements: 25,
            total_achievements: 50,
            store_url: Some("https://store.steampowered.com/app/1817070".into()),
        },
        models::StaticGame {
            name: "Marvel's Spider-Man 2".into(),
            icon_url: "https://media.steampowered.com/steamcommunity/public/images/apps/2651280/74853ef20b2cce99818a4732ffb38a1234db2827.jpg".into(),
            playtime_hours: 70,
            unlocked_achievements: 25,
            total_achievements: 50,
            store_url: Some("https://store.steampowered.com/app/2651280".into()),
        },
    ]
}

fn experience() -> Vec<models::Experience> {
    vec![
        models::Experience {
            role: "Frontend Engineer".into(),
            company: "MySkill.id".into(),
            period: "Feb 2024 — Present".into(),
        },
        models::Experience {
            role: "Frontend Engineer Trainee".into(),
            company: "Sea Labs Indonesia".into(),
            period: "Oct 2023 — Feb 2024".into(),
        },
        models::Experience {
            role: "Frontend Engineer".into(),
            company: "Diponegoro University".into(),
            period: "Dec 2022 — Apr 2023".into(),
        },
    ]
}

fn tools() -> Vec<models::ToolGroup> {
    fn group(name: &str, items: &[(&str, Option<&str>)]) -> models::ToolGroup {
        models::ToolGroup {
            name: name.into(),
            items: items
                .iter()
                .map(|(name, url)| models::Tool {
                    name: (*name).into(),
                    url: url.map(Into::into),
                })
                .collect(),
        }
    }

    vec![
        group(
            "Frontend",
            &[
                ("React", Some("https://react.dev")),
                ("Next.js", Some("https://nextjs.org")),
                ("TypeScript", Some("https://www.typescriptlang.org")),
                ("Tailwind CSS", Some("https://tailwindcss.com")),
                ("Astro", Some("https://astro.build")),
                ("Chakra UI", Some("https://chakra-ui.com")),
            ],
        ),
        group(
            "Backend & Infra",
            &[
                ("Rust", Some("https://www.rust-lang.org")),
                ("Axum", Some("https://github.com/tokio-rs/axum")),
                ("Node.js", Some("https://nodejs.org")),
                ("Supabase", Some("https://supabase.com")),
                ("Docker", Some("https://www.docker.com")),
                ("cPanel", None),
            ],
        ),
        group(
            "Tooling",
            &[
                ("VS Code", Some("https://code.visualstudio.com")),
                ("Git", Some("https://git-scm.com")),
                ("Postman", Some("https://www.postman.com")),
                ("Figma", Some("https://www.figma.com")),
                ("Notion", Some("https://www.notion.so")),
                ("Vercel", Some("https://vercel.com")),
            ],
        ),
    ]
}

fn links() -> Vec<models::Link> {
    vec![
        models::Link {
            name: "CV".into(),
            url: "https://self.so/yafialhakim".into(),
        },
        models::Link {
            name: "GitHub".into(),
            url: "https://github.com/yaffalhakim1".into(),
        },
        models::Link {
            name: "LinkedIn".into(),
            url: "https://www.linkedin.com/in/yafialhakim/".into(),
        },
        models::Link {
            name: "Fastwork".into(),
            url: "https://fastwork.id/user/yaffalhaki/web-development-26119147".into(),
        },
        models::Link {
            name: "Gumroad".into(),
            url: "https://6679524908482.gumroad.com/l/nothing-design-astro".into(),
        },
        models::Link {
            name: "Email".into(),
            url: "mailto:yafialhakim64@gmail.com".into(),
        },
    ]
}

fn hero_cards() -> Vec<models::HeroCard> {
    vec![
        models::HeroCard {
            label: "Role".into(),
            value: "Frontend Engineer @ MySkill.id".into(),
            url: None,
        },
        models::HeroCard {
            label: "Location".into(),
            value: "Jakarta, Indonesia".into(),
            url: None,
        },
        models::HeroCard {
            label: "Services".into(),
            value: "Hire me on Fastwork".into(),
            url: Some("https://fastwork.id/user/yaffalhaki/web-development-26119147".into()),
        },
        models::HeroCard {
            label: "Templates".into(),
            value: "Nothing Theme on Gumroad".into(),
            url: Some("https://6679524908482.gumroad.com/l/nothing-design-astro".into()),
        },
    ]
}

fn export_dir() -> PathBuf {
    std::env::var("EXPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("dist"))
}

fn export_site(out_dir: &PathBuf) {
    let config = site_config();
    let posts = content::load_posts();
    let state = AppState {
        config,
        posts,
        steam: steam::load(),
    };

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
        &out_dir.join("about/index.html"),
        &render::render_about(&state.config),
    );
    write_file(
        &out_dir.join("games/index.html"),
        &render::render_games(&state),
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

    write_file(
        &out_dir.join("404.html"),
        &render::render_not_found(&state.config.base_url),
    );

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

fn main() {
    let mut args = std::env::args().skip(1);

    if let Some(arg) = args.next() {
        if arg == "--refresh-steam" {
            match steam::refresh() {
                Ok(data) => match steam::write_snapshot(&data) {
                    Ok(()) => {
                        eprintln!(
                            "Wrote {} games, {} achievement sets to {}",
                            data.total_games,
                            data.achievements.len(),
                            steam::snapshot_path().display()
                        );
                        return;
                    }
                    Err(error) => {
                        eprintln!("Failed to write snapshot: {error}");
                        std::process::exit(1);
                    }
                },
                Err(error) => {
                    eprintln!("Failed to refresh Steam data: {error}");
                    std::process::exit(1);
                }
            }
        }
        if arg == "--export" {
            let dir = args.next().map(PathBuf::from).unwrap_or_else(export_dir);
            export_site(&dir);
            return;
        }
        eprintln!("Unknown argument: {arg}");
        std::process::exit(2);
    }

    serve();
}

#[tokio::main]
async fn serve() {
    let config = site_config();
    let posts = content::load_posts();

    eprintln!(
        "Loaded {} posts from {}",
        posts.len(),
        content::posts_dir().display()
    );

    let state = AppState {
        config,
        posts,
        steam: steam::load(),
    };

    let static_service = get_service(ServeDir::new("static"));

    let app = Router::new()
        .route("/", get(routes::home::home))
        .route("/about", get(routes::about::about))
        .route("/games", get(routes::games::games))
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
