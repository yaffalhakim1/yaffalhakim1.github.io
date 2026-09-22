use crate::content::Post;
use crate::models::SiteConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: SiteConfig,
    pub posts: Vec<Post>,
}
