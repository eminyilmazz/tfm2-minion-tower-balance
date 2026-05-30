use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use mod_api::*;

const MOD_ID: &str = "tfm2-minion-tower-balance";
const BALANCE_SAVE_VERSION: usize = 5;
const CONFIG_FILE_NAME: &str = "balance_config.txt";
const CONFIG_KEY: &str = "config";
const APPLIED_CONFIG_KEY: &str = "applied_config";
const MIN_PERCENT: usize = 1;
const MAX_PERCENT: usize = 500;
const LOG_FILE_NAME: &str = "tfm2-minion-tower-balance.log";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BalanceConfig {
    minion_hp_percent: usize,
    minion_damage_percent: usize,
    minion_growth_hp_percent: usize,
    minion_growth_damage_percent: usize,
    tower_hp_percent: usize,
    tower_damage_percent: usize,
    tower_projectile_speed_percent: usize,
    nexus_hp_percent: usize,
    morgard_minion_buff_percent: usize,
}

impl Default for BalanceConfig {
    fn default() -> Self {
        Self {
            minion_hp_percent: 90,
            minion_damage_percent: 80,
            minion_growth_hp_percent: 100,
            minion_growth_damage_percent: 100,
            tower_hp_percent: 120,
            tower_damage_percent: 110,
            tower_projectile_speed_percent: 110,
            nexus_hp_percent: 125,
            morgard_minion_buff_percent: 75,
        }
    }
}

impl BalanceConfig {
    fn legacy_default() -> Self {
        Self {
            minion_hp_percent: 105,
            minion_damage_percent: 90,
            minion_growth_hp_percent: 105,
            minion_growth_damage_percent: 95,
            tower_hp_percent: 115,
            tower_damage_percent: 110,
            tower_projectile_speed_percent: 110,
            nexus_hp_percent: 110,
            morgard_minion_buff_percent: 100,
        }
    }

    fn neutral() -> Self {
        Self {
            minion_hp_percent: 100,
            minion_damage_percent: 100,
            minion_growth_hp_percent: 100,
            minion_growth_damage_percent: 100,
            tower_hp_percent: 100,
            tower_damage_percent: 100,
            tower_projectile_speed_percent: 100,
            nexus_hp_percent: 100,
            morgard_minion_buff_percent: 100,
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        Self::parse_with_base(raw, Self::default())
    }

    fn parse_with_base(raw: &str, mut config: Self) -> Option<Self> {
        let mut parsed_any = false;

        for entry in raw.split(|ch| matches!(ch, ';' | ',' | '\n' | '\r')) {
            let entry = entry
                .split_once('#')
                .map_or(entry, |(before_comment, _)| before_comment)
                .trim();
            if entry.is_empty() {
                continue;
            }

            let Some((key, value)) = entry.split_once('=') else {
                continue;
            };

            let value = value.trim().trim_end_matches('%').trim();
            let Ok(percent) = value.parse::<usize>() else {
                return None;
            };

            let percent = clamp_percent(percent);
            match key.trim() {
                "minion_hp" => config.minion_hp_percent = percent,
                "minion_damage" => config.minion_damage_percent = percent,
                "minion_growth_hp" => config.minion_growth_hp_percent = percent,
                "minion_growth_damage" => config.minion_growth_damage_percent = percent,
                "tower_hp" => config.tower_hp_percent = percent,
                "tower_damage" => config.tower_damage_percent = percent,
                "tower_projectile_speed" => config.tower_projectile_speed_percent = percent,
                "nexus_hp" => config.nexus_hp_percent = percent,
                "morgard_minion_buff" => config.morgard_minion_buff_percent = percent,
                _ => continue,
            }

            parsed_any = true;
        }

        parsed_any.then_some(config)
    }

    fn serialize(self) -> String {
        format!(
            "minion_hp={};minion_damage={};minion_growth_hp={};minion_growth_damage={};tower_hp={};tower_damage={};tower_projectile_speed={};nexus_hp={};morgard_minion_buff={}",
            self.minion_hp_percent,
            self.minion_damage_percent,
            self.minion_growth_hp_percent,
            self.minion_growth_damage_percent,
            self.tower_hp_percent,
            self.tower_damage_percent,
            self.tower_projectile_speed_percent,
            self.nexus_hp_percent,
            self.morgard_minion_buff_percent
        )
    }
}

fn clamp_percent(percent: usize) -> usize {
    percent.clamp(MIN_PERCENT, MAX_PERCENT)
}

fn load_config(save_data: &ModSaveData, key: &str) -> Option<BalanceConfig> {
    load_config_with_base(save_data, key, BalanceConfig::default())
}

fn load_config_with_base(
    save_data: &ModSaveData,
    key: &str,
    base: BalanceConfig,
) -> Option<BalanceConfig> {
    save_data
        .get_string(MOD_ID, key)
        .and_then(|value| BalanceConfig::parse_with_base(&value, base))
}

fn desired_config(save_data: &ModSaveData) -> BalanceConfig {
    load_config(save_data, CONFIG_KEY).unwrap_or_default()
}

fn applied_config(save_data: &ModSaveData) -> BalanceConfig {
    let save_version = save_data.save_version(MOD_ID);
    let applied_parse_base = if save_version >= BALANCE_SAVE_VERSION {
        BalanceConfig::default()
    } else {
        BalanceConfig::legacy_default()
    };

    load_config_with_base(save_data, APPLIED_CONFIG_KEY, applied_parse_base).unwrap_or_else(|| {
        if save_version >= 2 {
            BalanceConfig::legacy_default()
        } else {
            BalanceConfig::neutral()
        }
    })
}

fn scale_usize(value: &mut usize, from_percent: usize, to_percent: usize) {
    if *value == 0 || from_percent == to_percent {
        return;
    }

    let from_percent = from_percent.max(MIN_PERCENT);
    *value = ((*value).saturating_mul(to_percent) + (from_percent / 2)) / from_percent;
    if *value == 0 {
        *value = 1;
    }
}

fn scale_i32(value: &mut i32, from_percent: usize, to_percent: usize) {
    if *value == 0 || from_percent == to_percent {
        return;
    }

    let from_percent = from_percent.max(MIN_PERCENT) as i64;
    let raw = *value as i64 * to_percent as i64;
    let scaled = if raw >= 0 {
        (raw + (from_percent / 2)) / from_percent
    } else {
        (raw - (from_percent / 2)) / from_percent
    };
    *value = scaled.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
}

fn scale_buff_state(buff: &mut BuffState, from_percent: usize, to_percent: usize) {
    scale_i32(&mut buff.attack, from_percent, to_percent);
    scale_i32(&mut buff.attack_mult, from_percent, to_percent);
    scale_i32(&mut buff.magic_power, from_percent, to_percent);
    scale_i32(&mut buff.magic_power_mult, from_percent, to_percent);
    scale_i32(&mut buff.defence, from_percent, to_percent);
    scale_i32(&mut buff.defence_mult, from_percent, to_percent);
    scale_i32(&mut buff.hp_regen, from_percent, to_percent);
    scale_i32(&mut buff.magic_resistance, from_percent, to_percent);
    scale_i32(&mut buff.magic_resistance_mult, from_percent, to_percent);
    scale_i32(&mut buff.vamp, from_percent, to_percent);
    scale_i32(&mut buff.hp_mult, from_percent, to_percent);
    scale_i32(&mut buff.move_speed_mult, from_percent, to_percent);
    scale_i32(&mut buff.attack_speed_mult, from_percent, to_percent);
    scale_i32(&mut buff.skill_cooldown_mult, from_percent, to_percent);
    scale_usize(&mut buff.damage_reflect, from_percent, to_percent);
    scale_usize(&mut buff.damaged_amplify, from_percent, to_percent);
    scale_usize(&mut buff.defence_penetration, from_percent, to_percent);
    scale_usize(
        &mut buff.magic_resistance_penetration,
        from_percent,
        to_percent,
    );
    scale_usize(&mut buff.toughness, from_percent, to_percent);
    scale_usize(&mut buff.heal_reduce, from_percent, to_percent);
    scale_usize(
        &mut buff.base_attack_enemy_max_hp_damage,
        from_percent,
        to_percent,
    );
    scale_usize(&mut buff.self_max_hp_damage, from_percent, to_percent);
    scale_usize(
        &mut buff.skill_enemy_max_hp_damage,
        from_percent,
        to_percent,
    );
    scale_usize(&mut buff.damaged_reduce, from_percent, to_percent);
    scale_usize(&mut buff.dot_amplify, from_percent, to_percent);
    scale_i32(&mut buff.ult_cooldown_mult, from_percent, to_percent);
    scale_i32(&mut buff.crit_chance, from_percent, to_percent);
    scale_i32(&mut buff.radius_mult, from_percent, to_percent);
    scale_usize(
        &mut buff.base_attack_damaged_reduce,
        from_percent,
        to_percent,
    );
    scale_usize(&mut buff.skill_damaged_reduce, from_percent, to_percent);
}

fn scale_u64(value: &mut u64, from_percent: usize, to_percent: usize) {
    if *value == 0 || from_percent == to_percent {
        return;
    }

    let from_percent = from_percent.max(MIN_PERCENT) as u64;
    *value = ((*value).saturating_mul(to_percent as u64) + (from_percent / 2)) / from_percent;
    if *value == 0 {
        *value = 1;
    }
}

fn scale_stat(
    stat: &mut EntityStat,
    from_hp_percent: usize,
    to_hp_percent: usize,
    from_attack_percent: usize,
    to_attack_percent: usize,
) {
    scale_usize(&mut stat.hp, from_hp_percent, to_hp_percent);
    scale_usize(&mut stat.attack, from_attack_percent, to_attack_percent);
}

macro_rules! balance_tower {
    ($tower:expr, $from:expr, $to:expr) => {{
        scale_usize(
            &mut $tower.stat.hp,
            $from.tower_hp_percent,
            $to.tower_hp_percent,
        );
        scale_usize(
            &mut $tower.stat.attack,
            $from.tower_damage_percent,
            $to.tower_damage_percent,
        );
        scale_usize(
            &mut $tower.attack.attack,
            $from.tower_damage_percent,
            $to.tower_damage_percent,
        );
        scale_u64(
            &mut $tower.attack.speed,
            $from.tower_projectile_speed_percent,
            $to.tower_projectile_speed_percent,
        );
    }};
}

fn apply_balance_delta(setting: &mut GameSetting, from: BalanceConfig, to: BalanceConfig) {
    scale_stat(
        &mut setting.melee_minion.stat,
        from.minion_hp_percent,
        to.minion_hp_percent,
        from.minion_damage_percent,
        to.minion_damage_percent,
    );
    scale_stat(
        &mut setting.melee_minion.growth,
        from.minion_growth_hp_percent,
        to.minion_growth_hp_percent,
        from.minion_growth_damage_percent,
        to.minion_growth_damage_percent,
    );
    scale_stat(
        &mut setting.melee_minion.growth_2v2,
        from.minion_growth_hp_percent,
        to.minion_growth_hp_percent,
        from.minion_growth_damage_percent,
        to.minion_growth_damage_percent,
    );
    scale_stat(
        &mut setting.melee_minion.growth_3v3,
        from.minion_growth_hp_percent,
        to.minion_growth_hp_percent,
        from.minion_growth_damage_percent,
        to.minion_growth_damage_percent,
    );
    scale_usize(
        &mut setting.melee_minion.attack.attack,
        from.minion_damage_percent,
        to.minion_damage_percent,
    );
    scale_usize(
        &mut setting.melee_minion.from_tower_damage,
        from.tower_damage_percent,
        to.tower_damage_percent,
    );
    scale_usize(
        &mut setting.melee_minion.from_tower_damage_2v2,
        from.tower_damage_percent,
        to.tower_damage_percent,
    );
    scale_usize(
        &mut setting.melee_minion.from_tower_damage_3v3,
        from.tower_damage_percent,
        to.tower_damage_percent,
    );

    scale_stat(
        &mut setting.range_minion.stat,
        from.minion_hp_percent,
        to.minion_hp_percent,
        from.minion_damage_percent,
        to.minion_damage_percent,
    );
    scale_stat(
        &mut setting.range_minion.growth,
        from.minion_growth_hp_percent,
        to.minion_growth_hp_percent,
        from.minion_growth_damage_percent,
        to.minion_growth_damage_percent,
    );
    scale_stat(
        &mut setting.range_minion.growth_2v2,
        from.minion_growth_hp_percent,
        to.minion_growth_hp_percent,
        from.minion_growth_damage_percent,
        to.minion_growth_damage_percent,
    );
    scale_stat(
        &mut setting.range_minion.growth_3v3,
        from.minion_growth_hp_percent,
        to.minion_growth_hp_percent,
        from.minion_growth_damage_percent,
        to.minion_growth_damage_percent,
    );
    scale_usize(
        &mut setting.range_minion.attack.attack,
        from.minion_damage_percent,
        to.minion_damage_percent,
    );
    scale_usize(
        &mut setting.range_minion.from_tower_damage,
        from.tower_damage_percent,
        to.tower_damage_percent,
    );
    scale_usize(
        &mut setting.range_minion.from_tower_damage_2v2,
        from.tower_damage_percent,
        to.tower_damage_percent,
    );
    scale_usize(
        &mut setting.range_minion.from_tower_damage_3v3,
        from.tower_damage_percent,
        to.tower_damage_percent,
    );

    balance_tower!(setting.tower, from, to);
    balance_tower!(setting.tower_2v2, from, to);
    balance_tower!(setting.tower_3v3, from, to);
    balance_tower!(setting.twin_tower, from, to);
    scale_usize(
        &mut setting.nexus.stat.hp,
        from.nexus_hp_percent,
        to.nexus_hp_percent,
    );
    scale_buff_state(
        &mut setting.epic_minion_buff_increase,
        from.morgard_minion_buff_percent,
        to.morgard_minion_buff_percent,
    );
}

fn apply_config(ctx: &mut ServerModContext, config: BalanceConfig) {
    let previous_config = applied_config(&ctx.database.mod_save_data);

    if previous_config != config {
        apply_balance_delta(&mut ctx.database.game_setting, previous_config, config);
    }

    let serialized = config.serialize();
    let _ = ctx
        .database
        .mod_save_data
        .set_string(MOD_ID, CONFIG_KEY, serialized.clone());
    let _ = ctx
        .database
        .mod_save_data
        .set_string(MOD_ID, APPLIED_CONFIG_KEY, serialized);
    let _ = ctx
        .database
        .mod_save_data
        .set_version(MOD_ID, BALANCE_SAVE_VERSION);
}

#[derive(Default)]
struct BalanceServerExtension {
    last_config_text: Mutex<Option<String>>,
}

impl BalanceServerExtension {
    fn apply_file_config(&self, ctx: &mut ServerModContext, force: bool) -> bool {
        let text = match read_or_create_config_text() {
            Ok(text) => text,
            Err(message) => {
                log_line(&format!("failed to read {CONFIG_FILE_NAME}: {message}"));
                return false;
            }
        };

        if !force && self.last_config_text_matches(&text) {
            return true;
        }

        let Some(config) = BalanceConfig::parse(&text) else {
            self.set_last_config_text(text);
            log_line(&format!(
                "ignored invalid {CONFIG_FILE_NAME}; keep key=value percentages between {MIN_PERCENT} and {MAX_PERCENT}"
            ));
            return false;
        };

        apply_config(ctx, config);
        self.set_last_config_text(text);
        log_line(&format!(
            "loaded {CONFIG_FILE_NAME}: {}",
            config.serialize()
        ));
        true
    }

    fn last_config_text_matches(&self, text: &str) -> bool {
        self.last_config_text
            .lock()
            .is_ok_and(|last_config_text| last_config_text.as_deref() == Some(text))
    }

    fn set_last_config_text(&self, text: String) {
        if let Ok(mut last_config_text) = self.last_config_text.lock() {
            *last_config_text = Some(text);
        }
    }
}

impl ModServerExtension for BalanceServerExtension {
    fn on_server_start(&self, ctx: &mut ServerModContext) {
        log_line("server extension started; loading balance config file");
        if !self.apply_file_config(ctx, true) {
            let config = desired_config(&ctx.database.mod_save_data);
            apply_config(ctx, config);
            log_line(&format!(
                "using saved/default balance config: {}",
                config.serialize()
            ));
        }
    }

    fn before_management_tick(&self, ctx: &mut ServerModContext) {
        let _ = self.apply_file_config(ctx, false);
    }
}

fn read_or_create_config_text() -> Result<String, String> {
    for path in config_paths() {
        if path.exists() {
            return fs::read_to_string(&path).map_err(|err| format!("{} ({err})", path.display()));
        }
    }

    let contents = config_file_contents(BalanceConfig::default());
    for path in config_paths() {
        let Some(parent) = path.parent() else {
            continue;
        };

        if !parent.join("mod.mod_info").exists() {
            continue;
        }

        fs::write(&path, &contents).map_err(|err| format!("{} ({err})", path.display()))?;
        log_line(&format!(
            "created default {CONFIG_FILE_NAME} at {}",
            path.display()
        ));
        return Ok(contents);
    }

    Err("mod folder not found".to_string())
}

fn config_file_contents(config: BalanceConfig) -> String {
    format!(
        "# Minion and Tower Balance\n# Edit these values as percentages. 100 means vanilla. Values are clamped to {MIN_PERCENT}..={MAX_PERCENT}.\n# morgard_minion_buff scales the temporary Morgard minion empowerment strength.\n# Save the file while a save is loaded to reload it on the next management tick, or restart/reload the save.\n\nminion_hp={}\nminion_damage={}\nminion_growth_hp={}\nminion_growth_damage={}\ntower_hp={}\ntower_damage={}\ntower_projectile_speed={}\nnexus_hp={}\nmorgard_minion_buff={}\n",
        config.minion_hp_percent,
        config.minion_damage_percent,
        config.minion_growth_hp_percent,
        config.minion_growth_damage_percent,
        config.tower_hp_percent,
        config.tower_damage_percent,
        config.tower_projectile_speed_percent,
        config.nexus_hp_percent,
        config.morgard_minion_buff_percent
    )
}

fn init(_ctx: &GameCtx) -> ModRegistration {
    let mut registration = ModRegistration::new(MOD_ID);
    registration.set_server_extension(BalanceServerExtension::default());
    registration
}

fn log_line(message: &str) {
    for path in log_paths() {
        if let Some(parent) = path.parent() {
            if !parent.join("mod.mod_info").exists() {
                continue;
            }
        }

        let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
            continue;
        };

        let _ = writeln!(file, "{}", message);
        break;
    }
}

fn config_paths() -> Vec<PathBuf> {
    mod_file_paths(CONFIG_FILE_NAME)
}

fn log_paths() -> Vec<PathBuf> {
    mod_file_paths(LOG_FILE_NAME)
}

fn mod_file_paths(file_name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(current_dir) = std::env::current_dir() {
        paths.push(current_dir.join(file_name));
        paths.push(current_dir.join("mods").join(MOD_ID).join(file_name));
    }

    paths
}

declare_mod!(init);
