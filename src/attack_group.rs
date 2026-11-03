#[derive(Clone,Copy)]
pub struct Group {owner:u32,accessor:usize,family:u64,status:i32,motion:u64,frame:f32,claimed:bool,external:[u32;32],cursor:usize}
impl Group {
 pub const fn new()->Self {Self{owner:u32::MAX,accessor:0,family:0,status:0,motion:0,frame:0.0,claimed:false,external:[u32::MAX;32],cursor:0}}
 pub fn observe(&mut self,owner:u32,accessor:usize,family:u64,status:i32,motion:u64,frame:f32) {
  if self.owner!=owner || self.accessor!=accessor {*self=Self::new();self.owner=owner;self.accessor=accessor;}
  let changed=if family>0 {self.family!=family} else {self.family!=0 || self.status!=status || self.motion!=motion};
  let restart=family==0 && frame<=1.0 && self.frame>1.5;
  if changed || restart {self.claimed=false;}
  self.family=family;self.status=status;self.motion=motion;self.frame=frame;
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
