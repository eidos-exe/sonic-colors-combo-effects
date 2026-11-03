#[derive(Clone,Copy)]
pub enum Source { Fighter, OwnedBy(u32) }
/// Follow current ownership only. Unknown, inactive, cyclic, or overly deep chains do not score.
pub fn fighter_owner(mut id:u32,mut source:impl FnMut(u32)->Option<Source>)->Option<u32> {
    let mut visited=[u32::MAX;4];
    for depth in 0..visited.len() {
        if visited[..depth].contains(&id) {return None;}
        visited[depth]=id;
        match source(id)? {Source::Fighter=>return Some(id),Source::OwnedBy(owner)=>id=owner}
    }
    None
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn fighter_and_article_still_resolve() {assert_eq!(fighter_owner(1,|id|Some(if id==1{Source::Fighter}else{Source::OwnedBy(1)})),Some(1));assert_eq!(fighter_owner(2,|id|Some(if id==2{Source::OwnedBy(1)}else{Source::Fighter})),Some(1));}
 #[test] fn item_uses_current_owner_instead_of_creator() {let thrower=7;assert_eq!(fighter_owner(20,|id|match id {20=>Some(Source::OwnedBy(thrower)),7=>Some(Source::Fighter),_=>None}),Some(7));}
 #[test] fn item_owned_article_can_reach_fighter() {assert_eq!(fighter_owner(30,|id|match id {30=>Some(Source::OwnedBy(20)),20=>Some(Source::OwnedBy(7)),7=>Some(Source::Fighter),_=>None}),Some(7));}
 #[test] fn unowned_inactive_and_cyclic_sources_do_not_score() {assert_eq!(fighter_owner(20,|_|None),None);assert_eq!(fighter_owner(20,|_|Some(Source::OwnedBy(20))),None);assert_eq!(fighter_owner(20,|id|Some(Source::OwnedBy(if id==20{21}else{20}))),None);assert_eq!(fighter_owner(20,|id|Some(Source::OwnedBy(id+1))),None);}
}
