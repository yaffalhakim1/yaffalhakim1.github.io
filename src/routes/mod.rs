pub mod blog;
pub mod feeds;
pub mod home;
pub mod projects;

use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {
    pub meta_title: String,
    pub meta_description: String,
    pub meta_url: String,
    pub meta_type: String,
}
