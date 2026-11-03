use super::*;
use skyline::hooks::{Region,getRegionAddress};
use std::collections::HashMap;
use std::sync::RwLock;
use crate::data::*;

use std::io::Write;
use std::sync::atomic::{AtomicUsize,Ordering};
static PUMMELS:std::sync::Mutex<[crate::pummel::Pummel;8]>=std::sync::Mutex::new([crate::pummel::Pummel::new();8]);
unsafe fn bare_grab(boma:*mut BattleObjectModuleAccessor)->bool {
 [*FIGHTER_STATUS_KIND_CATCH,*FIGHTER_STATUS_KIND_CATCH_DASH,*FIGHTER_STATUS_KIND_CATCH_TURN,*FIGHTER_STATUS_KIND_CATCH_PULL,*FIGHTER_STATUS_KIND_CATCH_DASH_PULL,*FIGHTER_STATUS_KIND_CATCH_WAIT].contains(&StatusModule::status_kind(boma))
}
unsafe fn pummel_contact(boma:*mut BattleObjectModuleAccessor,slot:usize,hit:bool)->bool {
 let mut all=PUMMELS.lock().unwrap();let p=&mut all[slot];p.observe((*boma).battle_object_id,MotionModule::motion_kind(boma),MotionModule::frame(boma),StatusModule::status_kind(boma)==*FIGHTER_STATUS_KIND_CATCH_ATTACK);p.claim(hit)
}
static GROUPS:std::sync::Mutex<[crate::attack_group::Group;8]>=std::sync::Mutex::new([crate::attack_group::Group::new();8]);
lazy_static! {
 static ref FIGHTER_MOVE_FAMILIES:Vec<(i32,u64)>=vec![(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_HI_JUMP,3),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_LW_END,4),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_LW_HOLD,4),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_CANCEL,1),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_FAIL,1),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HIT,1),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING_START,1),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_HOMING,1),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_LANDING,1),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_N_REBOUND,1),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_DASH,2),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_END,2),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_HOLD_JUMP,2),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_HOLD,2),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_REBOUND,2),(*FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_TURN,2)];
 static ref MOVE_FAMILIES:Vec<(u64,u64)>={
  let mut result=Vec::new();
  for (index,dir) in ["n","s","hi","lw"].iter().enumerate() {for air in ["","air_"].iter() {for suffix in ["","_start","_hold","_loop","_end","_landing","_attack","_fall","_jump","_dash","_hit","_fail","_homing","_homing_start","_homing_end","_turn","_bound","_rise","_charge","_shoot","_attack_end","_attack_start","_jump_start","_jump_end","_wall"].iter() {result.push((Hash40::new(&format!("special_{}{}{}",air,dir,suffix)).hash,(index+1) as u64));}}}
  for (index,dir) in ["n","f","b","hi","lw"].iter().enumerate() {result.push((Hash40::new(&format!("attack_air_{}",dir)).hash,100+index as u64));result.push((Hash40::new(&format!("landing_air_{}",dir)).hash,100+index as u64));}
  result
 };
}
unsafe fn observe_attack(boma:*mut BattleObjectModuleAccessor,slot:usize) {
 let status=StatusModule::status_kind(boma);let motion=MotionModule::motion_kind(boma);
 let family=MOVE_FAMILIES.iter().find(|(hash,_)|*hash==motion).map(|(_,family)|*family).or_else(||FIGHTER_MOVE_FAMILIES.iter().find(|(kind,_)|*kind==status).map(|(_,family)|*family)).unwrap_or_else(|| {
  if status==*FIGHTER_STATUS_KIND_SPECIAL_N {1}else if status==*FIGHTER_STATUS_KIND_SPECIAL_S {2}else if status==*FIGHTER_STATUS_KIND_SPECIAL_HI {3}else if status==*FIGHTER_STATUS_KIND_SPECIAL_LW {4}else{0}
 });
 let mut all=GROUPS.lock().unwrap();all[slot].observe((*boma).battle_object_id,boma as usize,family,status,motion,MotionModule::frame(boma));
 if ItemModule::is_have_item(boma,0) {let id=ItemModule::get_have_item_id(boma,0);if id<=u32::MAX as u64 {all[slot].held(id as u32);}}
}
unsafe fn group_contact(owner:*mut BattleObjectModuleAccessor,source:*mut BattleObjectModuleAccessor)->bool {
 let slot=match combo_slot(owner){Some(slot)=>slot,None=>return false};observe_attack(owner,slot);
 let status=StatusModule::status_kind(owner);let category=utility::get_category(&mut *source);
 let loose_item=category==*BATTLE_OBJECT_CATEGORY_ITEM && !LinkModule::is_link(source,*ITEM_LINK_NO_HAVE);
 let mut all=GROUPS.lock().unwrap();let group=&mut all[slot];
 let accepted=if loose_item {group.external((*source).battle_object_id)}
 else if category==*BATTLE_OBJECT_CATEGORY_WEAPON {group.weapon((*source).battle_object_id)}
 else if status==*FIGHTER_STATUS_KIND_THROW || status==*FIGHTER_STATUS_KIND_CATCH_ATTACK {true}
 else {let jab=category==*BATTLE_OBJECT_CATEGORY_FIGHTER && [*FIGHTER_STATUS_KIND_ATTACK,*FIGHTER_STATUS_KIND_ATTACK_100].contains(&status);group.action(jab)};
 drop(all);
 // Remaining hits sustain the combo window without adding another attack count.
 if !accepted {let frames=EFFECT_CONFIG.read().unwrap().get(&utility::get_kind(&mut *owner)).map(|settings|settings.combo_frame).unwrap_or(90);let mut states=COMBOS.lock().unwrap();let combo=&mut states[slot];combo.identify((*owner).battle_object_id,owner as usize);if combo.hits>0 {combo.timer=frames;}}
 accepted
}

static THROW_CREDIT:std::sync::Mutex<[crate::throw_credit::ThrowCredit;8]>=std::sync::Mutex::new([crate::throw_credit::ThrowCredit::new();8]);
static COMBOS:std::sync::Mutex<[crate::combo_state::Combo;8]>=std::sync::Mutex::new([crate::combo_state::Combo::new();8]);
unsafe fn combo_slot(boma:*mut BattleObjectModuleAccessor)->Option<usize> {
 if boma.is_null() || utility::get_category(&mut *boma)!=*BATTLE_OBJECT_CATEGORY_FIGHTER {return None;}
 let slot=WorkModule::get_int(boma,*FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);if (0..8).contains(&slot) {Some(slot as usize)} else {None}
}
unsafe fn clear_own_defender(id:u32) {
 if !sv_battle_object::is_active(id) {return;}
 let boma=sv_battle_object::module_accessor(id);if boma.is_null() || utility::get_category(&mut *boma)!=*BATTLE_OBJECT_CATEGORY_FIGHTER || utility::get_kind(&mut *boma)!=*FIGHTER_KIND_SONIC {return;}
 if let Some(slot)=combo_slot(boma) {let mut all=COMBOS.lock().unwrap();all[slot].identify(id,boma as usize);all[slot].reset();}
}
static TRACE_COUNT:AtomicUsize=AtomicUsize::new(0);
fn trace(message:&str) {
    if TRACE_COUNT.fetch_add(1,Ordering::Relaxed)>=32 {return;}
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

unsafe extern "C" fn sonic_combo_fighter_frame(fighter:&mut L2CFighterCommon) {
    let boma=fighter.module_accessor;let slot=match combo_slot(boma){Some(slot)=>slot,None=>return};
    observe_attack(boma,slot);
    let kind=utility::get_kind(&mut *boma);let settings=match EFFECT_CONFIG.read().unwrap().get(&kind){Some(s)=>s.clone(),None=>return};
    let status=StatusModule::status_kind(boma);let frame=fighter.global_table[0xe].get_f32();
    THROW_CREDIT.lock().unwrap()[slot].observe((*boma).battle_object_id,MotionModule::motion_kind(boma),status==*FIGHTER_STATUS_KIND_THROW,false,0);
    let mut all=COMBOS.lock().unwrap();let combo=&mut all[slot];combo.identify((*boma).battle_object_id,boma as usize);
    if [*FIGHTER_STATUS_KIND_ENTRY,*FIGHTER_STATUS_KIND_DEAD,*FIGHTER_STATUS_KIND_REBIRTH,*FIGHTER_STATUS_KIND_WIN,*FIGHTER_STATUS_KIND_LOSE].contains(&status) {combo.reset();return;}
    let grabbing=[*FIGHTER_STATUS_KIND_CATCH_ATTACK,*FIGHTER_STATUS_KIND_CATCH_DASH,*FIGHTER_STATUS_KIND_CATCH_DASH_PULL,*FIGHTER_STATUS_KIND_CATCH_PULL,*FIGHTER_STATUS_KIND_CATCH_TURN,*FIGHTER_STATUS_KIND_CATCH_WAIT].contains(&status);
    combo.tick(!grabbing || frame%3.0==0.0,settings.combo_frame,settings.cooldown_frame);
    drop(all);
    if pummel_contact(boma,slot,status==*FIGHTER_STATUS_KIND_CATCH_ATTACK && MotionModule::frame(boma)>1.0 && AttackModule::is_infliction_status(boma,*COLLISION_KIND_MASK_HIT)) {award_contact(boma,*PostureModule::pos(boma),PostureModule::lr(boma),&settings);}
}

unsafe fn award_contact(owner_boma:*mut BattleObjectModuleAccessor,impact_pos:Vector3f,impact_lr:f32,settings:&CharacterData) {
 if WorkModule::is_flag(owner_boma,*FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL) {return;}
    let slot=match combo_slot(owner_boma){Some(slot)=>slot,None=>return};
    let combo_counter={let mut all=COMBOS.lock().unwrap();let combo=&mut all[slot];combo.identify((*owner_boma).battle_object_id,owner_boma as usize);combo.credit_contact(settings.combo_frame)};
    {
        if let Some(rank) = settings.rank_for_hits(combo_counter) {
            let pos = impact_pos;
            let lr = impact_lr;
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

}

unsafe fn source_fighter_id(id:u32)->Option<u32> {
    crate::hit_owner::fighter_owner(id,|current| {
        if current==u32::MAX || current==*BATTLE_OBJECT_ID_INVALID as u32 || !sv_battle_object::is_active(current) {return None;}
        let boma=sv_battle_object::module_accessor(current);
        if boma.is_null() {return None;}
        let category=utility::get_category(&mut *boma);
        if category==*BATTLE_OBJECT_CATEGORY_FIGHTER {Some(crate::hit_owner::Source::Fighter)}
        else if category==*BATTLE_OBJECT_CATEGORY_WEAPON {Some(crate::hit_owner::Source::OwnedBy(WorkModule::get_int(boma,*WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER) as u32))}
        else if category==*BATTLE_OBJECT_CATEGORY_ITEM {
            let owner=if LinkModule::is_link(boma,*ITEM_LINK_NO_HAVE) {LinkModule::get_parent_id(boma,*ITEM_LINK_NO_HAVE,true)} else {TeamModule::team_owner_id(boma)};
            if owner>u32::MAX as u64 {None} else {Some(crate::hit_owner::Source::OwnedBy(owner as u32))}
        } else {None}
    })
}

static mut NOTIFY_LOG_EVENT_COLLISION_HIT_OFFSET : usize = 0x67a7b0;

#[skyline::hook(offset = NOTIFY_LOG_EVENT_COLLISION_HIT_OFFSET)]
pub unsafe fn sonic_combo_collision_hit(fighter_manager: *mut smash::app::FighterManager, attacker_id: u32, defender_id: u32, damage: f32, attack_kind: i32, damage_meteor: bool) -> u64 {
    // Items can disappear when the native hit resolves, so keep their owner and impact first.
    if damage.is_finite() && damage>0.0 {clear_own_defender(defender_id);}
    let owner_id=source_fighter_id(attacker_id);
    let impact=if owner_id.is_some() && sv_battle_object::is_active(attacker_id) {
        let source=sv_battle_object::module_accessor(attacker_id);
        if source.is_null() {None} else {Some((*PostureModule::pos(source),PostureModule::lr(source),utility::get_category(&mut *source)))}
    } else {None};
    let contact_allowed=if let Some(id)=owner_id {
        if attacker_id==id && sv_battle_object::is_active(id) {
            let fighter=sv_battle_object::module_accessor(id);
            if fighter.is_null() || bare_grab(fighter) {false}
            else if damage.is_finite() && damage>0.0 && StatusModule::status_kind(fighter)==*FIGHTER_STATUS_KIND_CATCH_ATTACK && utility::get_kind(&mut *fighter)==*FIGHTER_KIND_SONIC {
                if let Some(slot)=combo_slot(fighter) {pummel_contact(fighter,slot,true)} else {false}
            } else if damage.is_finite() && damage>0.0 && StatusModule::status_kind(fighter)==*FIGHTER_STATUS_KIND_THROW && utility::get_kind(&mut *fighter)==*FIGHTER_KIND_SONIC {
                if let Some(slot)=combo_slot(fighter) {THROW_CREDIT.lock().unwrap()[slot].observe(id,MotionModule::motion_kind(fighter),true,false,1)>0} else {false}
            } else {true}
        } else {true}
    } else {false};
    let contact_allowed=contact_allowed && if damage.is_finite() && damage>0.0 {
        if let Some(id)=owner_id {let owner=sv_battle_object::module_accessor(id);let source=sv_battle_object::module_accessor(attacker_id);
            !owner.is_null() && !source.is_null() && utility::get_kind(&mut *owner)==*FIGHTER_KIND_SONIC && group_contact(owner,source)
        }else{false}
    }else{false};
    let og = original!()(fighter_manager, attacker_id, defender_id, damage, attack_kind, damage_meteor);
    if !contact_allowed {return og;}
    let owner_id=match owner_id {Some(id)=>id,None=>return og};
    if owner_id==defender_id {return og;}
    if !sv_battle_object::is_active(owner_id) || !sv_battle_object::is_active(defender_id) {return og;}
    let owner_boma=sv_battle_object::module_accessor(owner_id);
    let defender_boma=sv_battle_object::module_accessor(defender_id);
    if owner_boma.is_null() || defender_boma.is_null() {return og;}
    let (impact_pos,impact_lr,source_category)=match impact {Some(value)=>value,None=>return og};
    let defender_category=utility::get_category(&mut *defender_boma);
    if ![*BATTLE_OBJECT_CATEGORY_FIGHTER,*BATTLE_OBJECT_CATEGORY_ENEMY,*BATTLE_OBJECT_CATEGORY_ITEM].contains(&defender_category) {return og;}

    // Make sure character is allowed to have effects
    let attacker_kind = utility::get_kind(&mut *owner_boma);
    let settings = match EFFECT_CONFIG.read().expect("Failed to read config").get(&attacker_kind) {
        Some(settings) => settings.clone(),
        None => return og,
    };

    if HIT_SEEN.fetch_add(1,Ordering::Relaxed)==0 {trace("own hit callback active");}
    if WorkModule::is_flag(owner_boma,*FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL) || HitModule::get_whole(defender_boma,*HIT_STATUS_INVINCIBLE)!=0 || !damage.is_finite() || damage<=0.0 {return og;}
    let slot=match combo_slot(owner_boma){Some(slot)=>slot,None=>return og};
    award_contact(owner_boma,impact_pos,impact_lr,&settings);
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
