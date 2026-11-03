#[derive(Clone,Copy)]
pub struct Combo {pub object:u32,pub accessor:usize,pub hits:i32,pub timer:i32,pub spawned:bool}
impl Combo {
 pub const fn new()->Self {Self{object:u32::MAX,accessor:0,hits:0,timer:0,spawned:false}}
 pub fn identify(&mut self,object:u32,accessor:usize) {if self.object!=object || self.accessor!=accessor {*self=Self::new();self.object=object;self.accessor=accessor;}}
 pub fn reset(&mut self) {self.hits=0;self.timer=0;self.spawned=false;}
 pub fn tick(&mut self,decrement:bool,frames:i32,cooldown:i32) {let old=self.timer;if decrement {self.timer=self.timer.saturating_sub(1);}if old<=0 {self.hits=0;}if old.saturating_add(cooldown)<=frames {self.spawned=false;}}
 pub fn credit_contact(&mut self,frames:i32)->i32 {self.spawned=true;self.hits=self.hits.saturating_add(1);self.timer=frames;self.hits}
 pub fn credit(&mut self,frames:i32)->Option<i32> {if self.spawned {return None;}self.spawned=true;self.hits=self.hits.saturating_add(1);self.timer=frames;Some(self.hits)}
}
#[cfg(test)] mod tests {use super::*;
 #[test] fn two_fighters_keep_independent_combos() {let mut slots=[Combo::new();8];slots[0].identify(0,10);slots[1].identify(65536,20);slots[0].credit(90);slots[1].credit(90);slots[1].spawned=false;assert_eq!(slots[1].credit(90),Some(2));assert_eq!(slots[0].hits,1);slots[0].reset();assert_eq!(slots[1].hits,2);}
 #[test] fn player_order_does_not_change_scoring() {for slot in 0..8 {let mut slots=[Combo::new();8];let s=&mut slots[slot];s.identify(slot as u32,slot+1);assert_eq!(s.credit(90),Some(1));for _ in 0..17 {s.tick(true,90,16);}assert_eq!(s.credit(90),Some(2));}}
 #[test] fn replacement_fighter_cannot_inherit_score() {let mut s=Combo::new();s.identify(0,10);s.credit(90);s.identify(65536,20);assert_eq!(s.hits,0);assert_eq!(s.credit(90),Some(1));}
 #[test] fn expired_combo_starts_over() {let mut s=Combo::new();s.credit(90);for _ in 0..91 {s.tick(true,90,16);}assert_eq!(s.credit(90),Some(1));}
}

#[cfg(test)] mod contact_tests {use super::*;
 #[test] fn rapid_three_hit_jab_counts_all_three_contacts() {let mut s=Combo::new();assert_eq!(s.credit_contact(90),1);s.tick(true,90,16);assert_eq!(s.credit_contact(90),2);s.tick(true,90,16);assert_eq!(s.credit_contact(90),3);}
 #[test] fn multihits_count_even_on_same_frame() {let mut s=Combo::new();for hit in 1..=8 {assert_eq!(s.credit_contact(90),hit);}}
 #[test] fn no_contact_does_not_add_hits() {let mut s=Combo::new();s.credit_contact(90);for _ in 0..20 {s.tick(true,90,16);}assert_eq!(s.hits,1);}
 #[test] fn jab_credit_is_independent_of_player_slot() {for slot in 0..8 {let mut all=[Combo::new();8];all[slot].identify(slot as u32,slot+1);for hit in 1..=3 {assert_eq!(all[slot].credit_contact(90),hit);}for other in 0..8 {if other!=slot {assert_eq!(all[other].hits,0);}}}}
}
