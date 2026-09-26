use crate::content::Post;
use crate::models::SiteConfig;
use crate::steam::SteamData;

#[derive(Clone)]
pub struct AppState {
    pub config: SiteConfig,
    pub posts: Vec<Post>,
    pub steam: Option<SteamData>,
}
