use crate::content::Post;
use crate::models::SiteConfig;
use crate::state::AppState;
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};

pub async fn rss(State(state): State<AppState>) -> Response {
    rss_response(&rss_string(&state.config, &state.posts))
}

pub async fn sitemap(State(state): State<AppState>) -> Response {
    sitemap_response(&sitemap_string(&state.config, &state.posts))
}

fn rss_response(xml: &str) -> Response {
    (
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        xml.to_string(),
    )
        .into_response()
}

fn sitemap_response(xml: &str) -> Response {
    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml.to_string(),
    )
        .into_response()
}

pub fn rss_string(config: &SiteConfig, posts: &[Post]) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n");
    xml.push_str(&format!("<title>{}</title>\n", xml_escape(&config.title)));
    xml.push_str(&format!("<link>{}/</link>\n", xml_escape(&config.base_url)));
    xml.push_str(&format!(
        "<description>{}</description>\n",
        xml_escape(&config.description)
    ));
    xml.push_str(&format!(
        "<atom:link href=\"{}/rss.xml\" rel=\"self\" type=\"application/rss+xml\" />\n",
        xml_escape(&config.base_url)
    ));

    for post in posts {
        xml.push_str("<item>\n");
        xml.push_str(&format!("<title>{}</title>\n", xml_escape(&post.title)));
        xml.push_str(&format!(
            "<link>{}/blog/{}</link>\n",
            xml_escape(&config.base_url),
            xml_escape(&post.slug)
        ));
        xml.push_str(&format!(
            "<description>{}</description>\n",
            xml_escape(&post.description)
        ));
        xml.push_str(&format!("<pubDate>{}</pubDate>\n", rfc2822(post.date)));
        xml.push_str(&format!(
            "<guid>{}/blog/{}</guid>\n",
            xml_escape(&config.base_url),
            xml_escape(&post.slug)
        ));
        xml.push_str("</item>\n");
    }

    xml.push_str("</channel>\n</rss>\n");
    xml
}

pub fn sitemap_string(config: &SiteConfig, posts: &[Post]) -> String {
    let base = &config.base_url;
    let total_pages = crate::routes::blog::page_count(posts.len(), crate::routes::blog::PAGE_SIZE);
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for path in ["/", "/projects", "/blog"] {
        xml.push_str(&format!("<url><loc>{base}{path}</loc></url>\n"));
    }

    for page in 2..=total_pages {
        xml.push_str(&format!("<url><loc>{base}/blog/page/{page}</loc></url>\n"));
    }

    for post in posts {
        xml.push_str(&format!(
            "<url><loc>{base}/blog/{}</loc></url>\n",
            xml_escape(&post.slug)
        ));
    }

    xml.push_str("</urlset>\n");
    xml
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn rfc2822(date: chrono::NaiveDate) -> String {
    use chrono::Datelike;
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let day = date.weekday().num_days_from_monday() as usize;
    format!(
        "{}, {:02} {} {} 00:00:00 +0000",
        days[day],
        date.day(),
        months[date.month() as usize - 1],
        date.year()
    )
}
