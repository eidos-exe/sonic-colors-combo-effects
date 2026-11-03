#[derive(Clone,Copy)]
pub struct ThrowCredit {object:u32,motion:u64,active:bool,credited:bool}
impl ThrowCredit {
 pub const fn new()->Self {Self{object:u32::MAX,motion:0,active:false,credited:false}}
 pub fn observe(&mut self,object:u32,motion:u64,active:bool,ready:bool,contacts:u8)->u8 {
  if !active || !self.active || self.object!=object || self.motion!=motion {self.credited=false;}
  self.object=object;self.motion=motion;self.active=active;
  if !active {return contacts;}
  if !self.credited && (contacts>0 || ready) {self.credited=true;return 1;}
  0
 }
}
#[cfg(test)] mod tests {use super::*;
 #[test] fn fresh_throw_is_one_silent_hit_even_with_internal_contacts() {let mut s=ThrowCredit::new();let mut hits=0;hits+=s.observe(1,2,true,false,3);hits+=s.observe(1,2,true,true,0);hits+=s.observe(1,2,true,false,1);assert_eq!(hits,1);assert!(hits<2);}
 #[test] fn fallback_then_native_event_is_still_only_one_throw() {let mut s=ThrowCredit::new();assert_eq!(s.observe(1,2,true,true,0),1);assert_eq!(s.observe(1,2,true,false,1),0);}
 #[test] fn next_attack_or_item_after_throw_reaches_two() {let mut s=ThrowCredit::new();let throw=s.observe(1,2,true,false,1);let item=1;assert_eq!(throw+item,2);}
 #[test] fn next_throw_can_start_a_new_sequence() {let mut s=ThrowCredit::new();s.observe(1,2,true,true,0);s.observe(1,2,false,false,0);assert_eq!(s.observe(1,2,true,true,0),1);}
 #[test] fn ordinary_multihits_are_not_collapsed() {let mut s=ThrowCredit::new();assert_eq!(s.observe(1,2,false,false,3),3);}
 #[test] fn grab_and_hold_are_not_throw_hits() {let mut s=ThrowCredit::new();assert_eq!(s.observe(1,2,false,false,0),0);assert_eq!(s.observe(1,2,true,false,0),0);}
}
