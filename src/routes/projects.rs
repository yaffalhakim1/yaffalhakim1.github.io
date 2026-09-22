use crate::models::SiteConfig;
use crate::state::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::response::IntoResponse;

#[derive(Template, WebTemplate)]
#[template(path = "projects.html")]
pub struct ProjectsTemplate {
    pub config: SiteConfig,
    pub meta_title: String,
    pub meta_description: String,
    pub meta_url: String,
    pub meta_type: String,
}

pub async fn projects(State(state): State<AppState>) -> impl IntoResponse {
    ProjectsTemplate {
        config: state.config.clone(),
        meta_title: "Projects — Yafi Alhakim".into(),
        meta_description: "Open source projects and tools built by Yafi Alhakim.".into(),
        meta_url: format!("{}/projects", state.config.base_url),
        meta_type: "website".into(),
    }
}
