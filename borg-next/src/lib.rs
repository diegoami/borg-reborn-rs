//! Track R's read-only projection of the vendored core's own memory.
//! No engine reads, globals, caches, coercions or decisions live here.
#![forbid(unsafe_code)]

use borg_rs::memory::traits::*;
use borg_rs::memory::{
    BorgGoal, BorgGrid, BorgKill, BorgState, BorgStatus, BorgTake, CaveState, AUTO_MAX_X,
    AUTO_MAX_Y,
};

// Each named field cites its exact source index. Values stay i32: even
// apparent booleans and resistance traits must retain borg-rs's semantics.
macro_rules! trait_group {
    ($name:ident { $($field:ident => $source:ident),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name {
            $(#[doc = concat!("Source: BorgState.borg.trait_[", stringify!($source), "].")]
            pub $field: i32,)*
        }
        impl $name {
            fn read(state: &BorgState) -> Self {
                Self { $($field: state.borg.tr($source),)* }
            }
        }
    };
}

trait_group!(Attributes {
    str => BI_STR,
    int => BI_INT,
    wis => BI_WIS,
    dex => BI_DEX,
    con => BI_CON,
    astr => BI_ASTR,
    aint => BI_AINT,
    awis => BI_AWIS,
    adex => BI_ADEX,
    acon => BI_ACON,
    cstr => BI_CSTR,
    cint => BI_CINT,
    cwis => BI_CWIS,
    cdex => BI_CDEX,
    ccon => BI_CCON,
    str_index => BI_STR_INDEX,
    int_index => BI_INT_INDEX,
    wis_index => BI_WIS_INDEX,
    dex_index => BI_DEX_INDEX,
    con_index => BI_CON_INDEX,
    sstr => BI_SSTR,
    sint => BI_SINT,
    swis => BI_SWIS,
    sdex => BI_SDEX,
    scon => BI_SCON,
});

trait_group!(Character {
    speed => BI_SPEED,
    cdepth => BI_CDEPTH,
    maxdepth => BI_MAXDEPTH,
    class => BI_CLASS,
    light => BI_LIGHT,
    curhp => BI_CURHP,
    maxhp => BI_MAXHP,
    hp_adj => BI_HP_ADJ,
    cursp => BI_CURSP,
    maxsp => BI_MAXSP,
    sp_adj => BI_SP_ADJ,
    fail1 => BI_FAIL1,
    fail2 => BI_FAIL2,
    clevel => BI_CLEVEL,
    maxclevel => BI_MAXCLEVEL,
    esp => BI_ESP,
    recall => BI_RECALL,
});

trait_group!(Food {
    food => BI_FOOD,
    food_hi => BI_FOOD_HI,
    food_lo => BI_FOOD_LO,
    food_cure_conf => BI_FOOD_CURE_CONF,
    food_cure_blind => BI_FOOD_CURE_BLIND,
});

trait_group!(Abilities {
    gold => BI_GOLD,
    mod_moves => BI_MOD_MOVES,
    dam_red => BI_DAM_RED,
    sdig => BI_SDIG,
    feath => BI_FEATH,
    reg => BI_REG,
    sinv => BI_SINV,
    infra => BI_INFRA,
    fast_shots => BI_FAST_SHOTS,
    disp => BI_DISP,
    dism => BI_DISM,
    dev => BI_DEV,
    sav => BI_SAV,
    stl => BI_STL,
    srch => BI_SRCH,
    thn => BI_THN,
    thb => BI_THB,
    tht => BI_THT,
    dig => BI_DIG,
});

trait_group!(Resistances {
    ifire => BI_IFIRE,
    iacid => BI_IACID,
    icold => BI_ICOLD,
    ielec => BI_IELEC,
    ipois => BI_IPOIS,
    rfire => BI_RFIRE,
    rcold => BI_RCOLD,
    relec => BI_RELEC,
    racid => BI_RACID,
    rpois => BI_RPOIS,
    rfear => BI_RFEAR,
    rlite => BI_RLITE,
    rdark => BI_RDARK,
    rblind => BI_RBLIND,
    rconf => BI_RCONF,
    rsnd => BI_RSND,
    rshrd => BI_RSHRD,
    rnxus => BI_RNXUS,
    rnthr => BI_RNTHR,
    rkaos => BI_RKAOS,
    rdis => BI_RDIS,
    hlife => BI_HLIFE,
    fract => BI_FRACT,
    srfire => BI_SRFIRE,
    srcold => BI_SRCOLD,
    srelec => BI_SRELEC,
    sracid => BI_SRACID,
    srpois => BI_SRPOIS,
    srfear => BI_SRFEAR,
    srlite => BI_SRLITE,
    srdark => BI_SRDARK,
    srblind => BI_SRBLIND,
    srconf => BI_SRCONF,
    srsnd => BI_SRSND,
    srshrd => BI_SRSHRD,
    srnxus => BI_SRNXUS,
    srnthr => BI_SRNTHR,
    srkaos => BI_SRKAOS,
    srdis => BI_SRDIS,
    shlife => BI_SHLIFE,
    sfract => BI_SFRACT,
});

trait_group!(Status {
    king => BI_KING,
    isweak => BI_ISWEAK,
    ishungry => BI_ISHUNGRY,
    isfull => BI_ISFULL,
    isgorged => BI_ISGORGED,
    isblind => BI_ISBLIND,
    isafraid => BI_ISAFRAID,
    isconfused => BI_ISCONFUSED,
    ispoisoned => BI_ISPOISONED,
    iscut => BI_ISCUT,
    isstun => BI_ISSTUN,
    isheavystun => BI_ISHEAVYSTUN,
    isparalyzed => BI_ISPARALYZED,
    isimage => BI_ISIMAGE,
    isforget => BI_ISFORGET,
    isencumb => BI_ISENCUMB,
    isstudy => BI_ISSTUDY,
    isfixlev => BI_ISFIXLEV,
    isfixexp => BI_ISFIXEXP,
    hasfixexp => BI_HASFIXEXP,
    isfixstr => BI_ISFIXSTR,
    isfixint => BI_ISFIXINT,
    isfixwis => BI_ISFIXWIS,
    isfixdex => BI_ISFIXDEX,
    isfixcon => BI_ISFIXCON,
    isfixall => BI_ISFIXALL,
});

trait_group!(Combat {
    armor => BI_ARMOR,
    tohit => BI_TOHIT,
    todam => BI_TODAM,
    wtohit => BI_WTOHIT,
    wtodam => BI_WTODAM,
    wid => BI_WID,
    wdd => BI_WDD,
    wds => BI_WDS,
    bid => BI_BID,
    btohit => BI_BTOHIT,
    btodam => BI_BTODAM,
    sling => BI_SLING,
    bart => BI_BART,
    blows => BI_BLOWS,
    extra_blows => BI_EXTRA_BLOWS,
    shots => BI_SHOTS,
    heavywepon => BI_HEAVYWEPON,
    heavybow => BI_HEAVYBOW,
});

trait_group!(Ammo {
    ammo_count => BI_AMMO_COUNT,
    ammo_tval => BI_AMMO_TVAL,
    ammo_sides => BI_AMMO_SIDES,
    ammo_power => BI_AMMO_POWER,
    amissiles => BI_AMISSILES,
    amissiles_special => BI_AMISSILES_SPECIAL,
    amissiles_cursed => BI_AMISSILES_CURSED,
    quiver_slots => BI_QUIVER_SLOTS,
});

trait_group!(Curses {
    first_cursed => BI_FIRST_CURSED,
    where_cursed => BI_WHERE_CURSED,
    crsenveloping => BI_CRSENVELOPING,
    crsirritation => BI_CRSIRRITATION,
    crstele => BI_CRSTELE,
    crspois => BI_CRSPOIS,
    crssiren => BI_CRSSIREN,
    crshalu => BI_CRSHALU,
    crspara => BI_CRSPARA,
    crssdem => BI_CRSSDEM,
    crssdra => BI_CRSSDRA,
    crssund => BI_CRSSUND,
    crsstone => BI_CRSSTONE,
    crsnotel => BI_CRSNOTEL,
    crstwep => BI_CRSTWEP,
    crsagrv => BI_CRSAGRV,
    crsvuln => BI_CRSVULN,
    crsdull => BI_CRSDULL,
    crssick => BI_CRSSICK,
    crsweak => BI_CRSWEAK,
    crsclum => BI_CRSCLUM,
    crsslow => BI_CRSSLOW,
    crsannoy => BI_CRSANNOY,
    crshpimp => BI_CRSHPIMP,
    crsmpimp => BI_CRSMPIMP,
    crssteelskin => BI_CRSSTEELSKIN,
    crsairswing => BI_CRSAIRSWING,
    crsfear => BI_CRSFEAR,
    crsdrain_xp => BI_CRSDRAIN_XP,
    crsfvuln => BI_CRSFVULN,
    crsevuln => BI_CRSEVULN,
    crscvuln => BI_CRSCVULN,
    crsavuln => BI_CRSAVULN,
    crsunkno => BI_CRSUNKNO,
});

trait_group!(Weapon {
    ws_animal => BI_WS_ANIMAL,
    ws_evil => BI_WS_EVIL,
    ws_undead => BI_WS_UNDEAD,
    ws_demon => BI_WS_DEMON,
    ws_orc => BI_WS_ORC,
    ws_troll => BI_WS_TROLL,
    ws_giant => BI_WS_GIANT,
    ws_dragon => BI_WS_DRAGON,
    wk_undead => BI_WK_UNDEAD,
    wk_demon => BI_WK_DEMON,
    wk_dragon => BI_WK_DRAGON,
    w_impact => BI_W_IMPACT,
    wb_acid => BI_WB_ACID,
    wb_elec => BI_WB_ELEC,
    wb_fire => BI_WB_FIRE,
    wb_cold => BI_WB_COLD,
    wb_pois => BI_WB_POIS,
});

trait_group!(Supplies {
    aphase => BI_APHASE,
    ateleport => BI_ATELEPORT,
    aescape => BI_AESCAPE,
    afuel => BI_AFUEL,
    aheal => BI_AHEAL,
    aezheal => BI_AEZHEAL,
    alife => BI_ALIFE,
    aid => BI_AID,
    aspeed => BI_ASPEED,
    astfmagi => BI_ASTFMAGI,
    astfdest => BI_ASTFDEST,
    atportother => BI_ATPORTOTHER,
    acurepois => BI_ACUREPOIS,
    adettrap => BI_ADETTRAP,
    adetdoor => BI_ADETDOOR,
    adetevil => BI_ADETEVIL,
    amagicmap => BI_AMAGICMAP,
    arecharge => BI_ARECHARGE,
    alite => BI_ALITE,
    apfe => BI_APFE,
    aglyph => BI_AGLYPH,
    accw => BI_ACCW,
    acsw => BI_ACSW,
    aclw => BI_ACLW,
    aench_toh => BI_AENCH_TOH,
    aench_tod => BI_AENCH_TOD,
    aench_swep => BI_AENCH_SWEP,
    aench_arm => BI_AENCH_ARM,
    aench_sarm => BI_AENCH_SARM,
    abrand => BI_ABRAND,
    need_enchant_to_a => BI_NEED_ENCHANT_TO_A,
    need_enchant_to_h => BI_NEED_ENCHANT_TO_H,
    need_enchant_to_d => BI_NEED_ENCHANT_TO_D,
    need_brand_weapon => BI_NEED_BRAND_WEAPON,
    aresheat => BI_ARESHEAT,
    arescold => BI_ARESCOLD,
    arespois => BI_ARESPOIS,
    ateleportlvl => BI_ATELEPORTLVL,
    ahword => BI_AHWORD,
    amassban => BI_AMASSBAN,
    ashroom => BI_ASHROOM,
});

trait_group!(Inventory {
    arod1 => BI_AROD1,
    arod2 => BI_AROD2,
    worn_need_id => BI_WORN_NEED_ID,
    all_need_id => BI_ALL_NEED_ID,
    adigger => BI_ADIGGER,
    good_s_chg => BI_GOOD_S_CHG,
    good_w_chg => BI_GOOD_W_CHG,
    multiple_bonuses => BI_MULTIPLE_BONUSES,
    dinv => BI_DINV,
    weight => BI_WEIGHT,
    carry => BI_CARRY,
    empty => BI_EMPTY,
    sauron_dead => BI_SAURON_DEAD,
    prep_big_fight => BI_PREP_BIG_FIGHT,
});

/// A fresh view for one think. Its lifetime prevents retaining borrowed
/// map/entity/goal memory across any mutation of the core state.
pub struct View<'a> {
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub attributes: Attributes,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub character: Character,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub food: Food,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub abilities: Abilities,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub resistances: Resistances,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub status: Status,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub combat: Combat,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub ammo: Ammo,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub curses: Curses,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub weapon: Weapon,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub supplies: Supplies,
    /// Source: `BorgState.borg.trait_`; see each field for its BI index.
    pub inventory: Inventory,
    /// Source: `BorgState.borg.status` (all native status flags).
    pub core_status: &'a BorgStatus,
    /// Source: `BorgState.borg.goal` (all goal flags and counters).
    pub goals: &'a BorgGoal,
    /// Source: `BorgState.borg.c.x`.
    pub x: i32,
    /// Source: `BorgState.borg.c.y`.
    pub y: i32,
    /// Source: `BorgState.cave`.
    pub cave: Cave<'a>,
    pub threat: ThreatMemory<'a>,
    state: &'a BorgState,
}

impl<'a> View<'a> {
    /// Build after the core has updated its memory for this think.
    /// R1.2 will call this from the shadow hook; R1.1 installs no hook.
    pub fn new(state: &'a BorgState) -> Self {
        Self {
            attributes: Attributes::read(state),
            character: Character::read(state),
            food: Food::read(state),
            abilities: Abilities::read(state),
            resistances: Resistances::read(state),
            status: Status::read(state),
            combat: Combat::read(state),
            ammo: Ammo::read(state),
            curses: Curses::read(state),
            weapon: Weapon::read(state),
            supplies: Supplies::read(state),
            inventory: Inventory::read(state),
            core_status: &state.borg.status,
            goals: &state.borg.goal,
            x: state.borg.c.x,
            y: state.borg.c.y,
            cave: Cave(&state.cave),
            threat: ThreatMemory::read(state),
            state,
        }
    }

    /// Source: `KillState.kills`, excluding slot zero and deleted records
    /// (`BorgKill.r_idx == 0`). Retains slot IDs used by `BorgGrid.kill`.
    pub fn monsters(&self) -> impl Iterator<Item = (usize, &BorgKill)> {
        self.state
            .kills
            .kills
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, k)| k.r_idx != 0)
    }

    /// Source: `TakeState.takes`, excluding slot zero and deleted records
    /// (`BorgTake.kidx == 0`). Positions are native `BorgTake.x/y`.
    pub fn objects(&self) -> impl Iterator<Item = (usize, &BorgTake)> {
        self.state
            .takes
            .takes
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, t)| t.kidx != 0)
    }
}

/// Borrowed grid memory; never exposes a mutable grid or engine access.
pub struct Cave<'a>(&'a CaveState);

impl Cave<'_> {
    /// Source: `CaveState.grid(x, y)` / `borg_grids[y][x]`.
    /// Unallocated memory has no grid.
    /// Out-of-bounds positions have no grid; unknown in-bounds grids retain
    /// their native zero/unknown fields rather than becoming floor.
    pub fn grid(&self, x: i32, y: i32) -> Option<&BorgGrid> {
        if !(0..AUTO_MAX_X).contains(&x) || !(0..AUTO_MAX_Y).contains(&y) {
            return None;
        }
        self.0.remembered_grid(x, y)
    }
}

/// Native danger context, borrowed through the same lifetime as the view.
/// Reads memory only; all engine/table queries are in `threat_facts::Facts`.
pub struct ThreatMemory<'a> {
    state: &'a BorgState,
    /// Native temporary buffs, not engine timed effects.
    pub temporary: &'a borg_rs::memory::BorgTemp,
    pub clock: &'a borg_rs::memory::BorgTimes,
    pub antibounce_count: i16,
    pub nearby_unique: i16,
    pub morgoth_position: bool,
    pub simulation: Simulation,
}
/// Native attack simulation modifiers (may be hypothetical during defence).
#[derive(Clone, Copy)]
pub struct Simulation {
    pub attacking: bool,
    pub on_glyph: bool,
    pub create_door: bool,
    pub sleep_spell: bool,
    pub sleep_spell_ii: bool,
    pub crush_spell: bool,
    pub slow_spell: bool,
    pub confuse_spell: bool,
    pub fear_mon_spell: bool,
}
impl<'a> ThreatMemory<'a> {
    fn read(state: &'a BorgState) -> Self {
        let t = &state.traits;
        Self {
            state,
            temporary: &state.borg.temp,
            clock: &state.borg.time,
            antibounce_count: state.borg.antibounce_count,
            nearby_unique: state.borg.near.unique,
            morgoth_position: state.borg.morgoth_position,
            simulation: Simulation {
                attacking: t.attacking,
                on_glyph: t.on_glyph,
                create_door: t.create_door,
                sleep_spell: t.sleep_spell,
                sleep_spell_ii: t.sleep_spell_ii,
                crush_spell: t.crush_spell,
                slow_spell: t.slow_spell,
                confuse_spell: t.confuse_spell,
                fear_mon_spell: t.fear_mon_spell,
            },
        }
    }
    pub fn regional_fear(&self, y: i32, x: i32) -> u16 {
        self.state.danger.fear_region[(y / 11) as usize][(x / 11) as usize]
    }
    pub fn monster_fear(&self, y: i32, x: i32) -> u16 {
        self.state.danger.fear_monsters[y as usize][x as usize]
    }
    pub fn teleport_ignored(&self) -> &[i32] {
        &self.state.fight_attack.tp_other_index[..self.state.fight_attack.tp_other_n as usize]
    }
    pub fn glyphs(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        (0..self.state.glyph.track.num as usize).map(|i| self.state.glyph.track.at(i))
    }
    /// Preserve native slot order and the `nxt` bound, including holes.
    pub fn monster_slots(&self) -> &[BorgKill] {
        &self.state.kills.kills[..self.state.kills.nxt as usize]
    }
}
pub mod danger;
pub mod shadow;
pub use shadow::register;

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> BorgState {
        let mut state = BorgState::default();
        state.borg.trait_ = (0..BI_MAX).map(|i| i as i32 * 7 - 13).collect();
        state
    }

    #[test]
    fn named_traits_keep_native_values() {
        let state = state();
        let view = View::new(&state);
        assert_eq!(view.character.curhp, state.borg.tr(BI_CURHP));
        assert_eq!(view.character.cursp, state.borg.tr(BI_CURSP));
        assert_eq!(view.character.clevel, state.borg.tr(BI_CLEVEL));
        assert_eq!(view.character.cdepth, state.borg.tr(BI_CDEPTH));
        assert_eq!(view.character.speed, state.borg.tr(BI_SPEED));
        assert_eq!(view.resistances.rfire, state.borg.tr(BI_RFIRE));
        assert_eq!(view.resistances.srfire, state.borg.tr(BI_SRFIRE));
        assert_eq!(view.status.ispoisoned, state.borg.tr(BI_ISPOISONED));
        assert_eq!(view.supplies.aclw, state.borg.tr(BI_ACLW));
        assert_eq!(view.supplies.aphase, state.borg.tr(BI_APHASE));
        assert_eq!(view.ammo.amissiles, state.borg.tr(BI_AMISSILES));
        assert_eq!(state.borg.trait_[0], -13);
    }

    #[test]
    fn entity_slots_goals_positions_and_holes_are_preserved() {
        let mut state = state();
        state.borg.c.x = 14;
        state.borg.c.y = 9;
        state.borg.goal.fleeing = true;
        state.borg.goal.recalling = 87;
        state.kills.kills = vec![BorgKill::default(); 5];
        state.kills.kills[0].r_idx = 999; // sentinel never becomes an entity
        state.kills.kills[3].r_idx = 12;
        state.kills.kills[3].pos.x = 11;
        state.kills.kills[3].pos.y = 8;
        state.kills.kills[3].power = 73;
        state.takes.takes = vec![BorgTake::default(); 4];
        state.takes.takes[0].kidx = 999;
        state.takes.takes[2].kidx = 21;
        state.takes.takes[2].x = 15;
        state.takes.takes[2].y = 10;
        let view = View::new(&state);
        assert_eq!((view.x, view.y), (14, 9));
        assert!(view.goals.fleeing);
        assert_eq!(view.goals.recalling, 87);
        assert!(std::ptr::eq(view.goals, &state.borg.goal));
        assert!(std::ptr::eq(view.core_status, &state.borg.status));
        let monsters: Vec<_> = view.monsters().collect();
        assert_eq!(monsters.len(), 1);
        assert_eq!(monsters[0].0, 3);
        assert_eq!((monsters[0].1.pos.x, monsters[0].1.pos.y), (11, 8));
        assert_eq!(monsters[0].1.power, 73);
        let objects: Vec<_> = view.objects().collect();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].0, 2);
        assert_eq!((objects[0].1.x, objects[0].1.y), (15, 10));
    }

    #[test]
    fn unallocated_and_outside_memory_are_absent() {
        let state = state();
        let view = View::new(&state);
        for (x, y) in [(0, 0), (-1, 0), (0, -1), (AUTO_MAX_X, 0), (0, AUTO_MAX_Y)] {
            assert!(view.cave.grid(x, y).is_none());
        }
        assert_eq!(view.monsters().count(), 0);
        assert_eq!(view.objects().count(), 0);
    }

    #[test]
    fn each_think_reads_the_current_memory() {
        let mut state = state();
        assert_ne!(View::new(&state).character.curhp, 19);
        state.borg.trait_[BI_CURHP as usize] = 19;
        assert_eq!(View::new(&state).character.curhp, 19);
    }
}

pub mod stateful;
