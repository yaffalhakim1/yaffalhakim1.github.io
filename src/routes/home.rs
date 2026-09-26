use crate::content::Post;
use crate::models::{Project, SiteConfig};
use crate::state::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::response::IntoResponse;

#[derive(Template, WebTemplate)]
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

pub async fn home(State(state): State<AppState>) -> impl IntoResponse {
    IndexTemplate {
        config: state.config.clone(),
        featured: state.config.projects.iter().take(4).cloned().collect(),
        recent: state.posts.iter().take(5).cloned().collect(),
        meta_title: format!("{} — {}", state.config.author, state.config.title),
        meta_description: state.config.description.clone(),
        meta_url: format!("{}/", state.config.base_url),
        meta_type: "website".into(),
        og_image: crate::models::og_image(&state.config.base_url),
    }
}
