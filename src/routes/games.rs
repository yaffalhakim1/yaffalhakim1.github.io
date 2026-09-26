use crate::state::AppState;
use crate::steam::{self, SteamData};
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::response::IntoResponse;

/// A game flattened for display, with playtime already formatted so the
/// template stays free of logic.
pub struct GameView {
    pub appid: u64,
    pub name: String,
    pub icon_url: String,
    pub playtime: String,
    pub recent: String,
    pub raw_playtime_minutes: u64,
    pub store_url: Option<String>,
}

/// Rows below this share are noise: one unlucky unlock on a 100-achievement
/// game would otherwise pad the list with near-empty bars.
const PROGRESS_MIN_PERCENT: usize = 25;

pub struct GameProgress {
    pub name: String,
    pub icon_url: String,
    pub unlocked: usize,
    pub total: usize,
    pub percent: usize,
}

#[derive(Template, WebTemplate)]
#[template(path = "games.html")]
pub struct GamesTemplate {
    pub data: Option<SteamData>,
    pub heatmap: Option<steam::Heatmap>,
    pub recent: Vec<GameView>,
    pub top_games: Vec<GameView>,
    pub progress: Vec<GameProgress>,
    pub total_playtime: String,
    pub meta_title: String,
    pub meta_description: String,
    pub meta_url: String,
    pub meta_type: String,
    pub og_image: String,
}

fn keep_progress_row(row: &GameProgress) -> bool {
    row.percent >= PROGRESS_MIN_PERCENT
}

fn view(game: &steam::Game) -> GameView {
    let icon_url = if game.img_icon_url.is_empty() {
        String::new()
    } else {
        format!(
            "https://media.steampowered.com/steamcommunity/public/images/apps/{}/{}.jpg",
            game.appid, game.img_icon_url
        )
    };
    GameView {
        appid: game.appid,
        name: game.name.clone(),
        icon_url,
        playtime: steam::format_playtime(game.playtime_forever),
        recent: steam::format_playtime(game.playtime_2weeks),
        raw_playtime_minutes: game.playtime_forever,
        store_url: Some(format!("https://store.steampowered.com/app/{}", game.appid)),
    }
}

fn template(state: &AppState) -> GamesTemplate {
    let data = state.steam.clone();
    // Heatmap hidden for now — restore when wanted (computation stays in `steam`).
    let heatmap: Option<steam::Heatmap> = None;

    let recent = data
        .as_ref()
        .map(|d| {
            let mut games: Vec<_> = d
                .games
                .iter()
                .filter(|game| game.playtime_2weeks > 0)
                .collect();
            games.sort_by(|a, b| b.playtime_2weeks.cmp(&a.playtime_2weeks));
            games.into_iter().take(6).map(view).collect()
        })
        .unwrap_or_default();

    let mut all_top: Vec<GameView> = data
        .as_ref()
        .map(|d| d.games.iter().map(view).collect())
        .unwrap_or_default();

    for sg in &state.config.static_games {
        all_top.push(GameView {
            appid: 0,
            name: sg.name.clone(),
            icon_url: sg.icon_url.clone(),
            playtime: format!("{} hrs", sg.playtime_hours),
            recent: String::new(),
            raw_playtime_minutes: (sg.playtime_hours * 60) as u64,
            store_url: sg.store_url.clone(),
        });
    }
    all_top.sort_by(|a, b| b.raw_playtime_minutes.cmp(&a.raw_playtime_minutes));
    let top_games: Vec<GameView> = all_top.into_iter().take(12).collect();

    let mut progress: Vec<GameProgress> = data
        .as_ref()
        .map(|d| {
            d.achievements
                .iter()
                .filter(|game| !game.achievements.is_empty())
                .map(|game| {
                    let total = game.achievements.len();
                    let unlocked = game
                        .achievements
                        .iter()
                        .filter(|a| a.achieved == 1)
                        .count();
                    let icon_url = if game.img_icon_url.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "https://media.steampowered.com/steamcommunity/public/images/apps/{}/{}.jpg",
                            game.appid, game.img_icon_url
                        )
                    };
                    GameProgress {
                        name: game.name.clone(),
                        icon_url,
                        unlocked,
                        total,
                        percent: if total > 0 { unlocked * 100 / total } else { 0 },
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    for sg in &state.config.static_games {
        let percent = if sg.total_achievements > 0 {
            sg.unlocked_achievements * 100 / sg.total_achievements
        } else {
            0
        };
        progress.push(GameProgress {
            name: sg.name.clone(),
            icon_url: sg.icon_url.clone(),
            unlocked: sg.unlocked_achievements,
            total: sg.total_achievements,
            percent,
        });
    }
    progress.sort_by(|a, b| b.percent.cmp(&a.percent).then(a.name.cmp(&b.name)));
    // Steam-derived and curated rows are filtered by the same rule, after the
    // sort, so the list never mixes thresholds.
    progress.retain(keep_progress_row);

    GamesTemplate {
        total_playtime: data
            .as_ref()
            .map(|d| steam::format_total_playtime(d.total_playtime))
            .unwrap_or_default(),
        data,
        heatmap,
        recent,
        top_games,
        progress,
        meta_title: "Games — Yafi Alhakim".into(),
        meta_description: "Steam library, playtime, and achievement progress."
            .into(),
        meta_url: format!("{}/games", state.config.base_url),
        meta_type: "website".into(),
        og_image: crate::models::og_image(&state.config.base_url),
    }
}

pub async fn games(State(state): State<AppState>) -> impl IntoResponse {
    template(&state)
}

pub fn games_html(state: &AppState) -> String {
    template(state)
        .render()
        .expect("failed to render games page")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, percent: usize) -> GameProgress {
        GameProgress {
            name: name.into(),
            icon_url: String::new(),
            unlocked: percent,
            total: 100,
            percent,
        }
    }

    #[test]
    fn progress_list_drops_rows_below_25_percent() {
        // 25 is the boundary and survives; 24 does not.
        let mut rows = vec![
            row("complete", 100),
            row("edge", 25),
            row("just-under", 24),
            row("barely-started", 1),
            row("untouched", 0),
        ];

        rows.retain(keep_progress_row);

        let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(names, ["complete", "edge"]);
    }
}
