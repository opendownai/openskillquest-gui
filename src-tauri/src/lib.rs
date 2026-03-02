use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub slug: String,
    pub desc: String,
    pub author: Option<String>,
    pub downloads: i64,
    pub stars: i64,
    pub category: Option<String>,
    pub install_cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub level: i32,
    pub exp: i64,
    pub max_exp: i64,
    pub installed: Vec<String>,
    pub daily_quests: i32,
    pub last_daily: String,
    pub achievements: Vec<String>,
    pub player_name: String,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            level: 1,
            exp: 0,
            max_exp: 500,
            installed: vec![],
            daily_quests: 0,
            last_daily: String::new(),
            achievements: vec![],
            player_name: "Agent Hunter".to_string(),
        }
    }
}

pub struct AppState {
    pub game: Mutex<GameState>,
    pub skills: Mutex<Vec<Skill>>,
}

fn get_save_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("OpenSkillQuest");
    fs::create_dir_all(&path).ok();
    path.push("save.json");
    path
}

fn load_game() -> GameState {
    let path = get_save_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<GameState>(&data) {
                return state;
            }
        }
    }
    GameState::default()
}

fn save_game(state: &GameState) {
    let path = get_save_path();
    if let Ok(data) = serde_json::to_string_pretty(state) {
        fs::write(path, data).ok();
    }
}

fn check_daily(state: &mut GameState) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if state.last_daily != today {
        state.daily_quests = 0;
        state.last_daily = today;
    }
}

fn calc_exp(downloads: i64, stars: i64) -> i64 {
    std::cmp::max(10, downloads / 1000 + stars / 100)
}

fn calc_rarity(downloads: i64, stars: i64) -> String {
    let score = downloads / 1000 + stars / 50;
    if score >= 500 {
        "Legendary".to_string()
    } else if score >= 200 {
        "Epic".to_string()
    } else if score >= 100 {
        "Rare".to_string()
    } else if score >= 50 {
        "Uncommon".to_string()
    } else {
        "Common".to_string()
    }
}

#[tauri::command]
fn get_game_state(state: State<AppState>) -> GameState {
    let mut game = state.game.lock().unwrap();
    check_daily(&mut game);
    save_game(&game);
    game.clone()
}

#[tauri::command]
async fn fetch_skills(state: State<'_, AppState>) -> Result<Vec<Skill>, String> {
    let resp = reqwest::get("https://opendown.ai/api/top-skills")
        .await
        .map_err(|e| e.to_string())?;
    
    let skills: Vec<Skill> = resp.json().await.map_err(|e| e.to_string())?;
    
    let filtered: Vec<Skill> = skills
        .into_iter()
        .filter(|s| !s.slug.starts_with("__"))
        .collect();
    
    let mut state_skills = state.skills.lock().unwrap();
    *state_skills = filtered.clone();
    
    Ok(filtered)
}

#[tauri::command]
fn get_skills(state: State<AppState>) -> Vec<Skill> {
    state.skills.lock().unwrap().clone()
}

#[tauri::command]
fn install_skill(skill_name: String, state: State<AppState>) -> Result<GameState, String> {
    let mut game = state.game.lock().unwrap();
    check_daily(&mut game);
    
    if game.installed.contains(&skill_name) {
        return Err("Skill already installed".to_string());
    }
    
    let skills = state.skills.lock().unwrap();
    let skill = skills
        .iter()
        .find(|s| s.name == skill_name || s.slug == skill_name)
        .ok_or("Skill not found")?;
    
    let exp = calc_exp(skill.downloads, skill.stars);
    let daily_limit = 3;
    let mut total_exp = exp;
    
    if game.daily_quests < daily_limit {
        game.daily_quests += 1;
        total_exp += 20;
    }
    
    game.exp += total_exp;
    while game.exp >= game.max_exp {
        game.exp -= game.max_exp;
        game.level += 1;
        game.max_exp = (game.max_exp as f64 * 1.5) as i64;
    }
    
    game.installed.push(skill_name.clone());
    save_game(&game);
    
    Ok(game.clone())
}

#[tauri::command]
fn complete_quest(skill_name: String, state: State<AppState>) -> Result<GameState, String> {
    let mut game = state.game.lock().unwrap();
    check_daily(&mut game);
    
    let skills = state.skills.lock().unwrap();
    let skill = skills
        .iter()
        .find(|s| s.name == skill_name || s.slug == skill_name)
        .ok_or("Skill not found")?;
    
    let exp = calc_exp(skill.downloads, skill.stars);
    let daily_limit = 3;
    let mut total_exp = exp;
    
    if game.daily_quests < daily_limit {
        game.daily_quests += 1;
        total_exp += 20;
    }
    
    game.exp += total_exp;
    while game.exp >= game.max_exp {
        game.exp -= game.max_exp;
        game.level += 1;
        game.max_exp = (game.max_exp as f64 * 1.5) as i64;
    }
    
    if !game.installed.contains(&skill_name) {
        game.installed.push(skill_name);
    }
    
    save_game(&game);
    Ok(game.clone())
}

#[tauri::command]
fn get_share_text(state: State<AppState>, player_name: Option<String>) -> String {
    let game = state.game.lock().unwrap();
    let build_str = game.installed.join(" | ");
    let name = player_name.unwrap_or_else(|| "Agent Hunter".to_string());
    format!(
        "🎮 My OpenSkillQuest Build\n━━━━━━━━━━━━━━━━━━━━━━━━━\n{} | Level {} | Total EXP: {}/{}\nSkills: {}\n━━━━━━━━━━━━━━━━━━━━━━━━━\n#OpenSkillQuest #ClawHub #RPG",
        name, game.level, game.exp, game.max_exp, build_str
    )
}

#[tauri::command]
fn get_skill_info(skill_name: String, state: State<AppState>) -> Result<serde_json::Value, String> {
    let skills = state.skills.lock().unwrap();
    let skill = skills
        .iter()
        .find(|s| s.name == skill_name || s.slug == skill_name)
        .ok_or("Skill not found")?;
    
    let exp = calc_exp(skill.downloads, skill.stars);
    let rarity = calc_rarity(skill.downloads, skill.stars);
    
    Ok(serde_json::json!({
        "name": skill.name,
        "slug": skill.slug,
        "desc": skill.desc,
        "downloads": skill.downloads,
        "stars": skill.stars,
        "exp": exp,
        "rarity": rarity,
        "install_cmd": skill.install_cmd,
        "category": skill.category,
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    
    let game = load_game();
    let skills = vec![];
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            game: Mutex::new(game),
            skills: Mutex::new(skills),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_game_state,
            fetch_skills,
            get_skills,
            install_skill,
            complete_quest,
            get_share_text,
            get_skill_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
