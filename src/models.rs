use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub source: Option<String>,
    pub demo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Experience {
    pub role: String,
    pub company: String,
    pub period: String,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolGroup {
    pub name: String,
    pub items: Vec<Tool>,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct HeroCard {
    pub label: String,
    pub value: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub author: String,
    pub github_url: String,
    pub role: String,
    pub location: String,
    pub bio: String,
    pub projects: Vec<Project>,
    pub experience: Vec<Experience>,
    pub tools: Vec<ToolGroup>,
    pub links: Vec<Link>,
    pub hero_cards: Vec<HeroCard>,
}

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: chrono::NaiveDate,
    pub slug: String,
    pub description: String,
    pub tags: Vec<String>,
}
/// Absolute URL of the Open Graph preview image for a page.
pub fn og_image(base_url: &str) -> String {
    format!("{base_url}/static/og.png")
}