#[derive(Clone,Copy)]
pub struct Group {owner:u32,accessor:usize,family:u64,status:i32,motion:u64,frame:f32,aerial_attack:bool,claimed:bool,external:[u32;32],cursor:usize}
impl Group {
 pub const fn new()->Self {Self{owner:u32::MAX,accessor:0,family:0,status:0,motion:0,frame:0.0,aerial_attack:false,claimed:false,external:[u32::MAX;32],cursor:0}}
 pub fn observe(&mut self,owner:u32,accessor:usize,family:u64,status:i32,motion:u64,frame:f32) {
  self.observe_move(owner,accessor,family,status,motion,frame,family>=100);
 }
 pub fn observe_move(&mut self,owner:u32,accessor:usize,family:u64,status:i32,motion:u64,frame:f32,aerial_attack:bool) {
  if self.owner!=owner || self.accessor!=accessor {*self=Self::new();self.owner=owner;self.accessor=accessor;}
  let changed=if family>0 {self.family!=family} else {self.family!=0 || self.status!=status || self.motion!=motion};
  // A fresh playback of the same aerial is a new move, even if no fall frame was observed.
  // A one-frame hitstop adjustment is not a restart, and a landing hit stays with its aerial.
  let same_playback=self.status==status && self.motion==motion;
  let restart=(family==0 || family>=100) && same_playback && frame+1.5<self.frame;
  let aerial_reentry=family>=100 && aerial_attack && !self.aerial_attack;
  if changed || restart || aerial_reentry {self.claimed=false;}
  self.family=family;self.status=status;self.motion=motion;self.frame=frame;self.aerial_attack=aerial_attack;
 }
 pub fn action(&mut self,jab:bool)->bool {if jab {return true;}if self.claimed {return false;}self.claimed=true;true}
 pub fn external(&mut self,id:u32)->bool {if self.external.contains(&id) {return false;}self.external[self.cursor]=id;self.cursor=(self.cursor+1)%self.external.len();true}
 pub fn held(&mut self,id:u32) {for old in self.external.iter_mut() {if *old==id {*old=u32::MAX;}}}
 pub fn weapon(&mut self,id:u32)->bool {if (1..=4).contains(&self.family) {let allowed=self.action(false);self.external(id);allowed}else{self.external(id)}}
}
#[cfg(test)] mod tests {use super::*;
 #[test] fn three_fast_jab_hits_are_three_counts() {let mut g=Group::new();g.observe(1,10,0,1,11,3.0);assert!(g.action(true));assert!(g.action(true));assert!(g.action(true));}
 #[test] fn aerial_multihit_is_one_until_next_attack() {let mut g=Group::new();g.observe(1,10,101,2,21,4.0);assert!(g.action(false));for f in [6.0,9.0,12.0] {g.observe(1,10,101,2,21,f);assert!(!g.action(false));}g.observe(1,10,0,0,0,1.0);g.observe(1,10,101,2,21,4.0);assert!(g.action(false));}
 #[test] fn hitstop_rewind_does_not_add_a_hit() {let mut g=Group::new();g.observe(1,10,0,2,21,6.0);assert!(g.action(false));g.observe(1,10,0,2,21,5.0);assert!(!g.action(false));}
 #[test] fn looping_special_is_one_including_its_projectiles() {let mut g=Group::new();g.observe(1,10,1,10,100,4.0);assert!(g.weapon(30));g.observe(1,10,1,11,101,1.0);assert!(!g.weapon(31));g.observe(1,10,1,11,101,1.0);assert!(!g.action(false));assert!(!g.weapon(32));}
 #[test] fn landing_hit_is_part_of_same_aerial() {let mut g=Group::new();g.observe(1,10,102,2,21,6.0);assert!(g.action(false));g.observe(1,10,102,3,22,1.0);assert!(!g.action(false));}
 #[test] fn airborne_item_is_separate_from_melee_attack() {let mut g=Group::new();g.observe(1,10,0,2,21,4.0);assert!(g.action(false));assert!(g.external(30));assert!(!g.external(30));}
 #[test] fn picked_up_item_can_score_on_next_throw() {let mut g=Group::new();assert!(g.external(30));assert!(!g.external(30));g.held(30);assert!(g.external(30));}
 #[test] fn late_projectile_does_not_consume_current_melee_move() {let mut g=Group::new();g.observe(1,10,0,2,21,4.0);assert!(g.weapon(30));assert!(g.action(false));assert!(!g.weapon(30));}
}

#[cfg(test)] mod aerial_regressions {use super::*;
 #[test] fn three_rapid_uses_of_same_up_air_count_three_moves() {let mut g=Group::new();for _ in 0..3 {g.observe_move(1,10,103,2,21,4.0,true);assert!(g.action(false));g.observe_move(1,10,103,2,21,20.0,true);}}
 #[test] fn new_aerial_can_start_after_frame_one() {let mut g=Group::new();g.observe_move(1,10,103,2,21,10.0,true);assert!(g.action(false));g.observe_move(1,10,103,2,21,3.0,true);assert!(g.action(false));}
 #[test] fn hits_inside_one_multihit_aerial_still_count_once() {let mut g=Group::new();for (i,f) in [4.0,8.0,12.0].iter().enumerate() {g.observe_move(1,10,103,2,21,*f,true);assert_eq!(g.action(false),i==0);}}
 #[test] fn landing_stays_with_old_aerial_but_next_takeoff_is_new() {let mut g=Group::new();g.observe_move(1,10,103,2,21,10.0,true);assert!(g.action(false));g.observe_move(1,10,103,3,22,1.0,false);assert!(!g.action(false));g.observe_move(1,10,103,2,21,4.0,true);assert!(g.action(false));}
 #[test] fn aerial_hitstop_adjustment_does_not_create_another_move() {let mut g=Group::new();g.observe_move(1,10,103,2,21,6.0,true);assert!(g.action(false));g.observe_move(1,10,103,2,21,5.0,true);assert!(!g.action(false));}
}
