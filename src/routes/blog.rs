use crate::content::Post;
use crate::models::SiteConfig;
use crate::state::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

pub const PAGE_SIZE: usize = 6;

#[derive(Debug, Deserialize)]
pub struct BlogQuery {
    pub page: Option<usize>,
}

pub fn page_count(total: usize, per_page: usize) -> usize {
    if per_page == 0 {
        return 0;
    }
    (total + per_page - 1) / per_page
}

pub fn slice_page<T>(items: &[T], page: usize, per_page: usize) -> &[T] {
    let start = page.saturating_mul(per_page);
    if start >= items.len() {
        return &[];
    }
    let end = (start + per_page).min(items.len());
    &items[start..end]
}

#[derive(Template, WebTemplate)]
#[template(path = "blog_list.html")]
pub struct BlogListTemplate {
    pub posts: Vec<Post>,
    pub page: usize,
    pub total_pages: usize,
    pub meta_title: String,
    pub meta_description: String,
    pub meta_url: String,
    pub meta_type: String,
    pub og_image: String,
}

pub async fn blog(State(state): State<AppState>, Query(query): Query<BlogQuery>) -> Response {
    let total_pages = page_count(state.posts.len(), PAGE_SIZE);
    let page = query.page.unwrap_or(1).max(1);

    if page > total_pages && total_pages > 0 {
        return not_found_response(&state.config.base_url);
    }

    let posts = slice_page(&state.posts, page.saturating_sub(1), PAGE_SIZE).to_vec();

    BlogListTemplate {
        posts,
        page,
        total_pages,
        meta_title: format!("Blog — Yafi Alhakim (page {page})"),
        meta_description: "Technical writing on Rust, React, and web engineering.".into(),
        meta_url: format!("{}/blog/page/{page}", state.config.base_url),
        meta_type: "website".into(),
        og_image: crate::models::og_image(&state.config.base_url),
    }
    .into_response()
}

pub fn blog_page_html(config: &SiteConfig, posts: &[Post], page: usize) -> Option<String> {
    let total_pages = page_count(posts.len(), PAGE_SIZE);
    if page == 0 || page > total_pages {
        return None;
    }
    let page_posts = slice_page(posts, page - 1, PAGE_SIZE).to_vec();
    let template = BlogListTemplate {
        posts: page_posts,
        page,
        total_pages,
        meta_title: format!("Blog — Yafi Alhakim (page {page})"),
        meta_description: "Technical writing on Rust, React, and web engineering.".into(),
        meta_url: format!("{}/blog/page/{page}", config.base_url),
        meta_type: "website".into(),
        og_image: crate::models::og_image(&config.base_url),
    };
    Some(template.render().expect("failed to render blog page"))
}

pub fn blog_index_html(config: &SiteConfig, posts: &[Post]) -> String {
    blog_page_html(config, posts, 1).expect("failed to render blog index")
}

#[derive(Template, WebTemplate)]
#[template(path = "blog_post.html")]
pub struct BlogPostTemplate {
    pub post: Post,
    pub prev: Option<Post>,
    pub next: Option<Post>,
    pub meta_title: String,
    pub meta_description: String,
    pub meta_url: String,
    pub meta_type: String,
    pub og_image: String,
}

pub fn blog_post_html(config: &SiteConfig, posts: &[Post], slug: &str) -> Option<String> {
    posts
        .iter()
        .position(|post| post.slug == slug)
        .map(|index| {
            let post = posts[index].clone();
            let slug = post.slug.clone();
            let title = post.title.clone();
            let description = post.description.clone();
            let prev = if index > 0 {
                Some(posts[index - 1].clone())
            } else {
                None
            };
            let next = posts.get(index + 1).cloned();
            let template = BlogPostTemplate {
                post,
                prev,
                next,
                meta_title: format!("{} — Yafi Alhakim", title),
                meta_description: description,
                meta_url: format!("{}/blog/{}", config.base_url, slug),
                meta_type: "article".into(),
                og_image: crate::models::og_image(&config.base_url),
            };
            template.render().expect("failed to render blog post")
        })
}

pub async fn blog_post(State(state): State<AppState>, Path(slug): Path<String>) -> Response {
    match state.posts.iter().position(|p| p.slug == slug) {
        Some(index) => {
            let post = state.posts[index].clone();
            let slug = post.slug.clone();
            let title = post.title.clone();
            let description = post.description.clone();
            let prev = if index > 0 {
                Some(state.posts[index - 1].clone())
            } else {
                None
            };
            let next = state.posts.get(index + 1).cloned();
            BlogPostTemplate {
                post,
                prev,
                next,
                meta_title: format!("{} — Yafi Alhakim", title),
                meta_description: description,
                meta_url: format!("{}/blog/{}", state.config.base_url, slug),
                meta_type: "article".into(),
                og_image: crate::models::og_image(&state.config.base_url),
            }
            .into_response()
        }
        None => not_found_response(&state.config.base_url),
    }
}

pub async fn blog_page(State(state): State<AppState>, Path(page): Path<String>) -> Response {
    match page.parse::<usize>() {
        Ok(1) => Redirect::to("/blog").into_response(),
        Ok(page) if page > 1 => {
            let total_pages = page_count(state.posts.len(), PAGE_SIZE);
            if page > total_pages {
                not_found_response(&state.config.base_url)
            } else {
                let posts = slice_page(&state.posts, page.saturating_sub(1), PAGE_SIZE).to_vec();
                BlogListTemplate {
                    posts,
                    page,
                    total_pages,
                    meta_title: format!("Blog — Yafi Alhakim (page {page})"),
                    meta_description: "Technical writing on Rust, React, and web engineering."
                        .into(),
                    meta_url: format!("{}/blog/page/{page}", state.config.base_url),
                    meta_type: "website".into(),
                    og_image: crate::models::og_image(&state.config.base_url),
                }
                .into_response()
            }
        }
        _ => not_found_response(&state.config.base_url),
    }
}

pub async fn not_found_handler(State(state): State<AppState>) -> Response {
    not_found_response(&state.config.base_url)
}

fn not_found_response(base_url: &str) -> Response {
    let html = not_found_html(base_url);
    (StatusCode::NOT_FOUND, axum::response::Html(html)).into_response()
}

pub fn not_found_html(base_url: &str) -> String {
    crate::routes::NotFoundTemplate {
        meta_title: "404 — Not Found".into(),
        meta_description: "The page you are looking for does not exist.".into(),
        meta_url: "/".into(),
        meta_type: "website".into(),
        og_image: crate::models::og_image(base_url),
    }
    .render()
    .unwrap_or_else(|_| "<h1>404</h1>".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SiteConfig;

    fn test_config() -> SiteConfig {
        SiteConfig {
            title: "Yafi Alhakim".into(),
            description: "desc".into(),
            base_url: "https://example.com".into(),
            author: "Test".into(),
            github_url: "https://github.com/test".into(),
            role: "Frontend Engineer".into(),
            location: "Jakarta, Indonesia".into(),
            bio: "bio".into(),
            projects: Vec::new(),
            experience: Vec::new(),
            tools: Vec::new(),
            links: Vec::new(),
            hero_cards: Vec::new(),
        }
    }

    fn test_post(slug: &str) -> Post {
        Post {
            title: slug.into(),
            date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            slug: slug.into(),
            description: "desc".into(),
            tags: Vec::new(),
            html: "<p>body</p>".into(),
        }
    }

    #[test]
    fn blog_page_html_uses_path_based_pagination_links() {
        let config = test_config();
        let posts: Vec<Post> = (0..12).map(|i| test_post(&format!("post-{i}"))).collect();

        let html = blog_page_html(&config, &posts, 2).expect("page 2 should render");
        assert!(html.contains("href=\"/blog/page/1\""));
        assert!(html.contains("href=\"/blog/page/1\""));
        assert!(!html.contains("href=\"/blog?page="));
    }

    #[test]
    fn blog_page_html_rejects_out_of_range() {
        let config = test_config();
        let posts: Vec<Post> = (0..12).map(|i| test_post(&format!("post-{i}"))).collect();

        assert!(blog_page_html(&config, &posts, 3).is_none());
        assert!(blog_page_html(&config, &posts, 0).is_none());
    }
}
