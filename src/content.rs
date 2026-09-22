use crate::models::Frontmatter;
use chrono::NaiveDate;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::fs;
use std::path::{Path, PathBuf};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;

#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub date: NaiveDate,
    pub slug: String,
    pub description: String,
    pub tags: Vec<String>,
    pub html: String,
}

pub fn posts_dir() -> PathBuf {
    PathBuf::from("content/posts")
}

pub fn load_posts() -> Vec<Post> {
    let dir = posts_dir();
    let mut posts = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Ok(post) = parse_post(&path) {
                    posts.push(post);
                }
            }
        }
    }

    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.slug.cmp(&a.slug)));
    posts
}

pub fn parse_post(path: &Path) -> Result<Post, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;

    let content = raw.strip_prefix("---\n").unwrap_or(&raw);
    let fm_end = content
        .find("\n---\n")
        .ok_or_else(|| format!("{}: missing frontmatter terminator", path.display()))?;
    let fm_str = &content[..fm_end];
    let body = &content[fm_end + 5..];

    let fm: Frontmatter =
        serde_yaml::from_str(fm_str).map_err(|e| format!("{}: {}", path.display(), e))?;

    let html = render_markdown(body);
    Ok(Post {
        title: fm.title,
        date: fm.date,
        slug: fm.slug,
        description: fm.description,
        tags: fm.tags,
        html,
    })
}

/// Render markdown body to HTML with syntect class-based code highlighting.
fn render_markdown(body: &str) -> String {
    let syntax_set = SyntaxSet::load_defaults_newlines();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(body, options);
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut events: Vec<Event> = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_buf.clear();
                if let CodeBlockKind::Fenced(lang) = &kind {
                    code_lang = lang.to_string();
                } else {
                    code_lang.clear();
                }
            }
            Event::Text(text) if in_code => {
                code_buf.push_str(&text);
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code = false;
                let syntax = syntax_set
                    .find_syntax_by_token(&code_lang)
                    .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
                let mut html_gen = ClassedHTMLGenerator::new_with_class_style(
                    syntax,
                    &syntax_set,
                    ClassStyle::Spaced,
                );
                for line in syntect::util::LinesWithEndings::from(&code_buf) {
                    let _ = html_gen.parse_html_for_line_which_includes_newline(line);
                }
                let inner = html_gen.finalize();
                events.push(Event::Html(
                    format!("<pre class=\"code-block\"><code>{inner}</code></pre>\n").into(),
                ));
            }
            e => events.push(e),
        }
    }

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::blog::slice_page;
    use std::io::Write;

    #[test]
    fn test_parse_post_frontmatter() {
        let tmp = std::env::temp_dir().join(format!("test_post_{}.md", std::process::id()));
        let mut f = fs::File::create(&tmp).unwrap();
        writeln!(
            f,
            "---\ntitle: \"Test Post\"\ndate: 2025-01-01\nslug: test-post\ndescription: \"desc\"\ntags: [rust, test]\n---\n\n# Hello\n\nWorld."
        )
        .unwrap();
        let post = parse_post(&tmp).unwrap();
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.slug, "test-post");
        assert_eq!(post.tags, vec!["rust", "test"]);
        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_load_posts_sorted_desc() {
        let posts = load_posts();
        assert!(posts.len() >= 2);
        for i in 0..posts.len() - 1 {
            assert!(
                posts[i].date >= posts[i + 1].date,
                "posts not sorted desc at index {i}"
            );
        }
    }

    #[test]
    fn test_page_count() {
        assert_eq!(crate::routes::blog::page_count(30, 6), 5);
        assert_eq!(crate::routes::blog::page_count(0, 6), 0);
        assert_eq!(crate::routes::blog::page_count(7, 6), 2);
    }

    #[test]
    fn test_slice_page() {
        let items: Vec<u32> = (0..30).collect();
        let page5 = slice_page(&items, 4, 6);
        assert_eq!(page5, &[24, 25, 26, 27, 28, 29]);
        assert!(slice_page(&items, 5, 6).is_empty());
    }
}
