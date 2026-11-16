//! Recorded conservative quiver packing, including forty-unit expansion bands.
use std::collections::{BTreeMap, BTreeSet};
use crate::borg_town_solve::{Choice, Group, GroupKey};
use crate::borg_town_stock::{Candidate, Stock};
use crate::headers::list_tvals_h::{TV_ARROW, TV_BOLT, TV_SHOT};
use crate::state::BorgState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Packing {
    pub block: i64,
    pub used: i64,
    /// Ammo class and room in its EXISTING compatible quiver stacks.
    pub classes: Vec<[i64; 2]>,
}
fn ammo(tval: u8) -> bool { matches!(u32::from(tval), TV_ARROW | TV_BOLT | TV_SHOT) }

pub fn quiver_units(st: &BorgState) -> i64 {
    let start = crate::borg_item::quiver_start() as usize;
    let mult = i64::from(crate::engine::z_info().thrown_quiver_mult).max(1);
    st.items.items.iter().skip(start).filter(|i| i.iqty > 0)
        .map(|i| i64::from(i.iqty) * if ammo(i.tval) { 1 } else { mult }).sum()
}
pub fn reserved(st: &BorgState) -> i32 {
    let block = i64::from(crate::engine::z_info().quiver_slot_size).max(1);
    (quiver_units(st) as u64).div_ceil(block as u64) as i32
}
impl Packing {
    pub fn of(st: &BorgState, stock: &Stock) -> Self {
        let block = i64::from(crate::engine::z_info().quiver_slot_size).max(1);
        let mut classes = BTreeMap::<u16, i64>::new();
        for c in stock.candidates.iter().filter(|c| ammo(c.tval)) {
            let room = crate::engine::store_stock(&st.access, usize::from(c.shop))
                .get(usize::from(c.slot)).map_or(0, |o| i64::from(o.quiver_stack_room()));
            if room == 0 && c.slots_cost == 0 { continue; }
            classes.entry(c.class).and_modify(|v| *v = (*v).min(room)).or_insert(room);
        }
        Self { block, used: quiver_units(st), classes: classes.into_iter().map(|(c,n)| [i64::from(c),n]).collect() }
    }
    pub fn valid(&self) -> bool {
        self.block > 0 && self.used >= 0 && self.classes.iter().all(|r| (0..=u16::MAX as i64).contains(&r[0]) && r[1] >= 0)
    }
    pub fn room(&self, class: u16) -> Option<i64> { self.classes.iter().find(|r| r[0] == i64::from(class)).map(|r| r[1]) }
    /// Do not model multiple new quiver blocks as one pack slot. Existing
    /// joins are bounded by their real capacity; other ammo classes are
    /// conservatively bounded to one block and charged one slot. Live
    /// execution buys one then refreshes these facts.
    pub fn bound(&self, groups: Vec<Group>, candidates: &[Candidate]) -> Vec<Group> {
        groups.into_iter().map(|mut g| {
            if let GroupKey::Class(class) = g.key {
                if let Some(room) = self.room(class) {
                    let max = if room > 0 { room } else { self.block };
                    g.options.retain(|o| o.picks.iter().map(|&(i,n)| {
                        debug_assert_eq!(candidates[i].class, class); i64::from(n)
                    }).sum::<i64>() <= max);
                }
            }
            g
        }).collect()
    }
    pub fn slots(&self, picks: &[(usize, i32)], candidates: &[Candidate]) -> i32 {
        let mut ordinary = BTreeSet::new();
        let mut joined = 0;
        for &(i,n) in picks {
            let c = &candidates[i];
            match self.room(c.class) {
                Some(room) if room > 0 => joined += i64::from(n),
                Some(_) => { ordinary.insert(c.class); },
                None if c.slots_cost > 0 => { ordinary.insert(c.class); },
                None => {},
            }
        }
        let before = (self.used as u64).div_ceil(self.block as u64);
        let after = ((self.used + joined) as u64).div_ceil(self.block as u64);
        ordinary.len() as i32 + (after - before) as i32
    }
    pub fn reprice(&self, choice: &mut Choice, candidates: &[Candidate]) { choice.slots = self.slots(&choice.picks, candidates); }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn c(class: u16) -> Candidate { Candidate { shop:0,slot:0,kind:1,tval:TV_ARROW as u8,sval:0,price:1,qty:99,weight:2,slots_cost:1,stack_count:0,class } }
    #[test]
    fn joining_quiver_is_free_until_native_pack_expansion_boundary() {
        let p=Packing {block:40,used:2,classes:vec![[0,38]]};let cs=[c(0)];
        assert_eq!(p.slots(&[(0,1)],&cs),0);
        assert_eq!(p.slots(&[(0,38)],&cs),0);
        let p=Packing {block:40,used:39,classes:vec![[0,20]]};
        assert_eq!(p.slots(&[(0,1)],&cs),0);assert_eq!(p.slots(&[(0,2)],&cs),1);
    }
    #[test]
    fn different_ammunition_kinds_share_reserved_quiver_room_once() {
        let p=Packing {block:40,used:39,classes:vec![[0,10],[1,10]]};let cs=[c(0),c(1)];
        assert_eq!(p.slots(&[(0,1),(1,1)],&cs),1);
        let p=Packing {block:40,used:40,classes:vec![[0,0]]};assert_eq!(p.slots(&[(0,40)],&cs),1);
    }
}
