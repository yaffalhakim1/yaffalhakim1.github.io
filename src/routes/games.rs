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
    pub icon: String,
    pub playtime: String,
    pub recent: String,
}

pub struct GameProgress {
    pub appid: u64,
    pub name: String,
    pub icon: String,
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

fn view(game: &steam::Game) -> GameView {
    GameView {
        appid: game.appid,
        name: game.name.clone(),
        icon: game.img_icon_url.clone(),
        playtime: steam::format_playtime(game.playtime_forever),
        recent: steam::format_playtime(game.playtime_2weeks),
    }
}

fn template(state: &AppState) -> GamesTemplate {
    let data = state.steam.clone();
    let heatmap = data.as_ref().and_then(steam::heatmap);

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

    let top_games = data
        .as_ref()
        .map(|d| d.games.iter().take(9).map(view).collect())
        .unwrap_or_default();

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
                    GameProgress {
                        appid: game.appid,
                        name: game.name.clone(),
                        icon: game.img_icon_url.clone(),
                        unlocked,
                        total,
                        percent: if total > 0 { unlocked * 100 / total } else { 0 },
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    progress.sort_by(|a, b| b.percent.cmp(&a.percent).then(a.name.cmp(&b.name)));

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
        meta_description: "Steam library, playtime, and a heatmap of achievement unlocks."
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