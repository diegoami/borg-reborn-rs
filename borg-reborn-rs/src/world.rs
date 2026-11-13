//! L2: the world model (ARCHITECTURE.md L2; task T2.2, #13).
//!
//! What the Borg remembers between thinks: the map of the current level,
//! a track for every monster it has seen there, the objects it has seen on
//! the floor, the town's shops, and a character model derived from its gear.
//! It changes only in [`World::update`], once per think, from an
//! [`Observation`] (GOALS principle 10). Strategies get `&World`, and a
//! "try this item on" question runs on a cloned [`Character`]
//! ([`Character::wearing`]), so no decision can change what the Borg knows
//! (spec §5.4, §11.1, §11.6).
//!
//! Everything here is plain data built only from observations, so replaying
//! a recorded observation log gives the same world as live play (the T2.2
//! exit check, [`World::digest`]).

use crate::knowledge::{self, Knowledge};
use crate::observation::{Grid, Item, MonsterSeen, Observation, Slot, Status, StoreStock};
use serde::{Deserialize, Serialize};

/// `MON_TMD_SLEEP` (list-mon-timed.h).
const MON_TMD_SLEEP: usize = 0;
/// `ELEM_MAX` (list-elements.h).
pub const ELEM_MAX: usize = 25;
/// `EQUIP_WEAPON` (list-equip-slots.h).
/// `TV_*` values: `list-tvals.h` numbers its `TV(...)` entries from
/// `TV_NULL` = 0 (T5.2, #126). The observation carries the engine's own
/// tval, so these, not the pre-4.2 numbers (sword 23), are what an item
/// says it is.
pub mod tv {
    pub const SHOT: u8 = 2;
    pub const ARROW: u8 = 3;
    pub const BOLT: u8 = 4;
    pub const BOW: u8 = 5;
    pub const DIGGING: u8 = 6;
    pub const HAFTED: u8 = 7;
    pub const POLEARM: u8 = 8;
    pub const SWORD: u8 = 9;
    pub const BOOTS: u8 = 10;
    pub const GLOVES: u8 = 11;
    pub const HELM: u8 = 12;
    pub const CROWN: u8 = 13;
    pub const SHIELD: u8 = 14;
    pub const CLOAK: u8 = 15;
    pub const SOFT_ARMOR: u8 = 16;
    pub const HARD_ARMOR: u8 = 17;
    pub const DRAG_ARMOR: u8 = 18;
    pub const LIGHT: u8 = 19;
    pub const AMULET: u8 = 20;
    pub const RING: u8 = 21;
    pub const SCROLL: u8 = 25;
    pub const POTION: u8 = 26;
    pub const FLASK: u8 = 27;
    pub const FOOD: u8 = 28;
    pub const MUSHROOM: u8 = 29;

    /// A melee weapon: hafted, polearm or sword (a digger digs; a bow
    /// shoots).
    pub fn is_melee(tval: u8) -> bool {
        matches!(tval, HAFTED | POLEARM | SWORD)
    }

    /// Body, cloak, shield, helm, gloves or boots: what `calc_mana` weighs
    /// (`player-calcs.c:1505-1525`).
    pub fn is_armour(tval: u8) -> bool {
        matches!(
            tval,
            BOOTS | GLOVES | HELM | CROWN | SHIELD | CLOAK | SOFT_ARMOR | HARD_ARMOR | DRAG_ARMOR
        )
    }
}

pub const EQUIP_WEAPON: u16 = 1;
/// `EQUIP_BOW` (list-equip-slots.h).
pub const EQUIP_BOW: u16 = 2;
/// `OBJ_MOD_STR` .. `OBJ_MOD_CON` (list-object-modifiers.h, from
/// list-stats.h): the first five modifiers are the stats, in `stat_use`'s
/// order.
pub const STAT_MAX: usize = 5;
/// `OBJ_MOD_BLOWS` (list-object-modifiers.h).
pub const OBJ_MOD_BLOWS: usize = 10;
/// `TMD_BLOODLUST` (list-player-timed.h).
pub const TMD_BLOODLUST: usize = 49;

/// Everything the Borg remembers.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct World {
    /// The game turn of the last observation.
    pub turn: i64,
    /// Observations taken in total.
    pub thinks: u64,
    pub level: Level,
    /// The town's shop entrances, kept across levels.
    pub shops: Vec<Shop>,
    /// Every store's stock and the home's, as last read. The C Borg reads
    /// them on every think, wherever it is (`borg_notice_store`).
    pub stock: Vec<StoreStock>,
    pub character: Character,
}

/// The current level, forgotten when the Borg changes level.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Level {
    pub depth: i32,
    pub width: i32,
    pub height: i32,
    /// Which generation this is, increasing on every level change. Unlike
    /// `arrived`, it is unique even when stairs change level without moving
    /// the game turn (the M3 review's reroll finding).
    pub generation: u64,
    /// The game turn the Borg arrived.
    pub arrived: i64,
    /// Row by row, `width * height` grids.
    pub grids: Vec<Remembered>,
    /// Every monster seen on this level, by the game's monster index.
    pub monsters: Vec<Track>,
}

/// What the Borg remembers of one grid: the last thing `map_info()` showed
/// for it. The C Borg's grid record without its panel artefacts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remembered {
    /// The remembered feature; 0 (`FEAT_NONE`) if never known.
    pub feat: u32,
    /// The last game turn the grid was in view, if ever.
    pub seen: Option<i64>,
    /// The kind of the first object the player remembers here.
    pub object_kind: Option<u32>,
    /// The real kind of the top object that is not ignored, from the last
    /// time the Borg saw an object here.
    pub top_kind: Option<u32>,
    pub trap: bool,
    pub webbed: bool,
}

/// A monster the Borg has seen. Out of view it is not forgotten: the track
/// keeps where and when it was last seen, and [`Track::confidence`] says
/// how much that still means (REDESIGN-LESSONS L11; better-borg-c D0014).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    /// The game's monster index.
    pub index: i32,
    pub race: u32,
    pub x: i32,
    pub y: i32,
    pub first_seen: i64,
    pub last_seen: i64,
    /// Seen in the latest observation.
    pub in_view: bool,
    pub hp: i32,
    pub max_hp: i32,
    pub timed: [i16; 10],
    pub speed: u8,
}

impl Track {
    pub fn awake(&self) -> bool {
        self.timed[MON_TMD_SLEEP] == 0
    }

    /// How sure the Borg can be that the monster is still where it was
    /// seen: 1 while in view, then falling with the moves it may have made
    /// since, from the game's energy table at its speed
    /// ([`Knowledge::turns_per_action`]). A sleeping monster counts a
    /// quarter of its moves. The shape is a parameter for M4's
    /// calibration; this one only has to fall with time and speed.
    pub fn confidence(&self, now: i64, k: &Knowledge) -> f64 {
        if self.in_view {
            return 1.0;
        }
        let elapsed = (now - self.last_seen).max(0) as f64;
        let moves = elapsed / k.turns_per_action(i32::from(self.speed));
        1.0 / (1.0 + moves * if self.awake() { 1.0 } else { 0.25 })
    }
}

/// A shop entrance in town, and when the Borg was last inside.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shop {
    /// `square_shopnum`: 0 General Store … 7 Home (store.txt order).
    pub number: i32,
    pub x: i32,
    pub y: i32,
    pub last_visit: Option<i64>,
}

/// The character, derived from the latest status and gear. Cloned, it is
/// the model a "try this on" question changes ([`Character::wearing`]).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Character {
    pub status: Option<Status>,
    pub equipment: Vec<Slot>,
    pub quiver: Vec<Item>,
    pub inventory: Vec<Item>,
    /// Derived from the known part of the equipment.
    pub gear: GearBonuses,
}

/// What the equipment adds up to, from what the player knows of it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GearBonuses {
    /// To-hit and to-dam of the equipment but the weapon and the launcher,
    /// as `calc_bonuses` adds them to `state.to_h` and `state.to_d`
    /// (`player-calcs.c:1999-2006`): the weapon's own bonuses go to its
    /// blows, the launcher's to its shots.
    pub to_h: i32,
    pub to_d: i32,
    /// Base armour plus its bonuses.
    pub ac: i32,
    /// Known object flags of all equipped items, OR-ed (`OF_*` bits).
    pub flags: [u8; 5],
    /// Resistance level per element, as `calc_bonuses` combines them:
    /// the best level any item gives, one lower when any item makes the
    /// element a vulnerability and nothing gives immunity
    /// (`player-calcs.c:calc_bonuses:1985-2037`). Race and class
    /// resistances are not included.
    pub resist: Vec<i16>,
    /// Known modifiers (`OBJ_MOD_*`), summed.
    pub modifiers: Vec<i32>,
}

impl GearBonuses {
    pub fn of(equipment: &[Slot]) -> Self {
        let mut g = GearBonuses {
            resist: vec![0; ELEM_MAX],
            ..Default::default()
        };
        let mut vulnerable = [false; ELEM_MAX];
        for (slot, item) in equipment
            .iter()
            .filter_map(|s| s.item.as_ref().map(|i| (s.slot_type, i)))
        {
            if slot != EQUIP_WEAPON && slot != EQUIP_BOW {
                g.to_h += i32::from(item.to_h);
                g.to_d += i32::from(item.to_d);
            }
            g.ac += i32::from(item.ac) + i32::from(item.to_a);
            for (f, b) in g.flags.iter_mut().zip(item.flags) {
                *f |= b;
            }
            for (i, &(level, _)) in item.elements.iter().enumerate().take(ELEM_MAX) {
                if level == -1 {
                    vulnerable[i] = true;
                } else if level > g.resist[i] {
                    g.resist[i] = level;
                }
            }
            if g.modifiers.len() < item.modifiers.len() {
                g.modifiers.resize(item.modifiers.len(), 0);
            }
            for (m, &v) in g.modifiers.iter_mut().zip(&item.modifiers) {
                *m += i32::from(v);
            }
        }
        for (r, v) in g.resist.iter_mut().zip(vulnerable) {
            if v && *r < 3 {
                *r -= 1;
            }
        }
        g
    }
}

impl Character {
    fn observe(&mut self, o: &Observation) {
        self.status = Some(o.status.clone());
        self.equipment = o.equipment.clone();
        self.quiver = o.quiver.clone();
        self.inventory = o.inventory.clone();
        self.gear = GearBonuses::of(&self.equipment);
    }

    /// The same character with `item` in equipment slot `slot` (an index
    /// into `equipment`) instead of what is there: a model to compare
    /// equipment by effect. The world is not touched.
    ///
    /// The stats in use follow the known stat modifiers of the swap, as
    /// `calc_bonuses` recomputes them (`player-calcs.c:2058-2061`:
    /// `stat_use = modify_stat_value(stat_cur, add)`). `add` sums the race's
    /// and class's adjustments, the gear's modifiers and what the Borg
    /// cannot see (a shape, unknown runes). It is found from the observed
    /// `stat_use`: exactly, except at the floor of 3, where a whole range of
    /// penalties gives 3. There the bonus nearest the known one (race,
    /// class and the gear's known modifiers) is taken, which is the
    /// engine's whenever nothing unseen changes the stat (M2 reviews, R3
    /// and the third review's 4).
    pub fn wearing(&self, k: &Knowledge, slot: usize, item: Option<Item>) -> Character {
        let mut c = self.clone();
        if let Some(s) = c.equipment.get_mut(slot) {
            s.item = item;
        }
        c.gear = GearBonuses::of(&c.equipment);
        if let Some(s) = c.status.as_mut() {
            let modifier = |g: &GearBonuses, i: usize| g.modifiers.get(i).copied().unwrap_or(0);
            let race = k.player_race(s.race);
            let class = k.class(s.class);
            for i in 0..STAT_MAX {
                let delta = modifier(&c.gear, i) - modifier(&self.gear, i);
                if delta == 0 {
                    continue;
                }
                let known = race.map_or(0, |r| r.stat_adj[i])
                    + class.map_or(0, |c| c.stat_adj[i])
                    + modifier(&self.gear, i);
                let (cur, used) = (i32::from(s.stat_cur[i]), s.stat_use[i]);
                let Some(add) = (-60..=60)
                    .filter(|&a| knowledge::modify_stat_value(cur, a) == used)
                    .min_by_key(|&a| ((a - known).abs(), a))
                else {
                    continue;
                };
                s.stat_use[i] = knowledge::modify_stat_value(cur, add + delta);
                s.stat_ind[i] = knowledge::stat_index(s.stat_use[i]) as i32;
            }
        }
        c
    }

    /// The wielded weapon, if any.
    pub fn weapon(&self) -> Option<&Item> {
        self.equipment
            .iter()
            .find(|s| s.slot_type == EQUIP_WEAPON)
            .and_then(|s| s.item.as_ref())
    }

    /// Blows per round times 100 with the current gear, as the engine sets
    /// `state.num_blows` (`player-calcs.c:calc_bonuses:2296-2318`): a
    /// weapon too heavy to wield ([`knowledge::heavy_wield`]) leaves one blow;
    /// otherwise [`knowledge::blows`] with the stats in use, the known
    /// `OBJ_MOD_BLOWS` of the equipment and bloodlust's `timed / 20`
    /// (`:2207-2209`). Shapechanges (`calc_shapechange`) are not modelled.
    pub fn blows(&self, k: &Knowledge) -> Option<i32> {
        let s = self.status.as_ref()?;
        let class = k.class(s.class)?;
        let str_index = knowledge::stat_index(s.stat_use[0]);
        let weight = self.weapon().map_or(0, |w| i32::from(w.weight));
        if self.weapon().is_some() && knowledge::heavy_wield(str_index, weight) {
            return Some(100);
        }
        let bloodlust = s.timed.get(TMD_BLOODLUST).map_or(0, |&t| i32::from(t) / 20);
        let extra = self.gear.modifiers.get(OBJ_MOD_BLOWS).copied().unwrap_or(0) + bloodlust;
        Some(knowledge::blows(
            class,
            str_index,
            knowledge::stat_index(s.stat_use[3]),
            weight,
            extra,
            s.percent_damage,
        ))
    }

    /// Expected melee damage per player turn against a monster with no
    /// resistances, if every blow hits: blows times the weapon's mean dice,
    /// plus its own known to-dam (`melee_damage`, `player-attack.c:480-492`)
    /// and the known to-dam of the other gear but the launcher
    /// (`state.to_d`, `player_damage_bonus`, `:825-828`). Without a weapon,
    /// a blow does 1. Hit chance, criticals, slays, brands and the
    /// percent-damage birth option come with the threat model (M4). Compare
    /// weapons by trying them on a copy:
    /// `character.wearing(k, slot, Some(item)).melee_per_turn(k)`.
    pub fn melee_per_turn(&self, k: &Knowledge) -> Option<f64> {
        let blows = f64::from(self.blows(k)?) / 100.0;
        let per_blow = self.weapon().map_or(1.0, |w| {
            f64::from(w.dd) * (f64::from(w.ds) + 1.0) / 2.0 + f64::from(w.to_d)
        });
        Some(blows * (per_blow + f64::from(self.gear.to_d)).max(0.0))
    }

    /// Free pack slots, by the engine's own count (quiver included).
    pub fn pack_free(&self) -> i32 {
        self.status
            .as_ref()
            .map_or(0, |s| s.pack_size - s.pack_used)
    }
}

impl Level {
    fn new(o: &Observation, generation: u64) -> Self {
        let (w, h) = (o.panel.level_width.max(0), o.panel.level_height.max(0));
        Level {
            depth: o.status.depth,
            width: w,
            height: h,
            generation,
            arrived: o.turn,
            grids: vec![Remembered::default(); (w * h) as usize],
            monsters: Vec::new(),
        }
    }

    pub fn at(&self, x: i32, y: i32) -> Option<&Remembered> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        self.grids.get((y * self.width + x) as usize)
    }

    fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut Remembered> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        self.grids.get_mut((y * self.width + x) as usize)
    }

    /// The floor objects the Borg remembers: (x, y, real kind).
    pub fn objects(&self) -> impl Iterator<Item = (i32, i32, u32)> + '_ {
        self.grids.iter().enumerate().filter_map(move |(i, g)| {
            let (x, y) = (i as i32 % self.width, i as i32 / self.width);
            g.top_kind.map(|k| (x, y, k))
        })
    }

    pub fn track(&self, index: i32) -> Option<&Track> {
        self.monsters.iter().find(|t| t.index == index)
    }

    fn see(&mut self, x: i32, y: i32, grid: &Grid, turn: i64) {
        let Some(r) = self.at_mut(x, y) else {
            return;
        };
        // map_info() reports the player's own memory (`g->f_idx =
        // square(player->cave, grid)->feat`, cave-map.c:100-101), and
        // that memory is mutable: at nightfall in town every boring
        // floor is forgotten (`square_set_known_feat(FEAT_NONE)` via
        // `cave_illuminate`, cave-map.c:581-585, game-world.c:478-492),
        // and a wall bump forgets what it disproves (`square_forget`,
        // cmd-cave.c). A feature 0 therefore always means the player's
        // memory of the grid is empty, and the world follows it —
        // holding a private map the player no longer has is how T3.4's
        // CI run diverged 2.5 million grids over one town night.
        r.feat = grid.feat;
        if grid.in_view {
            r.seen = Some(turn);
        }
        r.object_kind = grid.object_kind;
        if grid.object_kind.is_none() {
            r.top_kind = None;
        }
        r.trap = grid.trap;
        r.webbed = grid.webbed;
    }

    fn sight(&mut self, m: &MonsterSeen, turn: i64) {
        let fresh = Track {
            index: m.index,
            race: m.race,
            x: m.x,
            y: m.y,
            first_seen: turn,
            last_seen: turn,
            in_view: true,
            hp: m.hp,
            max_hp: m.max_hp,
            timed: m.timed,
            speed: m.speed,
        };
        match self.monsters.iter_mut().find(|t| t.index == m.index) {
            // The game reuses a dead monster's index: another race there
            // is another monster.
            Some(t) if t.race == m.race => {
                *t = Track {
                    first_seen: t.first_seen,
                    ..fresh
                }
            }
            Some(t) => *t = fresh,
            None => self.monsters.push(fresh),
        }
    }
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take in one observation. The only way the world changes.
    pub fn update(&mut self, o: &Observation) {
        // A new level: the depth or the level's size changed (every way
        // to leave a level in 4.2 changes the depth), or nothing is known
        // yet. The old level is forgotten, as the C Borg forgets it.
        if self.thinks == 0
            || o.status.depth != self.level.depth
            || o.panel.level_width != self.level.width
            || o.panel.level_height != self.level.height
        {
            self.level = Level::new(o, self.level.generation.saturating_add(1));
        }
        self.turn = o.turn;
        self.thinks += 1;

        let p = &o.panel;
        for dy in 0..p.height {
            for dx in 0..p.width {
                if let Some(g) = p.grids.get((dy * p.width + dx) as usize) {
                    self.level.see(p.x0 + dx, p.y0 + dy, g, o.turn);
                }
            }
        }
        for seen in &o.objects {
            if let Some(r) = self.level.at_mut(seen.x, seen.y) {
                r.top_kind = seen.kind;
            }
        }

        for t in &mut self.level.monsters {
            t.in_view = false;
        }
        for m in &o.monsters {
            self.level.sight(m, o.turn);
        }

        if o.status.depth == 0 {
            self.note_shops(o);
        }
        self.stock = o.stores.clone();
        self.character.observe(o);
    }

    fn note_shops(&mut self, o: &Observation) {
        let p = &o.panel;
        for dy in 0..p.height {
            for dx in 0..p.width {
                let Some(g) = p.grids.get((dy * p.width + dx) as usize) else {
                    continue;
                };
                if g.shop < 0 {
                    continue;
                }
                let (x, y) = (p.x0 + dx, p.y0 + dy);
                match self.shops.iter_mut().find(|s| s.number == g.shop) {
                    Some(s) => (s.x, s.y) = (x, y),
                    None => self.shops.push(Shop {
                        number: g.shop,
                        x,
                        y,
                        last_visit: None,
                    }),
                }
            }
        }
        self.shops.sort_by_key(|s| s.number);
        // In a shop the store header reads "Stor" or "Home"
        // (borg-think.c:borg_think:221-223) and the Borg stands on its
        // entrance.
        if matches!(o.store_header.as_str(), "Stor" | "Home") {
            let (x, y) = (o.status.x, o.status.y);
            if let Some(s) = self.shops.iter_mut().find(|s| (s.x, s.y) == (x, y)) {
                s.last_visit = Some(o.turn);
            }
        }
    }

    /// A short fingerprint of the whole world (FNV-1a over its JSON), to
    /// compare replayed and live state think by think.
    pub fn digest(&self) -> String {
        let json = serde_json::to_string(self).expect("the world is plain data");
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in json.bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{h:016x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::tests::sample;
    use crate::observation::ObjectSeen;

    /// Knowledge with the game's energy table around normal speed and the
    /// Warrior class.
    fn energy() -> Knowledge {
        let mut e = vec![1u8; 200];
        for (s, v) in [(110, 10), (120, 20), (130, 30)] {
            e[s] = v;
        }
        Knowledge {
            extract_energy: e,
            move_energy: 100,
            classes: vec![crate::knowledge::tests::warrior()],
            ..Default::default()
        }
    }

    #[test]
    fn blows_and_melee_come_from_the_gear() {
        let o = sample();
        let w = world_of(&o);
        let k = energy();
        // STR 17, DEX 14 (stat_use), a 12 lb dagger (weight 120): the
        // hand-worked C value of knowledge::blows.
        assert_eq!(w.character.blows(&k), Some(100));
        assert_eq!(w.character.melee_per_turn(&k), Some(2.5), "1d4: 2.5 a blow");
        let mut better = w.character.equipment[0].item.clone().unwrap();
        better.dd = 2;
        better.ds = 5;
        better.to_d = 3;
        let tried = w.character.wearing(&k, 0, Some(better));
        assert_eq!(
            tried.melee_per_turn(&k),
            Some(9.0),
            "2d5+3 is compared by effect"
        );
        assert_eq!(
            w.character.melee_per_turn(&k),
            Some(2.5),
            "the world is unchanged"
        );
        // A 30 lb weapon at STR 17 (hold 28 lb) is too heavy: one blow.
        let mut heavy = w.character.equipment[0].item.clone().unwrap();
        heavy.weight = 300;
        assert_eq!(w.character.wearing(&k, 0, Some(heavy)).blows(&k), Some(100));
    }

    /// M2 review, finding 6: a launcher's to-dam is for its shots, the
    /// weapon's for its blows, and the rest of the gear's for every blow;
    /// a stat ring changes the stats in use and so the blows.
    #[test]
    fn to_dam_and_stats_follow_the_engine() {
        let o = sample();
        let w = world_of(&o);
        let k = energy();
        let mut c = w.character.clone();
        c.equipment.push(Slot {
            slot_type: 3, // EQUIP_RING
            item: None,
        });
        let mut bow = c.equipment[0].item.clone().unwrap();
        bow.dd = 0;
        bow.ds = 0;
        bow.to_d = 10;
        assert_eq!(
            c.wearing(&k, 1, Some(bow)).melee_per_turn(&k),
            Some(2.5),
            "a (+0,+10) bow adds nothing to melee"
        );
        let mut ring = c.equipment[0].item.clone().unwrap();
        ring.to_d = 2;
        ring.modifiers = vec![0; 13];
        assert_eq!(
            c.wearing(&k, 2, Some(ring.clone())).melee_per_turn(&k),
            Some(4.5),
            "a ring of damage adds to every blow"
        );
        let mut weapon = c.equipment[0].item.clone().unwrap();
        weapon.to_d = 4;
        assert_eq!(c.wearing(&k, 0, Some(weapon)).melee_per_turn(&k), Some(6.5));
        ring.to_d = 0;
        ring.modifiers[0] = 3; // STR +3
        let strong = c.wearing(&k, 2, Some(ring));
        let s = strong.status.as_ref().unwrap();
        assert_eq!(s.stat_use[0], 18 + 20, "17 +3 is 18/20");
        assert_eq!(s.stat_ind[0], 17);
        assert_eq!(c.status.as_ref().unwrap().stat_use[0], 17, "the copy only");
        let off = strong.wearing(&k, 2, None);
        assert_eq!(
            off.status.as_ref().unwrap().stat_use[0],
            17,
            "taking it off undoes it"
        );

        // The reviewer's case (R3): STR 3 at the floor, held there by a
        // -5 item. Taking it off leaves 3 in the engine (the bonus in force
        // was the floor's, not zero); the model must not invent STR 8.
        let mut weak = c.clone();
        let mut cursed = c.equipment[0].item.clone().unwrap();
        cursed.modifiers = vec![0; 13];
        cursed.modifiers[0] = -5;
        weak.equipment[2].item = Some(cursed);
        weak.gear = GearBonuses::of(&weak.equipment);
        if let Some(s) = weak.status.as_mut() {
            s.stat_cur[0] = 3;
            s.stat_use[0] = 3;
        }
        let removed = weak.wearing(&k, 2, None);
        assert_eq!(
            removed.status.as_ref().unwrap().stat_use[0],
            3,
            "no stat is invented at the floor"
        );
    }

    /// M2 third review, 4: at the floor of 3, the bonus in force is the
    /// one nearest the known bonus (race, class, the gear's known
    /// modifiers), which is the engine's when nothing unseen changes it.
    #[test]
    fn the_stat_floor_follows_the_known_bonus() {
        let o = sample();
        let w = world_of(&o);
        let mut k = energy();
        k.classes[0].stat_adj = [0; 5];
        let mut c = w.character.clone();
        c.equipment.push(Slot {
            slot_type: 3,
            item: None,
        });
        let ring = |str_mod: i16| {
            let mut r = c.equipment[0].item.clone().unwrap();
            r.modifiers = vec![0; 13];
            r.modifiers[0] = str_mod;
            r
        };
        let at = |c: &Character, cur: i16, used: i32, worn: Option<Item>| {
            let mut c = c.clone();
            c.equipment[2].item = worn;
            c.gear = GearBonuses::of(&c.equipment);
            let s = c.status.as_mut().unwrap();
            s.stat_cur[0] = cur;
            s.stat_use[0] = used;
            c
        };
        let str_of = |c: &Character| c.status.as_ref().unwrap().stat_use[0];
        // STR 3, no bonus at all: a +5 ring gives 8, as in the engine.
        assert_eq!(str_of(&at(&c, 3, 3, None).wearing(&k, 2, Some(ring(5)))), 8);
        // STR 4 held at 3 by a -5 ring: taking it off gives 4 back.
        assert_eq!(
            str_of(&at(&c, 4, 3, Some(ring(-5))).wearing(&k, 2, None)),
            4
        );
        // STR 3 held at 3 by a -5 ring: taking it off leaves 3.
        assert_eq!(
            str_of(&at(&c, 3, 3, Some(ring(-5))).wearing(&k, 2, None)),
            3
        );
    }

    fn world_of(o: &Observation) -> World {
        let mut w = World::new();
        w.update(o);
        w
    }

    #[test]
    fn the_panel_is_remembered_in_level_coordinates() {
        let o = sample();
        let w = world_of(&o);
        assert_eq!((w.level.width, w.level.height), (198, 66));
        assert_eq!(w.level.at(10, 20).unwrap().feat, 1);
        assert_eq!(w.level.at(10, 20).unwrap().seen, Some(12345));
        assert_eq!(w.level.at(0, 0).unwrap().feat, 0, "never seen");
        assert_eq!(w.level.objects().collect::<Vec<_>>(), vec![(12, 21, 300)]);
    }

    #[test]
    fn a_forgotten_grid_erases_to_follow_the_player() {
        // The player's memory is what map_info reports, and it is
        // mutable: a forgotten grid (nightfall in town forgets every
        // boring floor, cave-map.c:581-585; a wall bump forgets what it
        // disproves) reports feature 0, and the world follows. The
        // world never holds a map the player no longer has.
        let mut o = sample();
        let mut w = world_of(&o);
        o.turn += 10;
        o.panel.grids[0].feat = 0;
        o.panel.grids[0].in_view = false;
        w.update(&o);
        let g = w.level.at(10, 20).unwrap();
        assert_eq!((g.feat, g.seen), (0, Some(12345)), "forgotten, once seen");
    }

    #[test]
    fn a_monster_out_of_view_is_tracked_with_falling_confidence() {
        let mut o = sample();
        let mut w = world_of(&o);
        assert!(w.level.track(7).unwrap().in_view);
        o.monsters.clear();
        o.turn += 100;
        w.update(&o);
        let t = w.level.track(7).expect("not forgotten");
        assert!(!t.in_view);
        let k = energy();
        let (soon, later) = (t.confidence(o.turn, &k), t.confidence(o.turn + 500, &k));
        assert!(soon < 1.0 && later < soon, "{soon} {later}");
        let mut fast = t.clone();
        fast.speed = 130;
        assert!(
            fast.confidence(o.turn, &k) < soon,
            "a fast monster is less certain"
        );
        let mut asleep = t.clone();
        asleep.timed[MON_TMD_SLEEP] = 50;
        assert!(
            asleep.confidence(o.turn, &k) > soon,
            "a sleeping monster stays put"
        );
    }

    #[test]
    fn a_reused_monster_index_is_a_new_monster() {
        let mut o = sample();
        let mut w = world_of(&o);
        o.turn += 50;
        o.monsters[0].race = 99;
        w.update(&o);
        let t = w.level.track(7).unwrap();
        assert_eq!((t.race, t.first_seen), (99, o.turn));
        assert_eq!(w.level.monsters.len(), 1);
    }

    #[test]
    fn a_new_level_is_a_fresh_map() {
        let mut o = sample();
        let mut w = world_of(&o);
        let generation = w.level.generation;
        o.status.depth = 2;
        o.turn += 10;
        o.monsters.clear();
        o.objects.clear();
        o.panel.grids[5].object_kind = None;
        w.update(&o);
        assert_eq!(w.level.depth, 2);
        assert_eq!(w.level.generation, generation + 1);
        assert_eq!(w.level.arrived, o.turn);
        assert!(w.level.monsters.is_empty());
        assert_eq!(w.level.objects().count(), 0);
    }

    #[test]
    fn an_object_seen_gone_is_forgotten() {
        let mut o = sample();
        let mut w = world_of(&o);
        o.turn += 10;
        o.panel.grids[5].object_kind = None;
        o.objects = vec![];
        w.update(&o);
        assert_eq!(w.level.objects().count(), 0);
        o.panel.grids[5].object_kind = Some(301);
        o.objects = vec![ObjectSeen {
            x: 12,
            y: 21,
            kind: None,
        }];
        w.update(&o);
        assert_eq!(w.level.at(12, 21).unwrap().object_kind, Some(301));
        assert_eq!(w.level.objects().count(), 0, "only ignored objects there");
    }

    #[test]
    fn shops_are_found_in_town_and_visits_noted() {
        let mut o = sample();
        o.status.depth = 0;
        o.panel.grids[2].shop = 7;
        o.panel.grids[2].feat = 14;
        let mut w = world_of(&o);
        assert_eq!(w.shops.len(), 1);
        assert_eq!((w.shops[0].number, w.shops[0].x, w.shops[0].y), (7, 12, 20));
        assert_eq!(w.shops[0].last_visit, None);
        o.turn += 10;
        (o.status.x, o.status.y) = (12, 20);
        o.store_header = "Home".into();
        w.update(&o);
        assert_eq!(w.shops[0].last_visit, Some(o.turn));
        o.status.depth = 1;
        o.turn += 10;
        w.update(&o);
        assert_eq!(w.shops.len(), 1, "the town is remembered below");
        assert_eq!(
            w.stock[0].wares[0].price, 25,
            "the stock is the latest read"
        );
    }

    #[test]
    fn gear_combines_as_the_engine_does_and_wearing_is_a_copy() {
        let o = sample();
        let w = world_of(&o);
        let c = &w.character;
        assert_eq!(c.gear.to_h, 0);
        assert_eq!(c.pack_free(), 22);
        let mut cloak = c.equipment[0].item.clone().unwrap();
        cloak.to_a = 3;
        cloak.ac = 1;
        cloak.elements[3] = (1, 0); // resists
        let mut cursed = cloak.clone();
        cursed.elements = vec![(0, 0); ELEM_MAX];
        cursed.elements[3] = (-1, 0);
        cursed.elements[4] = (-1, 0);
        let k = energy();
        let tried = c.wearing(&k, 1, Some(cloak)).wearing(&k, 0, Some(cursed));
        assert_eq!(tried.gear.ac, 1 + 3 + 1 + 3);
        assert_eq!(tried.gear.resist[3], 0, "resist and vulnerability cancel");
        assert_eq!(tried.gear.resist[4], -1, "vulnerable");
        assert_eq!(w.character.gear.ac, 0, "the world's character is unchanged");
    }

    /// The T2.2 exit check: replaying a recorded observation log into a
    /// fresh world, without the game, gives the live world's state after
    /// every think. CI records both logs (`BORG_OBSERVATION_LOG`,
    /// `BORG_WORLD_LOG`) in one run and points the `_CHECK` variables at
    /// them; without them this test has nothing to read.
    #[test]
    fn recorded_observations_rebuild_the_live_world() {
        let (Some(obs), Some(worlds)) = (
            std::env::var_os("BORG_OBSERVATION_LOG_CHECK"),
            std::env::var_os("BORG_WORLD_LOG_CHECK"),
        ) else {
            return;
        };
        let obs = std::fs::read_to_string(obs).expect("the observation log");
        let worlds = std::fs::read_to_string(worlds).expect("the world log");
        let (obs, worlds): (Vec<_>, Vec<_>) = (obs.lines().collect(), worlds.lines().collect());
        assert_eq!(obs.len(), worlds.len(), "one digest per observation");
        assert!(!obs.is_empty());
        let mut w = World::new();
        for (i, (line, live)) in obs.iter().zip(&worlds).enumerate() {
            w.update(&Observation::from_line(line).unwrap());
            assert_eq!(&w.digest(), live, "think {} differs from live play", i + 1);
        }
        eprintln!("{} thinks replayed into the same world", obs.len());
    }

    /// T2.3's check against the engine: on every think of a recorded run,
    /// the stat indices and the blows computed here equal the engine's own
    /// (`state.stat_ind`, `state.num_blows`). CI records the observations
    /// and the knowledge of one run; without them this test has nothing to
    /// read.
    #[test]
    fn recorded_blows_match_the_engine() {
        let (Some(obs), Some(know)) = (
            std::env::var_os("BORG_OBSERVATION_LOG_CHECK"),
            std::env::var_os("BORG_KNOWLEDGE_LOG_CHECK"),
        ) else {
            return;
        };
        let know = std::fs::read_to_string(know).expect("the knowledge log");
        let k = Knowledge::from_line(know.lines().next().unwrap()).unwrap();
        let obs = std::fs::read_to_string(obs).expect("the observation log");
        let mut w = World::new();
        let mut n = 0;
        for (i, line) in obs.lines().enumerate() {
            let o = Observation::from_line(line).unwrap();
            w.update(&o);
            let s = &o.status;
            let ind: Vec<i32> = s
                .stat_use
                .iter()
                .map(|&v| knowledge::stat_index(v) as i32)
                .collect();
            assert_eq!(ind, s.stat_ind.to_vec(), "think {}: stat indices", i + 1);
            assert_eq!(
                w.character.blows(&k),
                Some(s.blows),
                "think {}: blows",
                i + 1
            );
            n += 1;
        }
        assert!(n > 0);
        eprintln!("{n} thinks: blows and stat indices equal the engine's");
    }

    #[test]
    fn the_digest_follows_the_state() {
        let o = sample();
        let (a, b) = (world_of(&o), world_of(&o));
        assert_eq!(a.digest(), b.digest());
        let mut c = a.clone();
        c.update(&o);
        assert_ne!(a.digest(), c.digest(), "another think is another state");
    }
}
