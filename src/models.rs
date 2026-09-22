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
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub author: String,
    pub github_url: String,
    pub projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: chrono::NaiveDate,
    pub slug: String,
    pub description: String,
    pub tags: Vec<String>,
}
