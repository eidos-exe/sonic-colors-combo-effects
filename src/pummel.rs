#[derive(Clone,Copy)]
pub struct Pummel {object:u32,motion:u64,frame:f32,active:bool,credited:bool}
impl Pummel {
 pub const fn new()->Self {Self{object:u32::MAX,motion:0,frame:0.0,active:false,credited:false}}
 pub fn observe(&mut self,object:u32,motion:u64,frame:f32,active:bool) {if !active || !self.active || self.object!=object || self.motion!=motion || (frame<=1.0 && self.frame>1.5) {self.credited=false;}self.object=object;self.motion=motion;self.frame=frame;self.active=active;}
 pub fn claim(&mut self,damaging_hit:bool)->bool {if !self.active || !damaging_hit || self.credited {return false;}self.credited=true;true}
}
#[cfg(test)] mod tests {use super::*;
 #[test] fn grab_and_hold_do_not_score() {let mut p=Pummel::new();p.observe(1,2,3.0,false);assert!(!p.claim(true));p.observe(1,2,4.0,true);assert!(!p.claim(false));}
 #[test] fn pummel_scores_once_even_if_observed_twice() {let mut p=Pummel::new();p.observe(1,2,6.0,true);assert!(p.claim(true));assert!(!p.claim(true));p.observe(1,2,7.0,true);assert!(!p.claim(true));}
 #[test] fn next_pummel_and_other_fighter_score_separately() {let mut a=Pummel::new();let mut b=Pummel::new();a.observe(1,2,6.0,true);assert!(a.claim(true));a.observe(1,2,1.0,true);assert!(a.claim(true));b.observe(2,2,6.0,true);assert!(b.claim(true));}
}

#[cfg(test)] mod regression {use super::*;
 #[test] fn hitstop_rewind_is_not_another_pummel() {let mut p=Pummel::new();p.observe(1,2,6.0,true);assert!(p.claim(true));p.observe(1,2,5.0,true);assert!(!p.claim(true));p.observe(1,2,6.0,true);assert!(!p.claim(true));}
 #[test] fn first_pummel_does_not_reach_reward_threshold() {let mut hits=0;let mut p=Pummel::new();for frame in [6.0,5.0,6.0,7.0] {p.observe(1,2,frame,true);if p.claim(true) {hits+=1;}}assert_eq!(hits,1);assert!(hits<2);p.observe(1,2,0.0,true);p.observe(1,2,6.0,true);if p.claim(true) {hits+=1;}assert_eq!(hits,2);}
}
