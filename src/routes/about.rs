use crate::models::SiteConfig;
use crate::state::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::response::IntoResponse;

#[derive(Template, WebTemplate)]
#[template(path = "about.html")]
pub struct AboutTemplate {
    pub config: SiteConfig,
    pub meta_title: String,
    pub meta_description: String,
    pub meta_url: String,
    pub meta_type: String,
    pub og_image: String,
}

pub async fn about(State(state): State<AppState>) -> impl IntoResponse {
    AboutTemplate {
        config: state.config.clone(),
        meta_title: "About — Yafi Alhakim".into(),
        meta_description: "Background, experience, and stack of Yafi Alhakim.".into(),
        meta_url: format!("{}/about", state.config.base_url),
        meta_type: "website".into(),
        og_image: crate::models::og_image(&state.config.base_url),
    }
}
