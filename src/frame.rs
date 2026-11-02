use super::*;
use skyline::hooks::{Region,getRegionAddress};
use std::collections::HashMap;
use std::sync::RwLock;
use crate::data::*;

use std::io::Write;
use std::sync::atomic::{AtomicUsize,Ordering};
static TRACE_COUNT:AtomicUsize=AtomicUsize::new(0);
fn trace(message:&str) {
    if TRACE_COUNT.fetch_add(1,Ordering::Relaxed)>=12 {return;}
    if let Ok(mut f)=std::fs::OpenOptions::new().create(true).append(true).open("sd:/ultimate/arcropolis/logs/sonic_combo_runtime.log") {let _=writeln!(f,"{}",message);}
}
static FRAME_SEEN:AtomicUsize=AtomicUsize::new(0);
static HIT_SEEN:AtomicUsize=AtomicUsize::new(0);
lazy_static! {
    pub static ref EFFECT_CONFIG: RwLock<HashMap<i32, CharacterData>> = RwLock::new(HashMap::new());
}

pub fn get_fighter_kind_from_string(target_kind: &str) -> i32 {
    match target_kind {
        "sonic" => *FIGHTER_KIND_SONIC,
        _ => *FIGHTER_KIND_NONE,
    }
}

const FIGHTER_INSTANCE_WORK_ID_FLAG_COMBO_EFFECT_SPAWNED : i32 = 0x20000116;
const FIGHTER_INSTANCE_WORK_ID_INT_COMBO_TIMER : i32 = 0x1000010B;
const FIGHTER_INSTANCE_WORK_ID_INT_COMBO_COUNTER : i32 = 0x1000010C;

unsafe extern "C" fn sonic_combo_fighter_frame(fighter: &mut L2CFighterCommon) {
    if FRAME_SEEN.fetch_add(1,Ordering::Relaxed)==0 {trace("fighter callback active");}
    let boma = fighter.module_accessor;
    let kind = utility::get_kind(&mut *boma);
    let settings = match EFFECT_CONFIG.read().expect("Failed to read config").get(&kind) {
        Some(settings) => settings.clone(),
        None => return,
    };
    let status_kind = StatusModule::status_kind(boma);
    let status_frame = fighter.global_table[0xe].get_f32();
    let combo_timer = WorkModule::get_int(boma, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_TIMER);
    
    // Decrease combo timer every frame
    let catch_statuses = [
        *FIGHTER_STATUS_KIND_CATCH_ATTACK,
        *FIGHTER_STATUS_KIND_CATCH_DASH,
        *FIGHTER_STATUS_KIND_CATCH_DASH_PULL,
        *FIGHTER_STATUS_KIND_CATCH_PULL,
        *FIGHTER_STATUS_KIND_CATCH_TURN,
        *FIGHTER_STATUS_KIND_CATCH_WAIT,
    ];
    if !catch_statuses.contains(&status_kind) {
        WorkModule::dec_int(boma, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_TIMER);
    } else {
        // Decrease timer slower in grabs
        if status_frame % 3.0 == 0.0 {
            WorkModule::dec_int(boma, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_TIMER);
        }
    }

    // Reset counter to 0 when timer expires
    if combo_timer <= 0 {
        WorkModule::set_int(boma, 0, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_COUNTER);
    }
    
    // Reset effect spawned flag after a few frames
    let max_combo_frame = settings.combo_frame;
    let cooldown_frames = settings.cooldown_frame;
    if combo_timer + cooldown_frames <= max_combo_frame {
        WorkModule::off_flag(boma, FIGHTER_INSTANCE_WORK_ID_FLAG_COMBO_EFFECT_SPAWNED);
    }
}

static mut NOTIFY_LOG_EVENT_COLLISION_HIT_OFFSET : usize = 0x67a7b0;

#[skyline::hook(offset = NOTIFY_LOG_EVENT_COLLISION_HIT_OFFSET)]
pub unsafe fn sonic_combo_collision_hit(fighter_manager: *mut smash::app::FighterManager, attacker_id: u32, defender_id: u32, damage: f32, attack_kind: i32, damage_meteor: bool) -> u64 {
    let og = original!()(fighter_manager, attacker_id, defender_id, damage, attack_kind, damage_meteor);
    let attacker_boma = sv_battle_object::module_accessor(attacker_id);
    let defender_boma = sv_battle_object::module_accessor(defender_id);
    let attacker_category = utility::get_category(&mut *attacker_boma);
    let defender_category = utility::get_category(&mut *defender_boma);

    // Allowed categories
    let attacker_allowed_cats = [
        *BATTLE_OBJECT_CATEGORY_FIGHTER, // Fighter
        *BATTLE_OBJECT_CATEGORY_WEAPON, // Article
    ];
    let defender_allowed_cats = [
        *BATTLE_OBJECT_CATEGORY_FIGHTER, // Fighter
        *BATTLE_OBJECT_CATEGORY_ENEMY, // Stage hazard, etc?
        *BATTLE_OBJECT_CATEGORY_ITEM, // Assists, pokemon, probably the crates too
    ];
    let attacker_is_fighter = attacker_category == *BATTLE_OBJECT_CATEGORY_FIGHTER;

    // Handle attacker and defender not being the expected types
    if !attacker_allowed_cats.contains(&attacker_category)
    || !defender_allowed_cats.contains(&defender_category) {
        return og;
    }
    let owner_boma = if attacker_is_fighter {
        attacker_boma
    } else {
        sv_battle_object::module_accessor((WorkModule::get_int(attacker_boma, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER)) as u32)
    };

    // Make sure character is allowed to have effects
    let attacker_kind = utility::get_kind(&mut *owner_boma);
    let settings = match EFFECT_CONFIG.read().expect("Failed to read config").get(&attacker_kind) {
        Some(settings) => settings.clone(),
        None => return og,
    };

    if HIT_SEEN.fetch_add(1,Ordering::Relaxed)==0 {trace("own hit callback active");}
    // Clear counter when getting hit
    WorkModule::set_int(defender_boma, 0, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_COUNTER);
    WorkModule::set_int(defender_boma, 0, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_TIMER);

    // Successful hit, spawn effects & sound, set variables
    if !WorkModule::is_flag(owner_boma, FIGHTER_INSTANCE_WORK_ID_FLAG_COMBO_EFFECT_SPAWNED)
    && !WorkModule::is_flag(owner_boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL)
    && HitModule::get_whole(defender_boma, *HIT_STATUS_INVINCIBLE) == 0
    && damage > 1.0 {
        WorkModule::on_flag(owner_boma, FIGHTER_INSTANCE_WORK_ID_FLAG_COMBO_EFFECT_SPAWNED);
        WorkModule::inc_int(owner_boma, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_COUNTER);
        WorkModule::set_int(owner_boma, settings.combo_frame, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_TIMER);
        let combo_counter = WorkModule::get_int(owner_boma, FIGHTER_INSTANCE_WORK_ID_INT_COMBO_COUNTER);
        if let Some(rank) = settings.rank_for_hits(combo_counter) {
            let pos = *PostureModule::pos(attacker_boma);
            let lr = PostureModule::lr(attacker_boma);
            let position = Vector3f {
                x: pos.x + settings.offset_x * lr,
                y: pos.y + settings.offset_y,
                z: pos.z,
            };
            let rotation = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            SoundModule::play_se(owner_boma, Hash40::new(&rank.sound), true, false, false, false, enSEType(0));
            let handle=EffectModule::req(owner_boma, Hash40::new(&rank.effect), &position, &rotation, rank.scale, 0, -1, false, 0);
            trace(&format!("rank={} hits={} effect_handle={}",rank.effect,combo_counter,handle));
        }
    }

    og
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

static OFFSET_SEARCH_CODE: &[u8] = &[
    0xff, 0x03, 0x03, 0xd1, //.text:0000007100675A20                 SUB             SP, SP, #0xC0
    0xe8, 0x2b, 0x00, 0xfd, //.text:0000007100675A24                 STR             D8, [SP,#0xB0+var_60]
    0xfc, 0x6f, 0x06, 0xa9, //.text:0000007100675A28                 STP             X28, X27, [SP,#0xB0+var_50]
    0xfa, 0x67, 0x07, 0xa9, //.text:0000007100675A2C                 STP             X26, X25, [SP,#0xB0+var_40]
    0xf8, 0x5f, 0x08, 0xa9, //.text:0000007100675A30                 STP             X24, X23, [SP,#0xB0+var_30]
    0xf6, 0x57, 0x09, 0xa9, //.text:0000007100675A34                 STP             X22, X21, [SP,#0xB0+var_20]
    0xf4, 0x4f, 0x0a, 0xa9, //.text:0000007100675A38                 STP             X20, X19, [SP,#0xB0+var_10]
    0xfd, 0x7b, 0x0b, 0xa9, //.text:0000007100675A3C                 STP             X29, X30, [SP,#0xB0+var_s0]
    0xfd, 0xc3, 0x02, 0x91, //.text:0000007100675A40                 ADD             X29, SP, #0xB0
    0xfb, 0x03, 0x00, 0xaa  //.text:0000007100675A44                 MOV             X27, X0
];

pub fn install() {
    let (loaded,path)=crate::config_loading::load(std::path::Path::new("sd:/ultimate/mods"),"sonic_combo_fx.toml",include_str!("../sonic_combo_fx.toml"));
    let config: HashMap<i32,CharacterData> = loaded.into_iter().filter_map(|(name,settings)| {
        let kind=get_fighter_kind_from_string(&name);
        if kind==*FIGHTER_KIND_NONE {None} else {Some((kind,settings))}
    }).collect();
    trace(&format!("started; config={}; fighters={}",path,config.len()));
    *EFFECT_CONFIG.write().expect("Failed to write config") = config;

    unsafe {
        let text_ptr = getRegionAddress(Region::Text) as *const u8;
        let text_size = (getRegionAddress(Region::Rodata) as usize) - (text_ptr as usize);
        let text = std::slice::from_raw_parts(text_ptr, text_size);
        if let Some(offset) = find_subsequence(text, OFFSET_SEARCH_CODE) {
            NOTIFY_LOG_EVENT_COLLISION_HIT_OFFSET = offset;
        }
    }

    skyline::install_hook!(
        sonic_combo_collision_hit
    );
    Agent::new("sonic")
        .on_line(Main, sonic_combo_fighter_frame)
        .install();
}
