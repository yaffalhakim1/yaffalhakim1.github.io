//! Steam profile data: a committed snapshot plus an opt-in refresh from the
//! Steam Web API.
//!
//! The site is exported to static HTML, so nothing may call Steam from the
//! browser (that would leak the API key). Data is fetched at build time into
//! `content/steam.json` and read back from disk on every render.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn snapshot_path() -> PathBuf {
    PathBuf::from("content/steam.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub personaname: String,
    pub avatarfull: String,
    pub profileurl: String,
    #[serde(default)]
    pub realname: Option<String>,
    #[serde(default)]
    pub loccountrycode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub appid: u64,
    pub name: String,
    pub playtime_forever: u64,
    #[serde(default)]
    pub playtime_2weeks: u64,
    #[serde(default)]
    pub img_icon_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: u8,
    #[serde(default)]
    pub unlocktime: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAchievements {
    pub appid: u64,
    pub name: String,
    #[serde(default)]
    pub img_icon_url: String,
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamData {
    pub player: Option<Player>,
    pub games: Vec<Game>,
    pub achievements: Vec<GameAchievements>,
    pub total_playtime: u64,
    pub total_games: usize,
    pub unlocked_achievements: usize,
    pub fetched_at: String,
}

pub fn load() -> Option<SteamData> {
    let raw = std::fs::read_to_string(snapshot_path()).ok()?;
    serde_json::from_str(&raw).ok()
}
// ————— Heatmap —————

#[derive(Debug, Clone)]
pub struct HeatCell {
    pub date: NaiveDate,
    pub count: usize,
    pub level: u8,
    pub in_range: bool,
}

#[derive(Debug, Clone)]
pub struct HeatWeek {
    pub month: Option<String>,
    pub cells: Vec<HeatCell>,
}

#[derive(Debug, Clone)]
pub struct Heatmap {
    pub weeks: Vec<HeatWeek>,
    pub total: usize,
    pub busiest: usize,
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Count achievement unlocks per calendar day (UTC).
///
/// Steam only reports cumulative playtime, so unlock dates are the one real
/// per-day signal available for a contribution-style graph.
fn unlocks_by_day(data: &SteamData) -> BTreeMap<NaiveDate, usize> {
    let mut days: BTreeMap<NaiveDate, usize> = BTreeMap::new();
    for game in &data.achievements {
        for achievement in &game.achievements {
            if achievement.achieved != 1 || achievement.unlocktime <= 0 {
                continue;
            }
            if let Some(datetime) = chrono::DateTime::from_timestamp(achievement.unlocktime, 0) {
                *days.entry(datetime.date_naive()).or_insert(0) += 1;
            }
        }
    }
    days
}

fn level_for(count: usize) -> u8 {
    match count {
        0 => 0,
        1 => 1,
        2..=3 => 2,
        4..=9 => 3,
        _ => 4,
    }
}

pub fn heatmap(data: &SteamData) -> Option<Heatmap> {
    let days = unlocks_by_day(data);
    let first = *days.keys().next()?;
    let last = *days.keys().next_back()?;

    // Align to Sunday-started weeks, the way GitHub renders its graph.
    let start = first - Duration::days(first.weekday().num_days_from_sunday() as i64);
    let end = last + Duration::days(6 - last.weekday().num_days_from_sunday() as i64);

    let mut weeks: Vec<HeatWeek> = Vec::new();
    let mut cursor = start;
    let mut previous_month: Option<u32> = None;
    let mut weeks_since_label = 0usize;

    while cursor <= end {
        let mut cells = Vec::with_capacity(7);
        for offset in 0..7 {
            let date = cursor + Duration::days(offset);
            let count = days.get(&date).copied().unwrap_or(0);
            cells.push(HeatCell {
                date,
                count,
                level: level_for(count),
                in_range: date >= first && date <= last,
            });
        }

        // Label the first week of a month, but only when the previous label is
        // far enough back that the text cannot collide with it.
        let month = cells[0].date.month();
        let month_changed = previous_month.is_some_and(|previous| previous != month);
        let label = if previous_month.is_none() || (month_changed && weeks_since_label >= 3) {
            weeks_since_label = 0;
            Some(MONTHS[(month - 1) as usize].to_string())
        } else {
            weeks_since_label += 1;
            None
        };
        previous_month = Some(month);

        weeks.push(HeatWeek { month: label, cells });
        cursor += Duration::days(7);
    }

    Some(Heatmap {
        weeks,
        total: days.values().sum(),
        busiest: days.values().copied().max().unwrap_or(0),
    })
}
// ————— Refresh from the Steam Web API —————

fn api(path: &str, params: &[(&str, String)]) -> String {
    let mut url = format!("https://api.steampowered.com/{path}?");
    for (key, value) in params {
        url.push_str(&format!("{key}={}&", urlencode(value)));
    }
    url
}

fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn get_json(url: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("client: {e}"))?;
    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("request: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("steam responded {}", response.status()));
    }
    response.json().map_err(|e| format!("json: {e}"))
}

fn string_field(value: &serde_json::Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_string()
}

fn opt_string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value[key].as_str().map(str::to_string)
}

pub fn refresh() -> Result<SteamData, String> {
    let key =
        std::env::var("STEAM_API_KEY").map_err(|_| "STEAM_API_KEY is not set".to_string())?;
    let steam_id = std::env::var("STEAM_ID").map_err(|_| "STEAM_ID is not set".to_string())?;

    let profile = get_json(&api(
        "ISteamUser/GetPlayerSummaries/v2/",
        &[("key", key.clone()), ("steamids", steam_id.clone())],
    ))?;
    let player = profile["response"]["players"]
        .as_array()
        .and_then(|players| players.first())
        .map(|p| Player {
            personaname: string_field(p, "personaname"),
            avatarfull: string_field(p, "avatarfull"),
            profileurl: string_field(p, "profileurl"),
            realname: opt_string_field(p, "realname"),
            loccountrycode: opt_string_field(p, "loccountrycode"),
        });

    let owned = get_json(&api(
        "IPlayerService/GetOwnedGames/v1/",
        &[
            ("key", key.clone()),
            ("steamid", steam_id.clone()),
            ("include_appinfo", "true".to_string()),
            ("include_played_free_games", "true".to_string()),
        ],
    ))?;

    let mut games: Vec<Game> = owned["response"]["games"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|g| Game {
                    appid: g["appid"].as_u64().unwrap_or(0),
                    name: string_field(g, "name"),
                    playtime_forever: g["playtime_forever"].as_u64().unwrap_or(0),
                    playtime_2weeks: g["playtime_2weeks"].as_u64().unwrap_or(0),
                    img_icon_url: string_field(g, "img_icon_url"),
                })
                .collect()
        })
        .unwrap_or_default();

    games.retain(|game| game.appid != 0);
    games.sort_by(|a, b| b.playtime_forever.cmp(&a.playtime_forever));

    let total_playtime = games.iter().map(|game| game.playtime_forever).sum();
    let total_games = games.len();

    let mut achievements = Vec::new();
    let mut unlocked_achievements = 0;
    for game in games.iter().take(12) {
        let url = api(
            "ISteamUserStats/GetPlayerAchievements/v1/",
            &[
                ("key", key.clone()),
                ("steamid", steam_id.clone()),
                ("appid", game.appid.to_string()),
                ("l", "en".to_string()),
            ],
        );
        // Games without community stats return an error payload; skip them.
        let Ok(payload) = get_json(&url) else { continue };
        let Some(list) = payload["playerstats"]["achievements"].as_array() else {
            continue;
        };

        let mut parsed = Vec::with_capacity(list.len());
        for entry in list {
            let achieved = entry["achieved"].as_u64().unwrap_or(0) as u8;
            if achieved == 1 {
                unlocked_achievements += 1;
            }
            parsed.push(Achievement {
                apiname: string_field(entry, "apiname"),
                achieved,
                unlocktime: entry["unlocktime"].as_i64().unwrap_or(0),
                name: opt_string_field(entry, "name").unwrap_or_default(),
                description: opt_string_field(entry, "description").unwrap_or_default(),
                icon: opt_string_field(entry, "icon").unwrap_or_default(),
            });
        }

        achievements.push(GameAchievements {
            appid: game.appid,
            name: game.name.clone(),
            img_icon_url: game.img_icon_url.clone(),
            achievements: parsed,
        });
    }

    Ok(SteamData {
        player,
        games,
        achievements,
        total_playtime,
        total_games,
        unlocked_achievements,
        fetched_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    })
}

pub fn write_snapshot(data: &SteamData) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    let path = snapshot_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, json + "\n").map_err(|e| e.to_string())
}

pub fn format_playtime(minutes: u64) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours == 0 {
        return format!("{mins}min");
    }
    if mins == 0 {
        return format!("{hours}h");
    }
    format!("{hours}h {mins}min")
}

pub fn format_total_playtime(minutes: u64) -> String {
    format!("{}h", minutes / 60)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn achievement(unlocktime: i64, achieved: u8) -> Achievement {
        Achievement {
            apiname: "a".into(),
            achieved,
            unlocktime,
            name: String::new(),
            description: String::new(),
            icon: String::new(),
        }
    }

    fn data_with(unlocks: &[(&str, i64)]) -> SteamData {
        SteamData {
            player: None,
            games: Vec::new(),
            achievements: vec![GameAchievements {
                appid: 1,
                name: "Game".into(),
                img_icon_url: String::new(),
                achievements: unlocks
                    .iter()
                    .map(|(_, time)| achievement(*time, 1))
                    .collect(),
            }],
            total_playtime: 0,
            total_games: 0,
            unlocked_achievements: unlocks.len(),
            fetched_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn days(date: &str) -> i64 {
        NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    }

    #[test]
    fn heatmap_counts_unlocks_per_day() {
        // Two unlocks on the same day, one on another.
        let data = data_with(&[
            ("a", days("2025-03-04")),
            ("b", days("2025-03-04")),
            ("c", days("2025-03-06")),
        ]);
        let heat = heatmap(&data).expect("heatmap");

        assert_eq!(heat.total, 3);
        assert_eq!(heat.busiest, 2);

        let counts: Vec<(NaiveDate, usize)> = heat
            .weeks
            .iter()
            .flat_map(|week| week.cells.iter())
            .filter(|cell| cell.count > 0)
            .map(|cell| (cell.date, cell.count))
            .collect();
        assert_eq!(counts.len(), 2);
        assert_eq!(counts[0].1, 2);
        assert_eq!(counts[1].1, 1);
    }

    #[test]
    fn heatmap_ignores_locked_and_missing_timestamps() {
        // `unlocktime == 0` means the achievement has never been unlocked.
        let locked = data_with(&[("a", 0)]);
        assert!(heatmap(&locked).is_none());

        let mut data = data_with(&[("a", days("2025-03-04"))]);
        data.achievements[0].achievements[0].achieved = 0;
        assert!(heatmap(&data).is_none());
    }

    #[test]
    fn heatmap_weeks_align_to_sunday_and_cover_every_day() {
        // 2025-03-04 is a Tuesday, 2025-03-06 a Thursday.
        let data = data_with(&[("a", days("2025-03-04")), ("b", days("2025-03-06"))]);
        let heat = heatmap(&data).expect("heatmap");

        // One week is enough for two days in the same Sunday-started week.
        assert_eq!(heat.weeks.len(), 1);
        let week = &heat.weeks[0];
        assert_eq!(week.cells.len(), 7);
        assert_eq!(week.cells[0].date.weekday(), chrono::Weekday::Sun);
        // Padding cells fall outside the data range and get no intensity.
        assert!(!week.cells[0].in_range);
        assert_eq!(week.cells[0].level, 0);
        // The span runs 03-04..03-06, so three days are in range while only
        // the two with unlocks carry intensity.
        let in_range: Vec<NaiveDate> = week
            .cells
            .iter()
            .filter(|c| c.in_range)
            .map(|c| c.date)
            .collect();
        assert_eq!(in_range.len(), 3);
        assert_eq!(in_range[0].to_string(), "2025-03-04");
        assert_eq!(in_range[2].to_string(), "2025-03-06");
        assert_eq!(week.cells.iter().filter(|c| c.count > 0).count(), 2);
        assert_eq!(
            week.cells.iter().filter(|c| c.count == 0).count(),
            5,
            "empty days still render as level 0"
        );
    }

    #[test]
    fn heatmap_spans_multiple_weeks_and_labels_months() {
        let data = data_with(&[("a", days("2025-03-04")), ("b", days("2025-04-20"))]);
        let heat = heatmap(&data).expect("heatmap");

        assert!(heat.weeks.len() >= 7);
        // First cell of every week may carry a month label; at least one must.
        assert!(heat.weeks.iter().any(|week| week.month.is_some()));
        assert!(
            heat.weeks
                .iter()
                .flat_map(|week| week.cells.iter())
                .all(|cell| cell.level <= 4)
        );
    }

    #[test]
    fn level_thresholds_are_monotonic_and_bounded() {
        assert_eq!(level_for(0), 0);
        assert_eq!(level_for(1), 1);
        assert_eq!(level_for(2), 2);
        assert_eq!(level_for(3), 2);
        assert_eq!(level_for(4), 3);
        assert_eq!(level_for(9), 3);
        assert_eq!(level_for(10), 4);
        assert_eq!(level_for(999), 4);
    }

    #[test]
    fn playtime_formatting_handles_hours_and_minutes() {
        assert_eq!(format_playtime(0), "0min");
        assert_eq!(format_playtime(59), "59min");
        assert_eq!(format_playtime(60), "1h");
        assert_eq!(format_playtime(90), "1h 30min");
        assert_eq!(format_total_playtime(90), "1h");
    }
}