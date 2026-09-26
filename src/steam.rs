//! Steam profile data: a committed snapshot plus an opt-in refresh from the
//! Steam Web API.
//!
//! The site is exported to static HTML, so nothing may call Steam from the
//! browser (that would leak the API key). Data is fetched at build time into
//! `content/steam.json` and read back from disk on every render.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn snapshot_path() -> PathBuf {
    PathBuf::from("content/steam.json")
}

/// Overlay extracted from the local Steam cache (see `--import-local`).
///
/// Optional: when the file is absent the page renders the API snapshot alone.
pub fn local_path() -> PathBuf {
    PathBuf::from("content/steam-local.json")
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

/// One unlock recorded by the local Steam cache.
///
/// The cache keeps no achievement schema and no icon, only the apiname, the
/// display strings, and the unlock time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAchievement {
    pub apiname: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub unlocktime: i64,
}

/// Per-game overlay extracted from the local Steam cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalGame {
    pub appid: u64,
    #[serde(default)]
    pub name: String,
    /// Steam icon hash for the game, when the snapshot cannot supply one.
    ///
    /// Local-only games (the snapshot never fetched them) have no icon in the
    /// API data, so the hash is carried here by hand and the merge builds the
    /// same URL the snapshot path does.
    #[serde(default)]
    pub img_icon_url: String,
    /// How many achievements the game defines, unlocked or not.
    #[serde(default)]
    pub total: usize,
    #[serde(default)]
    pub achievements: Vec<LocalAchievement>,
}

/// The committed local-cache overlay (`content/steam-local.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalOverlay {
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub games: Vec<LocalGame>,
}

pub fn load() -> Option<SteamData> {
    let raw = std::fs::read_to_string(snapshot_path()).ok()?;
    let snapshot: SteamData = serde_json::from_str(&raw).ok()?;
    // A missing or malformed overlay is not an error: the page falls back to
    // the API snapshot exactly as it rendered before the overlay existed.
    Some(match read_local() {
        Some(overlay) => merge(snapshot, overlay),
        None => snapshot,
    })
}

// ————— Local cache overlay —————

fn read_local() -> Option<LocalOverlay> {
    let raw = std::fs::read_to_string(local_path()).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Icon hashes already committed to the overlay, keyed by appid.
///
/// The raw dump carries no icon data, so a re-import would wipe the hashes
/// added to `content/steam-local.json` by hand; the caller reads them back here
/// and reapplies them. A missing or malformed file yields no hashes, which is
/// exactly right for a first import.
pub fn existing_icons() -> BTreeMap<u64, String> {
    read_local()
        .map(|overlay| {
            overlay
                .games
                .into_iter()
                .filter(|game| !game.img_icon_url.is_empty())
                .map(|game| (game.appid, game.img_icon_url))
                .collect()
        })
        .unwrap_or_default()
}

/// Convert the raw `steam-everything.json` dump into the committed overlay.
///
/// The dump is a read-only extraction of the local Steam cache. Its `unlocked`
/// counter is a cache of `achievement_progress.json` and can lag behind the
/// per-app progress files — Khazan and Mortal Shell II both report 0 unlocks
/// while their `detail` lists hold real, timestamped unlocks — so the achieved
/// rows are counted from `detail` alone and the untrusted counter is dropped.
pub fn import_local(path: &Path) -> Result<LocalOverlay, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let dump: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;

    let rows = dump["rows"]
        .as_array()
        .ok_or_else(|| format!("{} has no `rows` array", path.display()))?;

    let mut games = Vec::new();
    for row in rows {
        // The dump keys apps by string because it was extracted from Steam's
        // JSON caches; the overlay uses real integers.
        let Some(appid) = row["appid"].as_str().and_then(|id| id.parse::<u64>().ok()) else {
            continue;
        };
        let Some(detail) = row["detail"].as_array() else {
            continue;
        };
        let achievements: Vec<LocalAchievement> = detail
            .iter()
            .filter_map(|entry| {
                let apiname = entry["api"].as_str().unwrap_or_default();
                let unlocktime = entry["at"].as_i64().unwrap_or(0);
                // A row without both an apiname and a timestamp cannot be
                // merged onto the API schema, so it is not worth carrying.
                if apiname.is_empty() || unlocktime <= 0 {
                    return None;
                }
                Some(LocalAchievement {
                    apiname: apiname.to_string(),
                    name: entry["name"].as_str().unwrap_or_default().to_string(),
                    description: entry["desc"].as_str().unwrap_or_default().to_string(),
                    unlocktime,
                })
            })
            .collect();
        if achievements.is_empty() {
            continue;
        }
        games.push(LocalGame {
            appid,
            name: row["name"].as_str().unwrap_or_default().to_string(),
            img_icon_url: String::new(),
            total: row["total"].as_u64().unwrap_or(0) as usize,
            achievements,
        });
    }

    games.sort_by_key(|game| game.appid);
    Ok(LocalOverlay {
        account: dump["account"].as_str().unwrap_or_default().to_string(),
        games,
    })
}

pub fn write_local_overlay(overlay: &LocalOverlay) -> Result<(), String> {
    let json = serde_json::to_string_pretty(overlay).map_err(|e| e.to_string())?;
    let path = local_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, json + "\n").map_err(|e| e.to_string())
}

/// Merge the local-cache overlay into an API snapshot.
///
/// The snapshot stays the base: it carries the owned-games list, the playtime,
/// and the full per-game achievement schema with icons. The overlay only adds
/// what Steam never returned — unlock timestamps for games the snapshot
/// skipped, and whole sets for games the snapshot never fetched at all.
pub fn merge(mut snapshot: SteamData, overlay: LocalOverlay) -> SteamData {
    // Titles and icons for games the snapshot knows but never fetched
    // achievements for; the local cache carries a name only in its own file,
    // and an icon only when one was added by hand.
    let owned: BTreeMap<u64, (String, String)> = snapshot
        .games
        .iter()
        .map(|game| (game.appid, (game.name.clone(), game.img_icon_url.clone())))
        .collect();

    let mut pending: BTreeMap<u64, LocalGame> = overlay
        .games
        .into_iter()
        .map(|game| (game.appid, game))
        .collect();

    for game in &mut snapshot.achievements {
        let Some(local) = pending.remove(&game.appid) else {
            continue;
        };

        // Local progress can be ahead of what Steam has uploaded, so anything
        // the local cache has an unlock time for is marked achieved even if the
        // snapshot still reports it locked.
        let unlocks: BTreeMap<&str, i64> = local
            .achievements
            .iter()
            .map(|achievement| (achievement.apiname.as_str(), achievement.unlocktime))
            .collect();

        // Unlocks the snapshot's schema does not list at all would otherwise be
        // dropped silently, which would understate the profile.
        let unknown: Vec<&LocalAchievement> = local
            .achievements
            .iter()
            .filter(|local| {
                !game
                    .achievements
                    .iter()
                    .any(|api| api.apiname == local.apiname)
            })
            .collect();

        for achievement in &mut game.achievements {
            if let Some(&unlocktime) = unlocks.get(achievement.apiname.as_str()) {
                achievement.achieved = 1;
                achievement.unlocktime = unlocktime;
            }
        }

        game.achievements
            .extend(unknown.into_iter().map(|local| Achievement {
                apiname: local.apiname.clone(),
                achieved: 1,
                unlocktime: local.unlocktime,
                name: local.name.clone(),
                description: local.description.clone(),
                icon: String::new(),
            }));
    }

    for (appid, game) in pending {
        snapshot
            .achievements
            .push(synthesize(game, owned.get(&appid)));
    }

    // Both inputs count unlocks in their own way; the merged view is the only
    // honest source for the profile chip.
    snapshot.unlocked_achievements = snapshot
        .achievements
        .iter()
        .flat_map(|game| game.achievements.iter())
        .filter(|achievement| achievement.achieved == 1)
        .count();
    snapshot
}

/// Build the achievement set for a game the snapshot never fetched.
///
/// The local cache only records unlocks, so the achieved entries are padded
/// with locked placeholders up to the schema total: the progress list needs an
/// honest denominator and renders counts, not per-achievement icons.
fn synthesize(game: LocalGame, owned: Option<&(String, String)>) -> GameAchievements {
    let total = game.total.max(game.achievements.len());
    let mut achievements: Vec<Achievement> = game
        .achievements
        .into_iter()
        .map(|local| Achievement {
            apiname: local.apiname,
            achieved: 1,
            unlocktime: local.unlocktime,
            name: local.name,
            description: local.description,
            icon: String::new(),
        })
        .collect();
    achievements.resize_with(total, || Achievement {
        apiname: String::new(),
        achieved: 0,
        unlocktime: 0,
        name: String::new(),
        description: String::new(),
        icon: String::new(),
    });

    let name = match owned {
        Some((name, _)) => name.clone(),
        None => game.name.clone(),
    };
    // The snapshot's icon wins; a game it never listed falls back to the hash
    // carried in the overlay, which the dump itself never provides.
    let img_icon_url = owned
        .map(|(_, icon)| icon.as_str())
        .filter(|icon| !icon.is_empty())
        .unwrap_or(game.img_icon_url.as_str())
        .to_string();
    GameAchievements {
        appid: game.appid,
        name,
        img_icon_url,
        achievements,
    }
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
    // ————— Local overlay merge —————

    fn ach(apiname: &str, achieved: u8, unlocktime: i64) -> Achievement {
        Achievement {
            apiname: apiname.into(),
            achieved,
            unlocktime,
            name: format!("{apiname} name"),
            description: String::new(),
            icon: String::new(),
        }
    }

    fn local_game(appid: u64, name: &str, total: usize, unlocks: &[(&str, i64)]) -> LocalGame {
        LocalGame {
            appid,
            name: name.into(),
            img_icon_url: String::new(),
            total,
            achievements: unlocks
                .iter()
                .map(|(apiname, at)| LocalAchievement {
                    apiname: (*apiname).into(),
                    name: format!("{apiname} name"),
                    description: String::new(),
                    unlocktime: *at,
                })
                .collect(),
        }
    }

    fn overlay(games: Vec<LocalGame>) -> LocalOverlay {
        LocalOverlay {
            account: "384463082".into(),
            games,
        }
    }

    fn snapshot(games: Vec<Game>, achievements: Vec<GameAchievements>) -> SteamData {
        SteamData {
            player: None,
            games,
            achievements,
            total_playtime: 0,
            total_games: 0,
            unlocked_achievements: 0,
            fetched_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn merge_stamps_local_unlock_times_onto_the_snapshot_schema() {
        // The snapshot has the full schema (icons, locked entries) but Steam
        // never returned unlock times for it; the local cache has the times.
        let base = snapshot(
            Vec::new(),
            vec![GameAchievements {
                appid: 292030,
                name: "The Witcher 3".into(),
                img_icon_url: "icon".into(),
                achievements: vec![ach("won", 0, 0), ach("locked", 0, 0)],
            }],
        );
        let merged = merge(
            base,
            overlay(vec![local_game(
                292030,
                "The Witcher 3",
                2,
                &[("won", 1_700_000_000)],
            )]),
        );

        let game = &merged.achievements[0];
        assert_eq!(
            game.achievements.len(),
            2,
            "schema is preserved, not replaced"
        );
        assert_eq!(game.achievements[0].achieved, 1);
        assert_eq!(game.achievements[0].unlocktime, 1_700_000_000);
        assert_eq!(
            game.achievements[0].icon,
            String::new(),
            "snapshot icon survives"
        );
        assert_eq!(game.achievements[1].achieved, 0);
        assert_eq!(game.achievements[1].unlocktime, 0);
        assert_eq!(merged.unlocked_achievements, 1);
    }

    #[test]
    fn merge_synthesizes_local_only_games_padded_to_total() {
        // Khazan-shaped: the snapshot owns the game (so it has a name and icon)
        // but never fetched its achievements.
        let base = snapshot(
            vec![Game {
                appid: 2680010,
                name: "The First Berserker: Khazan".into(),
                playtime_forever: 1461,
                playtime_2weeks: 0,
                img_icon_url: "khazan".into(),
            }],
            Vec::new(),
        );
        let merged = merge(
            base,
            overlay(vec![
                local_game(2680010, "Khazan", 57, &[("Boss_01", 100), ("Boss_02", 200)]),
                // Not in the owned list at all: the local title is all we have.
                LocalGame {
                    img_icon_url: "6b814f92e25cec848c9729ce26c8f39afcc6e5f7".into(),
                    ..local_game(730, "Counter-Strike 2", 1, &[("win", 300)])
                },
            ]),
        );

        let khazan = merged
            .achievements
            .iter()
            .find(|game| game.appid == 2680010)
            .expect("khazan synthesized");
        assert_eq!(
            khazan.name, "The First Berserker: Khazan",
            "owned title wins"
        );
        assert_eq!(khazan.img_icon_url, "khazan", "owned icon is carried over");
        assert_eq!(khazan.achievements.len(), 57, "padded to the schema total");
        assert_eq!(
            khazan
                .achievements
                .iter()
                .filter(|a| a.achieved == 1)
                .count(),
            2
        );
        assert_eq!(khazan.achievements[0].apiname, "Boss_01");
        assert_eq!(khazan.achievements[0].unlocktime, 100);
        assert_eq!(khazan.achievements[2].achieved, 0, "padding is locked");

        let cs2 = merged
            .achievements
            .iter()
            .find(|game| game.appid == 730)
            .expect("cs2 synthesized");
        assert_eq!(cs2.name, "Counter-Strike 2");
        // The snapshot never listed this appid, so the hand-added overlay hash
        // is the only icon source and must survive the synthesis.
        assert_eq!(cs2.img_icon_url, "6b814f92e25cec848c9729ce26c8f39afcc6e5f7");
        assert_eq!(cs2.achievements.len(), 1);

        assert_eq!(merged.unlocked_achievements, 3);
    }

    #[test]
    fn merge_keeps_local_unlocks_the_snapshot_schema_does_not_list() {
        // Dropping these would silently understate the profile, so an unknown
        // apiname is appended rather than ignored.
        let base = snapshot(
            Vec::new(),
            vec![GameAchievements {
                appid: 1,
                name: "Game".into(),
                img_icon_url: String::new(),
                achievements: vec![ach("known", 0, 0)],
            }],
        );
        let merged = merge(
            base,
            overlay(vec![local_game(
                1,
                "Game",
                2,
                &[("known", 10), ("extra", 20)],
            )]),
        );

        let game = &merged.achievements[0];
        assert_eq!(game.achievements.len(), 2);
        assert_eq!(game.achievements[1].apiname, "extra");
        assert_eq!(game.achievements[1].achieved, 1);
        assert_eq!(game.achievements[1].unlocktime, 20);
        assert_eq!(merged.unlocked_achievements, 2);
    }

    #[test]
    fn merge_total_unlocks_matches_the_honest_local_total() {
        // The real numbers this merge exists for: the API snapshot knows 374
        // unlocks, the local cache adds 66 the snapshot never fetched, and the
        // honest merged total is 440.
        let mut base_achievements = Vec::new();
        for index in 0..374 {
            base_achievements.push(ach(&format!("api{index}"), 1, 1_700_000_000 + index as i64));
        }
        let base = snapshot(
            Vec::new(),
            vec![GameAchievements {
                appid: 292030,
                name: "Overlap".into(),
                img_icon_url: String::new(),
                achievements: base_achievements,
            }],
        );

        // 100 of the local unlocks are already counted by the snapshot; the
        // other 66 come from a game the snapshot never fetched.
        let mut local_only = Vec::new();
        for index in 0..66 {
            local_only.push((format!("local{index}"), 1_780_000_000 + index as i64));
        }
        let local_only: Vec<(&str, i64)> = local_only
            .iter()
            .map(|(apiname, at)| (apiname.as_str(), *at))
            .collect();

        let merged = merge(
            base,
            overlay(vec![
                local_game(292030, "Overlap", 78, &[("api0", 10), ("api1", 11)]),
                local_game(2680010, "Khazan", 57, &local_only),
            ]),
        );

        assert_eq!(merged.unlocked_achievements, 440);
    }

    #[test]
    fn merge_reports_khazan_progress_despite_a_zero_cached_count() {
        // Khazan is the stale-count case end to end: the local cache counts 0
        // unlocks, its detail list holds 37, and the game is absent from the
        // snapshot's achievement sets. The merged row must read 37/57 (64%).
        let unlocks: Vec<(String, i64)> = (0..37)
            .map(|index| (format!("Boss_{index:02}"), 1_778_000_000 + index as i64))
            .collect();
        let unlocks: Vec<(&str, i64)> = unlocks
            .iter()
            .map(|(apiname, at)| (apiname.as_str(), *at))
            .collect();

        let merged = merge(
            snapshot(Vec::new(), Vec::new()),
            overlay(vec![local_game(
                2680010,
                "The First Berserker: Khazan",
                57,
                &unlocks,
            )]),
        );

        let khazan = &merged.achievements[0];
        let unlocked = khazan
            .achievements
            .iter()
            .filter(|a| a.achieved == 1)
            .count();
        assert_eq!(unlocked, 37);
        assert_eq!(khazan.achievements.len(), 57);
        assert_eq!(unlocked * 100 / khazan.achievements.len(), 64);
    }

    #[test]
    fn import_local_counts_detail_not_the_stale_unlocked_field() {
        // Khazan and Mortal Shell II both report `unlocked: 0` while their
        // `detail` lists hold real timestamped unlocks, so the converter counts
        // the rows instead of trusting the cached counter.
        let json = r#"{"account":"384463082","rows":[
            {"appid":"2680010","name":"The First Berserker: Khazan","unlocked":0,"total":57,
             "detail":[{"api":"Boss_01","name":"Ruler","desc":"d","at":1778654434,"at_str":"x"},
                       {"api":"Boss_02","name":"Phantom","desc":"d","at":1778686417,"at_str":"x"}]},
            {"appid":"7","name":null,"unlocked":0,"total":0,"detail":[]},
            {"appid":"not-a-number","name":"junk","unlocked":0,"total":0,"detail":[{"api":"a","at":5}]}]}"#;
        let path = std::env::temp_dir().join(format!("steam-local-{}.json", std::process::id()));
        std::fs::write(&path, json).unwrap();
        let imported = import_local(&path).expect("import");
        std::fs::remove_file(&path).ok();

        assert_eq!(imported.account, "384463082");
        // Rows without detail (and unparsable appids) are dropped, and the
        // string appid becomes a number.
        assert_eq!(imported.games.len(), 1);
        assert_eq!(imported.games[0].appid, 2680010);
        assert_eq!(imported.games[0].total, 57);
        assert_eq!(imported.games[0].achievements.len(), 2);
        assert_eq!(imported.games[0].achievements[0].unlocktime, 1778654434);
    }
}