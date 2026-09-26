use crate::content::Post;
use crate::models::{Project, SiteConfig};
use crate::routes::about::AboutTemplate;
use crate::routes::games::games_html;
use crate::state::AppState;
use crate::routes::blog::{blog_index_html, blog_page_html, blog_post_html, not_found_html};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub config: SiteConfig,
    pub featured: Vec<Project>,
    pub recent: Vec<Post>,
    pub meta_title: String,
    pub meta_description: String,
    pub meta_url: String,
    pub meta_type: String,
    pub og_image: String,
}

#[derive(Template)]
#[template(path = "projects.html")]
pub struct ProjectsTemplate {
    pub config: SiteConfig,
    pub meta_title: String,
    pub meta_description: String,
    pub meta_url: String,
    pub meta_type: String,
    pub og_image: String,
}

pub fn render_index(config: &SiteConfig, posts: &[Post]) -> String {
    let featured: Vec<_> = config.projects.iter().take(4).cloned().collect();
    let recent: Vec<_> = posts.iter().take(5).cloned().collect();
    let template = IndexTemplate {
        config: config.clone(),
        featured,
        recent,
        meta_title: format!("{} — {}", config.author, config.title),
        meta_description: config.description.clone(),
        meta_url: format!("{}/", config.base_url),
        meta_type: "website".into(),
        og_image: crate::models::og_image(&config.base_url),
    };
    template.render().expect("failed to render index")
}

pub fn render_about(config: &SiteConfig) -> String {
    let template = AboutTemplate {
        config: config.clone(),
        meta_title: "About — Yafi Alhakim".into(),
        meta_description: "Background, experience, and stack of Yafi Alhakim.".into(),
        meta_url: format!("{}/about", config.base_url),
        meta_type: "website".into(),
        og_image: crate::models::og_image(&config.base_url),
    };
    template.render().expect("failed to render about")
}

pub fn render_projects(config: &SiteConfig) -> String {
    let template = ProjectsTemplate {
        config: config.clone(),
        meta_title: "Projects — Yafi Alhakim".into(),
        meta_description: "Open source projects and tools built by Yafi Alhakim.".into(),
        meta_url: format!("{}/projects", config.base_url),
        meta_type: "website".into(),
        og_image: crate::models::og_image(&config.base_url),
    };
    template.render().expect("failed to render projects")
}

pub fn render_blog_index(config: &SiteConfig, posts: &[Post]) -> String {
    blog_index_html(config, posts)
}

pub fn render_blog_page(config: &SiteConfig, posts: &[Post], page: usize) -> Option<String> {
    blog_page_html(config, posts, page)
}

pub fn render_blog_post(config: &SiteConfig, posts: &[Post], slug: &str) -> Option<String> {
    blog_post_html(config, posts, slug)
}

pub fn render_games(state: &AppState) -> String {
    games_html(state)
}

pub fn render_not_found(base_url: &str) -> String {
    not_found_html(base_url)
}
