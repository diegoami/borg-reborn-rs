//! borg-reborn-rs (#219): calibration of the vendored core.
//!
//! Source locations: each citation points at the first line in the live tree that reads the
//! parameter, or inside the function that does (scripts/check-param-citations.py, #601).
//! Defaults retain the original integer operations and RNG calls.
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

// One row generates both the lookup index and its metadata: additions cannot
// accidentally shift the values away from their names.
macro_rules! parameters {
    (@class_defaults) => { None };
    (@class_defaults $values:expr) => { Some(&$values) };
    ($($name:ident => ($default:expr, $unit:expr, $min:expr, $max:expr, $source:expr, $description:expr $(, $class_defaults:expr)?)),* $(,)?) => {
        #[derive(Clone, Copy)]
        #[repr(usize)]
        pub(crate) enum Param { $($name),* }
        const PARAMETERS: &[Param] = &[$(Param::$name),*];
        pub(crate) const SPECS: &[Spec] = &[
            $(Spec { name: stringify!($name), default: $default, unit: $unit,
                min: $min, max: $max, source: $source, description: $description,
                class_defaults: parameters!(@class_defaults $($class_defaults)?) }),*
        ];
    };
}

#[derive(Serialize)]
pub(crate) struct Spec {
    pub name: &'static str,
    pub default: i32,
    pub unit: &'static str,
    pub min: i32,
    pub max: i32,
    pub source: &'static str,
    pub description: &'static str,
    #[serde(skip)]
    class_defaults: Option<&'static [i32; CLASSES.len()]>,
}

parameters! {
    caution_recall_survival => (0, "flag", 0, 1, "borg_caution.rs:3596", "#758: refuse flight Recall when modelled contact HP reaches current HP during the delay, or a reachable spell/status contribution is unmodelled. Sleepers may wake. Unmodelled risk is reported separately, never as damage. Ordinary trip-home Recall and countdown handling are unchanged. 0 retains today's play."),
    caution_recall_survival_delay => (34, "world_ticks", 15, 100, "borg_caution.rs:3597", "#758: flight Recall survival horizon, in ten-game-turn world ticks, converted with monster/player energy. 34 is the engine's maximum random delay (15 + randint0(20)); a conservative hypothesis."),
    caution_recall_blow_model => (0, "flag", 0, 1, "borg_caution.rs:3601", "#880: 1 = the flight Recall forecast prices a POISON blow as HP (its dice plus the poison timer: 1 HP per world tick, game-world.c:513-515, from the earliest strike to the end of the delay, at most the 5 + rlev ticks per possible hit the poisoners can add, mon-blows.c:683-684) and every other status blow by its dice, since no status stops the countdown; only SHATTER and ranged spells stay unmodelled; a breeder's unmodelled risk never vetoes; contact HP is capped by the open grids next to the Borg (that many fastest attackers for the whole delay) and discounted for RAND_25/RAND_50 movers. Needs caution_recall_survival = 1. 0 = #758's rule: any blow other than HURT/ACID/ELEC/FIRE/COLD/NONE refuses Recall."),
    caution_winning_fight => (0, "mode", 0, 2, "borg_caution.rs:4043", "#759: 1 attacks while estimated turns to kill every attacker are strictly fewer than turns to die; 2 only logs estimates and retains control actions. Uses remaining HP once, attack success, finite supplies, and monster/player speed. Unknown estimates fall through to ordinary caution. 0 retains today's play."),
    caution_winning_fight_margin_percent => (0, "percent of HP", 0, 99, "borg_caution.rs:4047", "#992: require the race-certified priority attack to leave strictly more than this percent of current HP after expected damage. A rejection yields to ordinary caution. Mode 2 logs the reserve check alongside the original verdict. 0 keeps the existing verdict. Phase Door uses the original race."),
    caution_ranged_only => (0, "flag", 0, 1, "borg_caution.rs:1990", "#838, #852 (owner): 1 = the scary list's ranged entries (`monster | up to CL | classes | ranged | reason`) are kept at range, up to the level for the class, while one of their blows has an effect the character is not protected from (#852: paralysis without Free Action, confusion, poison, blindness or fear without the resistance; any other effect but plain damage has no protection; an entry with only plain-damage blows stays kept): no melee blow at it, no hunt walk to it, and no flow step onto its grid; missiles, spells, throws and devices still reach it, or it is left alone. The flows around it are not changed (#852: path finding is another concern). Not a scare. Works for any listed monster, and needs no step away (caution_ranged_vs_immobile stays as it is). 0 = ranged entries are not read."),
    attack_on_sight => (0, "flag", 0, 1, "borg_fight_attack.rs:5799", "#760: prioritize attack-category monsters with their recorded reasons, never classify them as scares. Breeder entries honor the existing stoppable-colony policy. 0 retains today's play."),
    attack_kill_on_sight_min_hp_percent => (0, "percent", 0, 100, "borg_fight_attack.rs:6090", "#969: below this percent of max HP a `kill-on-sight` entry of the scary list (#869: the grey mushroom patch) is not attacked on sight when a known Potion of Cure Light, Serious or Critical Wounds is in the pack or an escape is at hand (stairs underfoot, up or down, or a Phase Door that can be read: borg_fast_fragile::escape_available). The floor only stops the attack on sight: the ordinary attack and caution then decide as for any monster, and the caution may still decline the cure (its own heal gates; caution_confused_clw_cure ships 0), so the Borg may neither attack nor drink. With no such potion, or at or above the floor, the attack still comes first, confused or not. 0 retains today's play (no floor: 11 of 28 deaths at 9085c266 swung confused until death)."),
    attack_kill_on_sight_floor_cure => (0, "flag", 0, 1, "borg_fight_attack.rs:6107", "#1064: 1 = when attack_kill_on_sight_min_hp_percent's floor declines the attack on sight of a `kill-on-sight` entry, the cure potion it counted is drunk in that think, before the flight: confused, Cure Serious and then Cure Critical Wounds (they cure the confusion, object.txt), then Cure Light Wounds (heals 20, shortens the confusion by 20 turns); not confused, the cheapest first. Not when the potion does not outlive the next turn (the damage-only danger at the Borg's grid, #878, at or above its HP plus the heal, capped at the HP missing) and an escape is at hand: the floor then takes the escape in the same think, the stairs underfoot or Phase Door (`phase_door`'s own gates, site `kill-on-sight floor`), and drinks the potion after all when neither can be used (review of #1082). And #867's hold of the confusion cure next to an immobile confuser (attack_confused_by_direction) does not apply to a confuser the floor declines. When the floor counted only an escape (no such potion), the floor itself drinks nothing and the caution re-evaluates its cures with the hold lifted, so a cure the floor does not count (a Cure Mind mushroom) may be taken. 0 = #969 as shipped before (the caution's heal gates and #867's hold could leave it neither attacking nor curing: Druid 781905420 died two steps from the stairs with 8 CLW)."),
    unseen_belief => (0, "flag", 0, 1, "borg_unseen_belief.rs:448", "#879 (owner): 1 = a monster seen within unseen_max_age Borg turns but not now, at least two grids away, whose last grid the Borg would not expect to see it on (dark, beyond its light), stays a target of the ranged attacks (bolts, balls, missiles, throws) at that grid, each attack's value weighted by the chance it is still there (unseen_decay_percent). Only a grid on the panel and inside the Borg's view shape (BORG_OKAY, BORG_VIEW; lit or not) can be aimed at, and infravision raises the two-grid floor for warm monsters. A believed record never counts as the monster next to the Borg. A bolt stops at the first monster on its line, so one that closed in is hit too. Melee never takes such a target. The Curse spell requires a real monster target and excludes believed records. Pairs with cave_light_engine_radius = 1 (#877), without which the dark grid two away is expected visible and the record deleted. 0 retains today's play."),
    unseen_max_age => (10, "Borg turns", 3, 2000, "borg_unseen_belief.rs:456", "#879: the oldest record still believed at its last grid, in Borg turns since the monster was last seen. Hypothesis: no measurement chose 10. Only with unseen_belief = 1."),
    unseen_decay_percent => (20, "percent per monster turn", 0, 100, "borg_unseen_belief.rs:471", "#879: the belief that a mobile, awake monster is still at its last grid falls by this many percent for each of its turns since it was seen (Borg turns x its energy / the Borg's); 100% for NEVER_MOVE or a sleeper. Hypothesis: no measurement chose 20. Only with unseen_belief = 1."),
    unseen_max_shots => (3, "shots", 1, 100, "borg_unseen_belief.rs:221", "#879: at most this many shots at a believed (unseen) monster per sighting; after a shot a sleeper counts as awake. The engine names no monster the player cannot see (\"It dies.\"), so a record shot dead stays until this cap. Hypothesis: no measurement chose 3 (a floating eye took 3 Magic Missiles and flasks). Only with unseen_belief = 1."),
    unseen_death_stops_shots => (0, "flag", 0, 1, "borg_unseen_belief.rs:209", "#889: 1 = \"It dies.\" or \"It is destroyed.\" among the messages of a ranged attack whose only believed target was one record spends that record\'s remaining unseen shots for this sighting (unseen_max_shots): no more shots at a monster that is probably dead. The record is kept (review of #1128, F1): the engine names a monster the player cannot see \"It\" (src/mon-msg.c:320-330, src/player-attack.c:1122-1123), so the message cannot say which monster died; a bolt can kill an untracked monster that stepped into its path, a ball another unseen monster. With several believed targets, or \"2 monsters die.\", nothing changes. 0 = up to unseen_max_shots shots, today\'s play. Needs unseen_belief = 1."),
    unseen_light_reveal => (0, "mode", 0, 2, "borg_unseen_belief.rs:318", "#889: before the attack choice, cast Light Room (fail 40% or less) at a monster believed at its last grid (unseen_belief) inside its sphere (SPHERE:LIGHT_WEAK:2, lib/gamedata/class.txt:221-224: at most two grids, in line of sight for a projection), on a grid the Borg does not already count as lit (BORG_GLOW, BORG_LIGHT), at most once per sighting of the record (review of #1128, F3: lighting an invisible monster\'s grid again reveals nothing). The sphere hurts light-sensitive monsters (HURT_LIGHT) and lights the grids it reaches, so the monster is seen, or its record is forgotten. 1 = only at a HURT_LIGHT monster, 2 = at any believed monster. Call Light\'s sphere (radius 1) never reaches a believed target. 0 = no such cast, today\'s play."),
    unseen_spread => (0, "flag", 0, 1, "borg_unseen_belief.rs:475", "#889: 1 = the belief spreads around the last grid: an erratic mover\'s random steps do not count as leaving (RAND_25 25%, RAND_50 50%, both 75%: src/mon-move.c:1087-1101), and a ball still hits a monster that took no more steps than the ball covers in every direction from its last grid (blast_slack: the most steps n such that every grid n king\'s moves away is within the ball by the engine\'s distance; not radius less distance, since one diagonal step can add 2: re-review of #1128 at e0278dcf; radius 1 to 4). Bolts, beams, arcs, throws, Starlight and the line-of-sight effects (dispels, mass sleep, slow, confuse: radius 10, which reach what the player sees, not a radius around the aim) take no slack (re-review of #1128, F1), nor do the dragon breaths, which are cones (ball_shaped; Codex re-review F2). borg_attack admits a record a ball could still reach (believed_for_aim, slack up to 3; review of #1128, F4), but only to the ball\'s own list (blast_aims): no other attack aims at it, and it never makes attack_mobile_threat_first drop the immobile targets (blast_only; re-review F2, Codex re-review F1). Its next-to-me test counts it as believed. 0 = the decay counts every monster turn, today\'s play."),
    trap_recover => (0, "flag", 0, 1, "borg_trap_way.rs:341", "#1067 (owner, 2026-11-23): 1 = below trap_recover_max_hp max HP, a trap set off that hurts (after it: HP below the maximum, poisoned, cut, blind, confused or stunned) stops exploration: the Borg walks back to the nearest staircase it may take (up; down only when not rising and not refused by preparedness; a flow that refuses known traps as today), cures or rests there (borg_recover, else a rest as needed), takes it at the first threat (danger above 0 on it) and explores again only whole (full HP, those statuses gone). With no staircase in reach it rests where it stands (danger 0, not hungry or weak). A trap is set off when the engine prints a trap.txt message or \"You set off the %s!\" (trap.c:146-246, cmd-cave.c:842-845). The Warrior 1868115537: a poison pit four steps from its up staircase, explored on, died 5 steps from it. 0 = today."),
    trap_recover_max_hp => (60, "max HP", 1, 10000, "borg_trap_way.rs:389", "#1067: trap_recover and trap_leave_level_count apply while the maximum HP is below this. 60 is the core's own line for a known trap (spread_blocked: below 60 current HP every known trap is a wall), above every CL1-6 character; #1061's stair_scum threshold (25, per class) is lower than the Warrior 1868115537 (31 then 57 HP). Per class with trap_recover_max_hp__<class>. Hypothesis: no measurement chose 60."),
    trap_way_back => (0, "flag", 0, 1, "borg_trap_way.rs:285", "#1067 (owner): 1 = when no known staircase can be reached by a flow that refuses known traps (spread_blocked, as every flow does below 60 HP or with a low disarm skill) but one can through them, the next action reopens the way: from the grid next to the first trap of the route, D (disarm) when the engine's chance (the skill less depth/5, at least 2; cmd-cave.c:808-825; the lower of the two skills when the kind is unknown) is at least trap_disarm_min_percent; else, only while trap_recover or trap_leave_level_count wants the stairs now (a walk leaves the trap in place), W (walk onto it) when the kind is known from setting it off there and its worst effect cannot kill at the current HP (blinding or confusion gas; a pit at more than 12 HP); else, hurt, it rests first and disarms whole. Never explores past it, until trap_disarm_max_tries attempts on that trap. Blind, confused or hallucinating it rests first (curing first, borg_recover); without light it leaves the light check to act. A flow may leave a trap the Borg stands on (after a W) but enters no other. Only the staircases it may take count (up; down only when not rising and not refused). Together with #1062: an unreachable staircase stops meaning off leash. 0 = today."),
    trap_disarm_min_percent => (50, "percent", 2, 100, "borg_trap_way.rs:625", "#1067: with trap_way_back, the least disarm chance at which a hurt Borg disarms the trap that cuts it off before it recovers (at 50 a try sets the trap off a quarter of the time: the engine rolls the chance twice, cmd-cave.c:828-846). Whole, it disarms at any chance. Hypothesis: no measurement chose 50."),
    trap_disarm_max_tries => (10, "attempts", 1, 1000, "borg_trap_way.rs:610", "#1067: with trap_way_back, the D and W attempts on one trap per level before the rule gives it up (play is then as today), so a 2% chance cannot hold the Borg for ever."),
    trap_leave_level_count => (0, "traps", 0, 100, "borg_trap_way.rs:343", "#1067 (owner: \"it had the form of a cursed level\"): below trap_recover_max_hp, the level's n-th distinct trap set off (each grid once; a D or W counts at its trap, a walk where the Borg then stands, a teleport rune at the step's grid when not confused, else not placed) makes the Borg leave by the nearest staircase it may take (up; down only when not rising and not refused by preparedness), whole first with trap_recover, once it can reach one. The issue proposes 2. 0 = off (today)."),
    trap_leave_near_count => (0, "traps", 0, 100, "borg_trap_way.rs:344", "#1067 (review of #1086): as trap_leave_level_count, but counting only the traps within trap_leave_radius of the Borg's first grid on the level. Every such trap is also one of the level, so this matters only below trap_leave_level_count (or with it 0). 0 = off (today)."),
    trap_leave_radius => (10, "grids", 0, 250, "borg_trap_way.rs:466", "#1067: with trap_leave_near_count, the distance (the engine's) from the Borg's first grid on the level within which a trap set off counts as near the arrival staircase. Hypothesis: no measurement chose 10."),
    deal_with_poison => (0, "flag", 0, 1, "borg_poison.rs:197", "#1140 (owner, 2026-11-23: \"just one switch\"): 1 = poison triage, all parts at once, from the HP the poison still takes, ceil(counter / (adj_con_fix[CON] + 1)) (the engine counter, game-world.c:282, 317-320, 513-519), against the HP and the Cure Light/Serious Wounds carried (each at most what it restores drunk at B's line, max HP - line); a cure counts only when usable now (spell mana and fail rate, a charged rod, staff or activation, the potion or food carried). A: a note while poisoned. B: poisoned, not cut, at danger 0, CLW/CSW are held unless the whole heal fits under the max HP, and one is drunk at HP <= the next turn's poison + poison_late_drink_margin. C: in the dungeon, when the poison takes at least HP + healing and no cure is carried, leave for town at once (leaving, fleeing, rising; Recall below DL1 or with no known up staircase, only when HP + healing outlast its 34-tick delay, or its 15 with no stairs known). While poisoned the early fights (fight_before_flight) wait next to a poisoner, when a cure is due or past HP + healing. D: in town with the poison at least the HP, the Alchemist first: Neutralize Poison, else CCW, if affordable, else the CLW that outlast it; no sale while birth_no_selling pays nothing; a bought cure drunk outside the store. E: next to awake poisoners, turns to kill x (damage + new poison) + the pending poison against HP + healing; a lost race leaves, stepping out of reach of one that never moves. F: a poison cure the Borg has (spells, rod, activation, food, Neutralize Poison, staff, CCW; Healing, *Healing* or Life only when the CLW/CSW cannot outlast it) is used once the poison is poison_cure_pending_percent of the HP, at any danger, no roll; never next to an awake poisoner (it poisons again at once). Rogue 626789725, Druid 799968561, Ranger 956247604 (borg-cemetery#146), Paladin 1200156945 (#1134). 0 = today."),
    poison_late_drink_margin => (2, "HP", 0, 100, "borg_poison.rs:243", "#1140: with deal_with_poison (part B), the HP above the next turn's poison at or below which a held CLW or CSW is drunk. Hypothesis: no measurement chose 2."),
    poison_cure_pending_percent => (50, "percent of current HP", 0, 1000, "borg_poison.rs:384", "#1140: with deal_with_poison (part F), the poison still to come, as a percent of the current HP, from which a carried cure is used (0 = any poison). Hypothesis: no measurement chose 50."),
    fast_fragile_policy => (0, "flag", 0, 1, "borg_fast_fragile.rs:47", "#842 (owner, 2026-11-21): a fast-fragile monster (a `fast-fragile` entry of the scary list, or by stats with fast_fragile_by_stats) is attacked with the best attack at hand every turn, without attack-on-sight's `target damage < HP` gate; the only alternatives are Phase Door or a staircase underfoot, when the next turn kills the Borg and the kill needs more than one more turn. Needs attack_on_sight = 1. 0 retains today's play."),
    fast_fragile_by_stats => (0, "flag", 0, 1, "borg_fast_fragile.rs:340", "#842: besides the named entries, mark a monster fast-fragile by its stats for this character: faster than the Borg, killed in fast_fragile_kill_turns of its best attack, at least fast_fragile_damage_percent of the current HP per Borg turn, mobile, not unique, not a breeder, at most fast_fragile_group_max others of its race in view. A named attack entry (any reason) decides its monster over the stats. Acts with fast_fragile_policy and fast_fragile_no_heal. 0 = the names only."),
    fast_fragile_no_heal => (0, "flag", 0, 2, "borg_fast_fragile.rs:536", "#842: 1 = no healing potion (borg_heal, the outmatched heal) while a fast-fragile attacker is awake next to the Borg: the cure buys a turn and loses the race (owner: a death sentence); the attack, the escapes and the flight decide instead. 2 = the same, except when cornered (the next turn kills, the kill needs more than one more turn, and neither Phase Door nor a staircase underfoot is at hand): then a cure comes before the attack. Needs attack_on_sight = 1. 0 retains today\'s play."),
    fast_fragile_escape_percent => (100, "percent of HP", 0, 1000, "borg_fast_fragile.rs:538", "#842: the next turn \'kills\' the Borg (for Phase Door, the stairs underfoot and the cornered exception) when the expected damage of the fast-fragile monsters within two grids reaches this percent of its HP. 100 is the mean; less leaves room for the spread of four bites (the Dunadan Priest 1309976551: 7.2 damage against 8 HP, dead in two rounds). Hypothesis: no measurement chose 100."),
    fast_fragile_phase_landing => (0, "flag", 0, 1, "borg_fast_fragile.rs:573", "#842 (review of #847): 1 = the fast-fragile Phase Door also takes the emergency landing test (escape_emergency_phase_tolerance / _horizon) like the core's other sites; 0 = no landing test (the #584 adjacent-enemy rule and the anti-summon gate still apply). At the HP it fires at, every landing grid is scary against a crow (100 of 100): on the 8 crow cases 1 = 4 dead of 8, 0 = 1 dead. Only with fast_fragile_policy = 1."),
    fast_fragile_speed_margin => (5, "speed points", 1, 40, "borg_fast_fragile.rs:353", "#842: fast = the monster's speed exceeds the Borg's by at least this many points. Monsters spread by about one point around their race's speed (mon-make.c:1158-1164): at +1 a small kobold was marked. The crow and the wild cat are +10. Hypothesis: no measurement chose 5."),
    fast_fragile_kill_turns => (2, "turns", 1, 10, "borg_fast_fragile.rs:348", "#842: fragile = the monster's HP at most this many turns of the Borg's best attack (focused planner's HP/turn, hit chance and failure included). Hypothesis: no measurement chose 2."),
    fast_fragile_damage_percent => (25, "percent of HP", 1, 1000, "borg_fast_fragile.rs:349", "#842: dangerous enough to matter = the monster's expected damage per Borg turn is at least this percent of the current HP. Hypothesis: no measurement chose 25."),
    fast_fragile_group_max => (2, "monsters", 0, 100, "borg_fast_fragile.rs:349", "#842: a monster is not fast-fragile when more than this many others of its race are in view (a swarm: cave spiders, wood spiders, crabans). Hypothesis: no measurement chose 2."),
    fast_fragile_heal_floor => (0, "percent of max HP", 0, 100, "borg_fast_fragile.rs:484", "#926: at or below this percent of the maximum HP the fast-fragile heal ban (fast_fragile_no_heal) lifts, and attack on sight steps aside for a carried cure after Phase Door and the stairs underfoot (Elf Priest 1294425391: 2/11 HP, 4 CLW, no Phase Door left). 0 retains today's play. Hypothesis: no measurement chose a value."),
    fast_fragile_escape_quantile => (0, "percent", 0, 100, "borg_fast_fragile.rs:464", "#926: 0 = \"the next turn kills\" (Phase Door, the stairs underfoot, the cornered exception) weighs the mean damage of the fast-fragile monsters within two grids (today); 1-99 = that quantile of their damage distribution before the Borg's next action (blows, hit chance and speed as the mean; floor/ceil of the energy ratio for the number of actions); 100 = its maximum. Still compared with fast_fragile_escape_percent of HP. Dwarf Necromancer 1398933929 at HP 8 against a wild cat: mean 7.6, median 8, 69th percentile 9, maximum 12. Hypothesis: no measurement chose a value."),
    fast_fragile_count_dot => (0, "flag", 0, 1, "borg_fast_fragile.rs:430", "#926: 1 = \"the next turn kills\" also counts the Borg's own poison (1 HP a world tick) and cut (1, 2 for a Severe Cut, 3 for a Deep Gash or worse) before its next action (game-world.c:513-538); when they alone take it below 0 HP before its next action (expected world ticks: 10 / the Borg's energy), the Borg drinks the carried potion that stops them before the attack, Phase Door and the stairs (a cut up to 20: CLW, CSW or CCW; a deeper cut: CSW or CCW; poison: Neutralize Poison or CCW; both: CCW), and the heal ban lifts (Half-Elf Necromancer 798380952: HP 0 and a graze). 0 retains today's play."),
    fast_fragile_contact_max => (0, "monsters", 0, 100, "borg_fast_fragile.rs:675", "#914: above 0, more than this many awake fast-fragile monsters next to the Borg is a group fight: attack on sight skips its fast-fragile attack on every marked target (one out of contact too), and the heal ban (fast_fragile_no_heal) lifts whether or not an escape is available, a group exception to #842. No hand-off to borg_caution is guaranteed: other attack-on-sight entries, the lone-target attack and the winning fight (fight_before_flight) can act first, and caution tries its heals before borg_escape, so its Phase Door is conditional. fast_fragile_group_max counts only others in view, so three cave spiders in contact passed it (seed 2008390085: the attack took the turn the control spent on Phase Door). At 0, crowded returns before counting: today's play. Compiled 0, shipped 0 (owner, 2026-11-23); 1 is the value to try, a hypothesis. With fast_fragile_by_stats = 0 only the named crow and wild cat are marked (no FRIENDS in monster.txt), so it acts only when two of them are in contact; independent spawns, a crow and a wild cat included, can still be adjacent. No test covers the combined policy."),
    view_rubble_blocks_sight => (0, "flag", 0, 1, "borg_cave_view.rs:258", "#917: 1 = passable rubble stops the Borg\'s view (borg_update_view) and the line of sight the view falls back on (borg_los_sight), as in the engine: the terrain has neither LOS nor PROJECT (lib/gamedata/terrain.txt:346-354), so update_view_one (src/cave-view.c:765-830) and los() (:85-248) stop at it. The rubble itself is still seen, as a wall is; unknown grids stay see-through. A grid in view whose sight changes when the map is read (an unknown grid found to be passable rubble, which the walk test misses) redoes the view (review of #1128, F2). C decides sight with the walk test (borg_cave_floor_grid, borg_cave_floor_bold), which counts passable rubble as floor, so grids behind it get BORG_VIEW: borg_follow_kill_aux then expects a monster on such a lit grid to be visible, deletes the record and refuses it a position belief (unseen_belief), and the Borg may aim or flow at targets it cannot see. Not shown on a recorded game yet. 0 = C\'s view, today\'s play."),
    update_dark_clears_light => (0, "flag", 0, 1, "borg_update.rs:439", "#885: 1 = a floor grid the engine shows neither glowing nor lit loses BORG_LIGHT when the map is read (the Borg's own light set is marked again in the same think, borg_update_light). C keeps the mark, and borg_update_light clears only the grids of the Borg's own light list, so a grid lit once some other way (a monster's light) stays BORG_LIGHT after it goes dark: borg_follow_kill_aux then expects an unseen monster there to be visible, deletes the record and refuses it a position belief (unseen_belief). Not shown on a recorded game yet. 0 = C's marks, today's play."),
    update_miss_wall_guess => (0, "flag", 0, 1, "borg_update.rs:1740", "#929: a ranged attack with no hit message takes the first unknown grid of its path for a wall (borg_target_unknown_wall, as in C). 1 = (a) a miss at a monster seen at the shot and in view is a miss, not a wall: a thrown or fired miss prints nothing and the missile reaches the monster's grid (player-attack.c:1101-1108, 1278-1280), so no grid is guessed and the next shot is allowed (successful_target 2, as after today's guess); (b) a guess never writes the Borg's own grid nor takes the target's grid for an unknown one: the walk starts at its first step, and when it reaches the target the step back is not written if it is the Borg's grid (C writes granite under the Borg after every miss at an adjacent target); the next shot stays allowed after every outcome. 0 = C's guess, today's play."),
    // #878: the Human Mage 461220874's white worm mass (borg-cemetery case 2026-11-22-reborn-confusion-csw-mage-461220874).
    lone_slow_attack => (0, "flag", 0, 1, "borg_lone_target.rs:94", "#878: 1 = attack on sight a lone slow monster with the best ranged attack, before retreat, back-away and the Phase Door escapes (fight_before_flight): one awake monster within lone_slow_radius and no other, not faster than the Borg, killed in at most lone_slow_casts uses of the attack the mana (charges, missiles) pays for, fail rate included, with its expected blow damage over those turns below lone_slow_damage_percent of the current HP; the other monsters' danger gate of attack on sight still applies. The spell counterpart of borg_adjacent_kill_in_one_blow (melee only). 0 retains today's play."),
    lone_slow_radius => (3, "grids", 1, 20, "borg_lone_target.rs:71", "#878: with lone_slow_attack, the target is the only awake monster within this many grids (the engine's distance). Hypothesis (issue: 1-3)."),
    lone_slow_casts => (2, "uses", 1, 10, "borg_lone_target.rs:130", "#878: with lone_slow_attack, the kill must take at most this many uses of the ranged attack (HP left over its expected damage a turn, fail rate included, rounded up) and the mana, charges or missiles must pay for them. The case: two Magic Missiles killed the worm. Hypothesis (issue: 2-3)."),
    lone_slow_damage_percent => (25, "percent of HP", 0, 1000, "borg_lone_target.rs:130", "#878: with lone_slow_attack, the target's expected blow damage a Borg turn (fight_incoming: HP, not danger; no fear prices) times the turns of the kill must stay below this percent of the current HP. Hypothesis (issue: 25-50)."),
    lone_slow_breeder_lost => (0, "flag", 0, 1, "borg_lone_target.rs:110", "#1008: with lone_slow_attack, 1 = no lone-slow attack on a breeder (MULTIPLY) while the level is lost to breeders (breeders_lost: more known than caution_breeder_hunt_max_count, or the colony cannot be stopped, under caution_breeder_policy; set only with caution_breeder_stoppable = 1, shipped on, so with it 0 this never fires), the condition the scary list's breeder attack entries already respect; the breeder policy's leave decides. Other monsters, and a level not lost, are unchanged. 0 = #878 as shipped."),
    escape_phase_lost_damage_only => (0, "flag", 0, 1, "borg_escape.rs:1203", "#878: 1 = the Phase rule (escape_phase_adjacent_rule) judges the fight lost on the danger without fear: the danger here with every danger_*_fear term, the breeder and friends bonuses (danger_breeder_*_extra_tenths, danger_friends_*_extra_tenths) and the fear maps (regional and monster fear) at 0 (params::damage_only), for both escape_phase_lost_test 0 and its fight comparisons. The Mage 461220874: danger 32 for one white worm mass (12 of it poison fear, then +150% for a breeder) against 2 without. 0 retains today's play."),
    escape_paralyser_fight_lost => (0, "flag", 0, 1, "borg_escape.rs:1209", "#1125: 1 = the Phase rule (escape_phase_adjacent_rule) judges the fight lost next to an awake known monster with a PARALYZE blow when the Borg has no Free Action (BI_FRACT 0), whatever the danger says, and the early fight (attack on sight, the lone slow monster, the winning fight) waits for caution: a paralyser as fast as the Borg strikes again as each paralysis ends (mon-blows.c:1017-1025: 3 + 1d(rlev) turns, at least 1 damage on a paralysed player; player_timed.txt:216-222 NONSTACKING), and a floating eye's gaze has no dice (danger 0 without fear, borg_danger.rs:479). The Phase Door is then allowed. 0 retains today's play."),
    // #1066: Half-Troll Priest 694417276 (Phase Door at 41/55 HP into an unexplored room: 0 scary of 100, a shrieker and four cave spiders) and Half-Elf Blackguard 2025089708 (borg-cemetery study lvl1-deaths-trackp-206cf4e7).
    escape_phase_unknown_scary => (0, "flag", 0, 1, "borg_escape.rs:1480", "#1066: 1 = in the Phase Door landing check (borg_caution_phase, every site that asks it), a sampled landing grid the Borg has never seen (FEAT_NONE) ends its attempt and counts as scary, instead of being skipped for another grid (C: unknown grids are resampled, so a Phase Door into an unexplored room counts 0 scary of 100). An unknown grid may be rock the engine never lands on, so this errs towards refusing. Not while HP is below escape_phase_unknown_hp_percent of the maximum (a real emergency). The teleport check is unchanged. 0 retains today's play. Hypothesis: no measurement chose it."),
    escape_phase_unknown_hp_percent => (50, "percent of max HP", 0, 1000, "borg_escape.rs:1481", "#1066: with escape_phase_unknown_scary, unknown landing grids count as scary only while the current HP is at least this percent of the maximum; below it they are skipped as before. 0 = always scary. Hypothesis (issue: 50)."),
    escape_phase_newcomer_wait => (0, "flag", 0, 1, "borg_escape.rs:1286", "#1066: with escape_phase_adjacent_rule, 1 = an adjacent fight the rule judged lost (escape_phase_lost_test) is not lost when HP is at least escape_phase_newcomer_hp_percent of the maximum, an awake known monster farther than one grid endangers the Borg where it stands (borg_danger_one_kill > 0), and #759's fight race against the adjacent awake monsters alone is winning (fight_race; any check it cannot certify, such as the Borg's own status, an unseen attacker or unknown damage, keeps the Phase Door): the verdict is Fight and the emergency Phase Door waits for the newcomer to arrive. Read only when the rule judged the fight lost. 0 retains today's play. Hypothesis: no measurement chose it."),
    escape_phase_newcomer_hp_percent => (50, "percent of max HP", 0, 1000, "borg_escape.rs:1290", "#1066: only with escape_phase_newcomer_wait = 1. The lowest health at which the Borg stays and fights, instead of reading Phase Door, when a new monster appears at range: current HP as a percent of maximum HP. At 50 it waits only at half health or more: the Priest 694417276 at 41/55 HP (75%) may wait, and at 25/55 (45%) it reads Phase Door as before. Lower = it waits at lower health (braver); 0 = at any health; above 100 = it never waits. Hypothesis (issue: 50)."),
    escape_phase_unknown_tolerance => (75, "count", 0, 1000, "borg_escape.rs:930", "#1066 (review of #1075): with escape_phase_unknown_scary and HP at or above escape_phase_unknown_hp_percent, a Phase Door site with no landing check (escape 1.2 critical, 1.3 last ditch, the twitchy ones, fast-fragile with fast_fragile_phase_landing 0) samples 100 landings (as the landing check does) and refuses when more than this many first meet a grid the Borg has never seen; when a landing check already counted them in the same think at the same grid (a refused emergency falling through to the critical phase), that count is reused instead of a new draw, judged by the stricter of that site's tolerance and this one (#1094). 75 = escape_emergency_phase_tolerance as compiled. Shoot N Scoot's own sampler counts never-seen grids as scary instead. Hypothesis: no measurement chose it."),
    escape_phase_newcomer_turn_percent => (100, "percent of current HP", 0, 1000, "borg_escape.rs:1295", "#1066 (review of #1075): only with escape_phase_newcomer_wait = 1. A second safety check on the wait: the most damage the Borg could take before it acts again, as a percent of its current HP. It adds the damage a turn of the monsters next to it (#759's fight race) and the full possible one-turn damage of the monsters farther away (no discount for spells they cast rarely; fear terms off when escape_phase_lost_damage_only = 1). The Borg waits only if that sum is below this percent of its current HP; otherwise it reads Phase Door as before. At 100 it waits only if the next turn cannot kill it: the Priest 694417276 faced 13.5 (a soldier next to it) + 22 (an apprentice at range) = 35.5 against 41 HP, so it waited; at 30 HP it would have read Phase Door. Lower = stricter (50: the next turn may cost at most half the HP); higher = braver. Hypothesis: no measurement chose it."),
    attack_reserve_spell_damage_only => (0, "flag", 0, 1, "borg_fight_attack.rs:3565", "#878: 1 = the reserve spell's \"must be dangerous\" test (danger here at least attack_reserve_spell_danger_multiplier x avoidance) reads the danger without fear, as escape_phase_lost_damage_only does. 0 retains today's play."),
    attack_reserve_faint_percent => (0, "percent of HP", 0, 1000, "borg_fight_attack.rs:3472", "#878: no reserve-spell cast (casting beyond the mana: player-spell.c:541-553 faints the caster for randint1(5 x oops + 1) world ticks of 10 game turns, player-util.c:835-842, game-world.c:651, :1080) when the expected blow damage the awake monsters deal during the mean faint, (5 x oops + 2) / 2 world ticks (energy / 10 Borg turns a tick at the Borg's speed), reaches this percent of the current HP: each monster from the turn it can reach the Borg (distance and speed), the target only if the cast fails (its fail rate). The Mage 461220874 fainted 11 times next to biting worms. 0 = off, today's play. Hypothesis: no measurement chose a value."),
    caution_unseen_bump_phase_turns => (0, "game turns", 0, 100000, "borg_caution.rs:9315", "#878: the unseen-bump Phase Door (caution_unseen_bump_response) only when the bump at that grid is at most this many game turns old (caution_unseen_bump_turns keeps the grid occupied longer: 200 shipped). The Mage 461220874 read its last Phase Door 132 game turns after its bump, at danger 0. 0 = off (the grid's caution_unseen_bump_turns), today's play."),
    caution_unseen_bump_phase_min_danger => (0, "danger", 0, 100000, "borg_caution.rs:9315", "#878: the unseen-bump Phase Door only when the danger here is at least this (the unseen monster itself has no estimate; the Mage 461220874 read two Phase Doors at danger 0 and 1, nearest awake monster 28 grids away). 0 = off, today's play."),
    cave_light_engine_radius => (0, "flag", 0, 1, "borg_cave_light.rs:130", "#877: 1 = the grids the Borg marks lit by its own light reach the engine's radius, cur_light - 1 (src/cave-view.c:658: a torch lights distance 1, not 2), so a monster two grids away in the dark is no longer expected visible and deleted when unseen (Mage 461220874, 44 of 50 worm forgets). A necromancer by unlight (cur_light <= 1) keeps C's radius. With 1, a non-necromancer whose light slot is empty still lights the map from other equipped gear (BI_LIGHT > 0, player-calcs.c:1624-1644; #888, kept by the owner 2026-11-22). #884: with 1 the lit grids are also the engine's shape (distance() <= radius: radius 2 lights (2,1), not C's (2,2), and C's (3,3) corners go), and the dark flow (flow_dark_2), the dark frontier (#796), Light Beam and Call Light's radius branches read the same radius (a torch takes C's radius-1 corner test, a lantern the 5x5 count; BI_LIGHT 1 keeps the corner test). 0 = C's radius (cur_light) and shape, today's play."),
    town_keep_last_class_book => (0, "flag", 0, 1, "borg_town_shadow.rs:884", "#1004: 1 protects the last owned copy of each spellbook the current class can browse from town disposal, regardless of level or mana. Home storage is allowed. 0 disables the protection."),
    town_book_lookahead_levels => (0, "levels", 0, 50, "borg_town_capability.rs:939", "#1004: the town planner values a class book it owns (pack or home) when one of its spells is castable now; above 0, also when a spell is learnt within this many levels, on the level alone (the mana is not compared, so a wide window can keep a book whose spell the class cannot pay for when it is learnt). At CL1 Rogues and Rangers (0 SP) and Half-Elf, Elf and Half-Troll Paladins (1 SP, Bless costs 2) value their first book at nothing. Purchases are unchanged. 0 = off, today's play."),
    town_hunt => (0, "flag", 0, 1, "borg_town_hunt.rs:22", "#789: in town let the fight forecast govern attacks and approaches to the town-melee-only category, prioritize the highest danger, and avoid Farmer Maggot. 0 retains the existing town scare rules."),
    town_hunt_ranged_bonus_percent => (0, "percent", 0, 1000, "borg_town_hunt.rs:198", "#789: extra percent applied to the expected damage score of a ranged attack among winning town plans. Melee remains available."),
    kit_leash => (0, "flag", 0, 1, "borg_kit.rs:201", "#840 (owner, 2026-11-21: \"the smaller the kit the smaller the leash\"): 1 = the leash follows the kit score: kit_leash_min + (C's leash - kit_leash_min) x score^kit_leash_power, while the kit is short, at any depth and level up to kit_max_level (the hunts and item detours the leash refuses stay refused). 0 = C's leash, today's play."),
    kit_leash_min => (1, "steps", 0, 20, "borg_kit.rs:314", "#840: the leash of a Borg with no basic kit at all (0-2 in the issue): it stays on or next to the stairs. Used with kit_leash."),
    kit_leash_power => (1, "power", 1, 4, "borg_kit.rs:314", "#840: the exponent of the score in the kit leash (1 = linear, 2 = squared: a half kit gets a quarter of the room). Integer, so a game replays from its keys. Used with kit_leash."),
    // borg-reborn-rs (#1061, owner proposal from the level-1 death review of 206cf4e7): the stair_scum exploration mode.
    stair_scum => (0, "flag", 0, 1, "borg_stair_scum.rs:152", "#1061 (owner: \"explore the level in a way that they can go back to the stairs faster than any character can come at them\"): 1 = while max HP < stair_scum_max_hp (per class) + stair_scum_max_hp_race_delta (per race) and the Borg cannot detect monsters (a Detect Monsters, Reveal Monsters or Detection spell, a Rod of Detection), in the dungeon: a grid is safe when a monster first seen there (the engine's light radius, a lit room's nearest unseen entry, 1 behind a door) and acting stair_scum_speed_ratio times per player action lands fewer than HP / stair_scum_contact_damage attacks on the walk to the nearest known staircase of either kind (known, trap-free grids) and the stair command; the dark flows, item and monster targets and every voluntary flow step stay in the safe area joined to a staircase, nearest the staircase first; outside it the Borg walks back onto the staircase; a door it opens is peeked through (step back, wait a turn); a door seen opened near the area, or a door sound no changed door explains, holds it on the staircase for stair_scum_alarm_turns; nothing left in the area: it takes a staircase (DL1-2 preferred, within the depth plan); on a staircase, a threat the fight race does not take is left by the stairs, and it rests there; with no known route to a staircase it explores toward one and chases nothing. 0 = today's play. Hypothesis: no measurement chose it."),
    stair_scum_max_hp => (25, "HP", 1, 1000, "borg_stair_scum.rs:159", "#1061: with stair_scum, the mode applies while the max HP is below this plus stair_scum_max_hp_race_delta. Per class with @<class> or __<class> (the owner: 25, 30 for a warrior, ...). Hypothesis: arms at 20, 25 and 30."),
    stair_scum_max_hp_race_delta => (0, "HP", -100, 100, "borg_stair_scum.rs:159", "#1061 (owner, 2026-11-22: \"race would be a delta\"): added to stair_scum_max_hp. Per race with @<race> or __<race> (p_race.txt names in lower case, - as _: human, half_elf, elf, hobbit, gnome, dwarf, half_orc, half_troll, dunadan, high_elf, kobold), e.g. __hobbit = 3, __half_troll = -5."),
    stair_scum_speed_ratio => (2, "actions per player action", 1, 10, "borg_stair_scum.rs:559", "#1061: with stair_scum, the monster actions per player action of the fastest monster that matters at the depth (2: the +10 monsters of DL1-2, Grip, Fang, crows, wild cats at speed:120); it may also act ratio - 1 times before the Borg's first action (game-world.c:1033-1067)."),
    stair_scum_contact_damage => (4, "HP per attack", 1, 1000, "borg_stair_scum.rs:539", "#1061 (review of #1061, P1): with stair_scum, the damage of one monster attack action (a round of blows): the retreat is safe while the attacks on the way times this stay below the HP (4: a wild cat's or a crow's two 1d3, Grip's 1d4 at its maximum)."),
    stair_scum_slack_steps => (0, "steps", 0, 50, "borg_stair_scum.rs:541", "#1061 (owner, 2026-11-23: \"a tunable parameter to make it less strict\"): with stair_scum, the steps a grid may lie beyond the strict safe walk (the survival test counts the walk this many steps shorter). 0 = the strict count of the review of #1061 (a torch at 12 HP: 1 step); 2 = the owner's \"two or three steps\". Hypothesis: zoo arms at 0, 1, 2 and 3."),
    stair_scum_slack_per_hp_tenths => (0, "tenths of a step per max HP", 0, 100, "borg_stair_scum.rs:542", "#1061 (owner, 2026-11-23: \"a rate to make the scum dependent on hp/class\"): with stair_scum, slack in tenths of a step per point of max HP, added to stair_scum_slack_steps: the mode loosens as the character grows. 1 = a step per 10 max HP (12 HP: 1, 24 HP: 2). Per class with __<class>, per race with __<race>. 0 = none. Hypothesis: zoo arms at 0, 1 and 2."),
    stair_scum_slack_per_level_percent => (50, "percent of a step per level", 0, 1000, "borg_stair_scum.rs:544", "#1061 (owner, 2026-11-23: \"slack grows by decile by level, say 50 means that druids at level 3 get to slack 2\"): with stair_scum, percent of a step of slack per character level above 1, rounded down, added to stair_scum_slack_steps and the HP rate. 50: half a step per level (a Druid at slack 1: 1 at CL1, 2 at CL3, 3 at CL5). Per class with __<class>, per race with __<race>. 0 = none; 50 by default (the owner, 2026-11-23: \"stair_scum_slack_per_level_percent: 50 as default\"), which acts only with stair_scum on. Hypothesis."),
    stair_scum_alarm_turns => (100, "game turns", 0, 10000, "borg_stair_scum.rs:931", "#1061: with stair_scum, after a known closed door seen open next to the safe area, or \"You hear a door burst open!\" with no changed door to place it, the Borg stands on the staircase this many game turns (100: ten turns at normal speed); 0 = no alarm."),
    kit_max_level => (15, "level", 1, 51, "borg_kit.rs:279", "#840: the kit mode (every kit_ setting but the start reserve) applies up to this character level (the issue's band tops out at 15)."),
    kit_band_split_level => (5, "level", 1, 51, "borg_kit.rs:178", "#840: character levels up to this use kit_cure_target and kit_phase_target, higher levels the _high ones (the issue's CL1-5 and CL6-15)."),
    kit_cure_target => (3, "count", 0, 20, "borg_kit.rs:180", "#840: cure potions (CLW + CSW + CCW; a cure spell counts as plenty) that make the cure share 1 at CL up to kit_band_split_level. A hypothesis, from the issue's example."),
    kit_phase_target => (2, "count", 0, 20, "borg_kit.rs:181", "#840: Phase Door (scrolls; the spell or an activation counts as plenty) that make the Phase Door share 1 at CL up to kit_band_split_level. A hypothesis, from the issue's example."),
    kit_cure_target_high => (5, "count", 0, 20, "borg_kit.rs:180", "#840: kit_cure_target above kit_band_split_level. A hypothesis."),
    kit_phase_target_high => (4, "count", 0, 20, "borg_kit.rs:181", "#840: kit_phase_target above kit_band_split_level. A hypothesis."),
    kit_missile_target => (20, "count", 0, 200, "borg_kit.rs:182", "#840: missiles for the launcher (flasks of oil not counted) that make the ranged share 1. A Mage's or Necromancer's attack spell makes it 1 too. The share is the larger of this and the flask share."),
    kit_flask_target => (5, "count", 0, 100, "borg_kit.rs:183", "#840: flasks of oil to throw that make the ranged share 1 (the issue: at least 20 missiles or 5 flasks)."),
    kit_light_turns_target => (1000, "turns", 0, 100000, "borg_kit.rs:184", "#840: turns of light, in the wielded light and the spares (torches by their turns, flasks of oil 7500 each when a lantern is wielded), that make the light share 1. A light that needs no fuel, or a Necromancer's unlight, is always 1."),
    kit_food_target => (3, "count", 0, 20, "borg_kit.rs:185", "#840: food (the core's BI_FOOD: rations, and the like) that makes the food share 1."),
    kit_weight_cure => (1, "weight", 0, 100, "borg_kit.rs:191", "#840: the weight of the cure share in the kit score (all five 1 = the plain mean)."),
    kit_weight_phase => (1, "weight", 0, 100, "borg_kit.rs:192", "#840: the weight of the Phase Door share in the kit score."),
    kit_weight_ranged => (1, "weight", 0, 100, "borg_kit.rs:193", "#840: the weight of the ranged (missiles or flasks) share in the kit score."),
    kit_weight_light => (1, "weight", 0, 100, "borg_kit.rs:194", "#840: the weight of the light share in the kit score."),
    kit_weight_food => (1, "weight", 0, 100, "borg_kit.rs:195", "#840: the weight of the food share in the kit score."),
    kit_descent_min_percent => (0, "percent", 0, 100, "borg_kit.rs:202", "#840: no descent (to a depth below the current one and below DL1, from the dungeon) while the kit score, in percent, is below this. 0 = off, today's play."),
    kit_leave_danger_percent => (0, "percent of HP", 0, 1000, "borg_kit.rs:203", "#840: a Borg short of kit takes the stairs it stands on (up) at the first danger of at least this share of its current HP, instead of fighting, and starts fleeing to the stairs when the danger is that high and it is off them (the #614 rule's kit twin, borg_caution.rs). 0 = off, today's play."),
    restock_healing => (0, "flag", 0, 1, "borg_healing_restock.rs:131", "#1214: return at zero healing potions or before depletion when the last observed funding and stock can restore the reserve. 0 retains the historical rules."),
    restock_healing_min_potions => (2, "potions", 1, 10, "borg_healing_restock.rs:135", "#1214: minimum known healing potions to carry. Below this count, restock before depletion if the Borg can cover the deficit. 1 only returns on depletion."),
    kit_restock => (0, "flag", 0, 1, "borg_kit.rs:204", "#840 remedy a: the kit is part of borg_restock at every depth, not only DL6+: a trip to town when the kit score is below kit_restock_below_percent and the purse has grown by kit_restock_min_gold since the Borg left town, and (with kit_restock_light_turns) when the light is short of turns with no spare. 0 = off, today's play."),
    kit_restock_below_percent => (75, "percent", 0, 100, "borg_kit.rs:385", "#840: with kit_restock (or kit_money_first), the kit score, in percent, below which the Borg goes to town if it can pay. A hypothesis."),
    kit_restock_min_gold => (50, "gold", 0, 100000, "borg_kit.rs:365", "#840: the gold gained since the Borg left town that makes a trip worth it (about two potions of Cure Light Wounds, 20 each, and a Phase Door, 15: object.txt). Leaving town short of kit because a store lacked stock does not bring the Borg back until it has found this much more."),
    kit_restock_light_turns => (0, "turns", 0, 100000, "borg_kit.rs:384", "#840: with kit_restock, restock when the wielded light and its spares hold fewer turns than this and the purse allows it. 0 = off. C's rule asks for a trip only once the light is out."),
    kit_start_reserve => (0, "flag", 0, 1, "borg_kit.rs:420", "#840 remedy b: the live town planner keeps kit_start_cure_reserve cures and kit_start_phase_reserve Phase Door as requirements (as town_phase_reserve and town_heal_reserve_hp_percent do), bought before missiles and oil. Per class with @<class>. 0 = off, today's play."),
    kit_start_cure_reserve => (3, "count", 0, 20, "borg_kit.rs:424", "#840: with kit_start_reserve, cure potions the town planner keeps, in HP of kit_start_cure_hp each."),
    kit_start_cure_hp => (20, "hp", 1, 1000, "borg_kit.rs:424", "#840: the hit points one reserved cure potion stands for (Cure Light Wounds heals 20: object.txt)."),
    kit_start_phase_reserve => (2, "count", 0, 10, "borg_kit.rs:420", "#840: with kit_start_reserve, Phase Door scrolls the town planner keeps (the larger of this and town_phase_reserve applies)."),
    kit_money_first => (0, "flag", 0, 1, "borg_kit.rs:205", "#840 remedy c: at character level up to kit_money_first_max_level, short of kit and too poor to fix it, no descent below DL1 and a leash of at least kit_money_leash while it looks for gold near the stairs; once the purse has grown by kit_restock_min_gold it goes to town (restock). The town hunt (town_hunt) and money scum (borg_money_scum_amount) are the other ways. 0 = off, today's play."),
    kit_money_first_max_level => (2, "level", 1, 51, "borg_kit.rs:298", "#840: the character level up to which kit_money_first applies (the issue: CL1-2)."),
    kit_money_leash => (6, "steps", 0, 250, "borg_kit.rs:317", "#840: with kit_money_first, the least leash while the Borg looks for gold (the kit leash alone may be 1: nothing within reach)."),
    // #403: final attack ranking, neutral for every supported class/family.
    attack_preference_melee_permille => (1000, "utility_permille", 500, 2000, "borg_fight_attack.rs:5604", "Final returned attack utility preference for the melee family; per class with attack_preference_melee_permille@<class> or the class files (#520); floor rounding, no eligibility changes."),
    attack_preference_projectile_permille => (1000, "utility_permille", 500, 2000, "borg_fight_attack.rs:5604", "Final returned attack utility preference for the projectile family; per class with attack_preference_projectile_permille@<class> or the class files (#520); floor rounding, no eligibility changes."),
    attack_preference_thrown_permille => (1000, "utility_permille", 500, 2000, "borg_fight_attack.rs:5604", "Final returned attack utility preference for the thrown family; per class with attack_preference_thrown_permille@<class> or the class files (#520); floor rounding, no eligibility changes."),
    attack_preference_spell_permille => (1000, "utility_permille", 500, 2000, "borg_fight_attack.rs:5604", "Final returned attack utility preference for the spell family; per class with attack_preference_spell_permille@<class> or the class files (#520); floor rounding, no eligibility changes."),
    attack_preference_device_permille => (1000, "utility_permille", 500, 2000, "borg_fight_attack.rs:5604", "Final returned attack utility preference for the device family; per class with attack_preference_device_permille@<class> or the class files (#520); floor rounding, no eligibility changes."),
    attack_preference_activation_permille => (1000, "utility_permille", 500, 2000, "borg_fight_attack.rs:5604", "Final returned attack utility preference for the activation family; per class with attack_preference_activation_permille@<class> or the class files (#520); floor rounding, no eligibility changes."),
    caution_bounce_healthy_tenths => (7, "tenths", 0, 100, "borg_caution.rs:105", "Controls bounce healthy tenths."),
    caution_bounce_limit => (300, "turns", 0, 100000, "borg_caution.rs:111", "Controls bounce limit."),
    caution_caster_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:166", "Controls caster stat vote."),
    caution_priest_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:169", "Controls priest stat vote."),
    caution_druid_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:172", "Controls druid stat vote."),
    caution_necromancer_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:175", "Controls necromancer stat vote."),
    caution_warrior_stat_vote => (1, "votes", 0, 100, "borg_caution.rs:178", "Controls warrior stat vote."),
    caution_stat_fix_hp_high => (850, "hp", 0, 10000, "borg_caution.rs:180", "Controls stat fix hp high."),
    caution_stat_fix_vote_high => (1, "votes", 0, 100, "borg_caution.rs:181", "Controls stat fix vote high."),
    caution_stat_fix_hp_low => (700, "hp", 0, 10000, "borg_caution.rs:183", "Controls stat fix hp low."),
    caution_stat_fix_vote_low => (3, "votes", 0, 100, "borg_caution.rs:184", "Controls stat fix vote low."),
    caution_priest_stat_fix_sp => (100, "sp", 0, 10000, "borg_caution.rs:187", "Controls priest stat fix sp."),
    caution_priest_stat_fix_votes => (5, "votes", 0, 100, "borg_caution.rs:190", "Controls priest stat fix votes."),
    caution_mage_stat_fix_sp => (100, "sp", 0, 10000, "borg_caution.rs:193", "Controls mage stat fix sp."),
    caution_mage_stat_fix_votes => (5, "votes", 0, 100, "borg_caution.rs:196", "Controls mage stat fix votes."),
    caution_heal_spell_fail => (15, "percent", 0, 100, "borg_caution.rs:266", "Controls heal spell fail."),
    // #328: fixed potion healing, donor 1443c2d1 borg_caution.rs (M13 patch0133).
    caution_clw_estimate_percent => (0, "percent", 0, 100, "borg_caution.rs:273", "Controls clw estimate percent."),
    caution_clw_estimate_min => (20, "hp", 0, 10000, "borg_caution.rs:265", "Controls clw estimate min."),
    caution_csw_estimate_percent => (0, "percent", 0, 100, "borg_caution.rs:269", "Controls csw estimate percent."),
    caution_csw_estimate_min => (40, "hp", 0, 10000, "borg_caution.rs:270", "Controls csw estimate min."),
    caution_ccw_estimate_percent => (0, "percent", 0, 100, "borg_caution.rs:274", "Controls ccw estimate percent."),
    caution_ccw_estimate_min => (60, "hp", 0, 10000, "borg_caution.rs:275", "Controls ccw estimate min."),
    caution_heal_estimate_percent => (35, "percent", 0, 100, "borg_caution.rs:280", "Controls heal estimate percent."),
    caution_heal_estimate_min => (300, "hp", 0, 10000, "borg_caution.rs:281", "Controls heal estimate min."),
    caution_healing_rod_fail => (500, "permille", 0, 1000, "borg_caution.rs:289", "Controls healing rod fail."),
    caution_ranged_vs_immobile => (0, "mode", 0, 3, "borg_caution.rs:1913", "#646, #713 (owner): immobile creatures (a monster that never moves and attacks with its blows). 1 (#646) = next to one, step away when confused (up to 3 steps) or carrying a ranged attack (1 step), and hold the confusion cure next to it. 2 (#713; shipped 0) = also keep one whose blows do more than plain damage (poison, confusion, paralysis, fear, drain...: 23 of the 25 in monster.txt, 15 of them at DL10 or shallower and 8 deeper, down to Wiruin at DL80) at range: never melee it, no hunt walk to it, flows keep out of its reach (stair flows may pass, never onto its grid), and always step away from it, while fleeing too, up to 3 times; ranged attacks kill it from afar. 3 = as 2 for every immobile attacker. #852 (owner: \"if it is protected this mode needs not to apply\"): in modes 1-3, an immobile attacker with a blow that does more than plain damage, whose such blows the character is all protected from (paralysis: Free Action; confusion, poison (also the temporary resistance), blindness, fear: the resistance; drains, theft, disenchantment and elements have no protection), is not handled at all: not kept at range, no step away, no cure held, not avoided by the step; a Free Action Borg fights a floating eye. Plain-damage ones (a grey mold) are handled as before. 0 = C."),
    caution_confused_heal_loss => (80, "percent", 0, 100, "borg_caution.rs:304", "Controls confused heal loss."),
    caution_confused_star_loss => (85, "percent", 0, 100, "borg_caution.rs:311", "Controls confused star loss."),
    caution_confused_star_danger_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:312", "Controls confused star danger multiplier."),
    caution_confused_staff_fail => (650, "permille", 0, 1000, "borg_caution.rs:338", "Controls confused staff fail."),
    caution_confused_cure_danger_tenths => (15, "tenths", 0, 100, "borg_caution.rs:340", "Controls confused cure danger tenths."),
    caution_confused_escape_danger_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:351", "Controls confused escape danger multiplier."),
    caution_blind_cure_chance => (85, "percent", 0, 100, "borg_caution.rs:368", "Controls blind cure chance."),
    caution_blind_escape_danger_tenths => (25, "tenths", 0, 100, "borg_caution.rs:372", "Controls blind escape danger tenths."),
    caution_blind_heal_loss => (300, "hp", 0, 10000, "borg_caution.rs:377", "Controls blind heal loss."),
    caution_blind_warrior_hp_divisor => (4, "divisor", 1, 100, "borg_caution.rs:385", "Controls blind warrior hp divisor."),
    caution_blind_confused_star_loss => (400, "hp", 0, 10000, "borg_caution.rs:402", "Controls blind confused star loss."),
    caution_blind_confused_danger_multiplier => (5, "multiplier", 0, 20, "borg_caution.rs:403", "Controls blind confused danger multiplier."),
    caution_blind_confused_min_loss => (100, "hp", 0, 10000, "borg_caution.rs:404", "Controls blind confused min loss."),
    caution_questor_heal_hp => (700, "hp", 0, 10000, "borg_caution.rs:413", "Controls questor heal hp."),
    caution_questor_holy_word_hp => (250, "hp", 0, 10000, "borg_caution.rs:414", "Controls questor holy word hp."),
    caution_questor_holy_word_fail => (14, "percent", 0, 100, "borg_caution.rs:414", "Controls questor holy word fail."),
    caution_questor_life_stat_votes => (5, "votes", 0, 100, "borg_caution.rs:416", "Controls questor life stat votes."),
    caution_questor_life_loss => (500, "hp", 0, 10000, "borg_caution.rs:418", "Controls questor life loss."),
    caution_questor_emergency_holy_hp => (250, "hp", 0, 10000, "borg_caution.rs:430", "Controls questor emergency holy hp."),
    caution_questor_emergency_holy_fail => (5, "percent", 0, 100, "borg_caution.rs:430", "Controls questor emergency holy fail."),
    caution_questor_safe_holy_hp => (550, "hp", 0, 10000, "borg_caution.rs:431", "Controls questor safe holy hp."),
    caution_questor_safe_holy_fail => (15, "percent", 0, 100, "borg_caution.rs:431", "Controls questor safe holy fail."),
    caution_questor_healing_fail => (15, "percent", 0, 100, "borg_caution.rs:432", "Controls questor healing fail."),
    caution_magi_sp_divisor => (5, "divisor", 1, 100, "borg_caution.rs:442", "Controls magi sp divisor."),
    caution_magi_chance => (50, "percent", 0, 100, "borg_caution.rs:443", "Controls magi chance."),
    caution_restore_mana_sp_divisor => (10, "divisor", 1, 100, "borg_caution.rs:452", "Controls restore mana sp divisor."),
    caution_restore_mana_sp => (70, "sp", 0, 10000, "borg_caution.rs:453", "Controls restore mana sp."),
    caution_restore_mana_maxsp => (200, "sp", 0, 10000, "borg_caution.rs:453", "Controls restore mana maxsp."),
    caution_restore_mana_danger_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:456", "Controls restore mana danger multiplier."),
    caution_restore_stat_votes => (5, "votes", 0, 100, "borg_caution.rs:479", "Controls restore stat votes."),
    caution_restore_stat_hp => (650, "hp", 0, 10000, "borg_caution.rs:481", "Controls restore stat hp."),
    caution_heal_unique_chance_bonus => (10, "percent", 0, 100, "borg_caution.rs:500", "Controls heal unique chance bonus."),
    caution_heal_lethal_chance_bonus => (75, "percent", 0, 100, "borg_caution.rs:505", "Controls heal lethal chance bonus."),
    caution_heal_nonpriest_chance_penalty => (25, "percent", 0, 100, "borg_caution.rs:507", "Controls heal nonpriest chance penalty."),
    caution_heal_risky_chance_penalty => (5, "percent", 0, 100, "borg_caution.rs:512", "Controls heal risky chance penalty."),
    caution_heal_band1_max_loss => (15, "percent", 0, 100, "borg_caution.rs:515", "Controls heal band1 max loss."),
    caution_heal_band1_skip_chance => (98, "percent", 0, 100, "borg_caution.rs:516", "Controls heal band1 skip chance."),
    caution_heal_band2_min_loss => (16, "percent", 0, 100, "borg_caution.rs:517", "Controls heal band2 min loss."),
    caution_heal_band2_max_loss => (25, "percent", 0, 100, "borg_caution.rs:517", "Controls heal band2 max loss."),
    caution_heal_band2_skip_chance => (95, "percent", 0, 100, "borg_caution.rs:519", "Controls heal band2 skip chance."),
    caution_heal_band3_min_loss => (26, "percent", 0, 100, "borg_caution.rs:520", "Controls heal band3 min loss."),
    caution_heal_band3_max_loss => (50, "percent", 0, 100, "borg_caution.rs:520", "Controls heal band3 max loss."),
    caution_heal_band3_skip_chance => (80, "percent", 0, 100, "borg_caution.rs:522", "Controls heal band3 skip chance."),
    caution_heal_band4_min_loss => (51, "percent", 0, 100, "borg_caution.rs:523", "Controls heal band4 min loss."),
    caution_heal_band4_max_loss => (65, "percent", 0, 100, "borg_caution.rs:523", "Controls heal band4 max loss."),
    caution_heal_band4_skip_chance => (50, "percent", 0, 100, "borg_caution.rs:525", "Controls heal band4 skip chance."),
    caution_heal_band5_min_loss => (66, "percent", 0, 100, "borg_caution.rs:526", "Controls heal band5 min loss."),
    caution_heal_band5_max_loss => (74, "percent", 0, 100, "borg_caution.rs:526", "Controls heal band5 max loss."),
    caution_heal_band5_skip_chance => (25, "percent", 0, 100, "borg_caution.rs:528", "Controls heal band5 skip chance."),
    caution_heal_band6_min_loss => (75, "percent", 0, 100, "borg_caution.rs:529", "Controls heal band6 min loss."),
    caution_heal_band6_skip_chance => (1, "percent", 0, 100, "borg_caution.rs:530", "Controls heal band6 skip chance."),
    caution_clw_min_loss => (30, "percent", 0, 100, "borg_caution.rs:540", "Controls clw min loss."),
    caution_clw_max_loss => (40, "percent", 0, 100, "borg_caution.rs:541", "Controls clw max loss."),
    caution_clw_max_level => (10, "level", 1, 51, "borg_caution.rs:541", "Controls clw max level."),
    caution_clw_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:543", "Controls clw danger divisor."),
    caution_csw_min_loss => (40, "percent", 0, 100, "borg_caution.rs:552", "Controls csw min loss."),
    caution_csw_max_loss => (50, "percent", 0, 100, "borg_caution.rs:553", "Controls csw max loss."),
    caution_csw_max_level => (20, "level", 1, 51, "borg_caution.rs:553", "Controls csw max level."),
    caution_csw_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:555", "Controls csw danger divisor."),
    caution_ccw_min_loss => (50, "percent", 0, 100, "borg_caution.rs:564", "Controls ccw min loss."),
    caution_ccw_max_loss => (55, "percent", 0, 100, "borg_caution.rs:564", "Controls ccw max loss."),
    caution_ccw_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:566", "Controls ccw danger divisor."),
    caution_emergency_ccw_hp => (50, "hp", 0, 10000, "borg_caution.rs:576", "Controls emergency ccw hp."),
    caution_deep_ccw_depth => (80, "depth", 0, 127, "borg_caution.rs:586", "Controls deep ccw depth."),
    caution_deep_ccw_danger => (50, "danger", 0, 10000, "borg_caution.rs:587", "Controls deep ccw danger."),
    caution_deep_ccw_loss => (20, "percent", 0, 100, "borg_caution.rs:588", "Controls deep ccw loss."),
    caution_heal_device_min_loss => (55, "percent", 0, 100, "borg_caution.rs:596", "Controls heal device min loss."),
    caution_conserve_heal_depth => (98, "depth", 0, 127, "borg_caution.rs:616", "Controls conserve heal depth."),
    caution_heal_potion_min_loss => (50, "percent", 0, 100, "borg_caution.rs:626", "Controls heal potion min loss."),
    caution_heal_all_min_loss => (60, "percent", 0, 100, "borg_caution.rs:640", "Controls heal all min loss."),
    caution_heal_unique_reserve_min_loss => (65, "percent", 0, 100, "borg_caution.rs:659", "Controls heal unique reserve min loss."),
    caution_heal_last_resort_loss => (75, "percent", 0, 100, "borg_caution.rs:685", "Controls heal last resort loss."),
    caution_cure_safe_danger_tenths => (2, "tenths", 0, 100, "borg_caution.rs:699", "Controls cure safe danger tenths."),
    caution_poison_cure_hp_divisor => (2, "divisor", 1, 100, "borg_caution.rs:706", "Controls poison cure hp divisor."),
    caution_poison_cure_fail => (60, "percent", 0, 100, "borg_caution.rs:708", "Controls poison cure fail."),
    caution_poison_herbal_fail => (60, "percent", 0, 100, "borg_caution.rs:709", "Controls poison herbal fail."),
    caution_poison_healing_fail => (60, "percent", 0, 100, "borg_caution.rs:724", "Controls poison healing fail."),
    caution_poison_holy_fail => (60, "percent", 0, 100, "borg_caution.rs:725", "Controls poison holy fail."),
    caution_poison_critical_hp => (2, "hp", 0, 10000, "borg_caution.rs:744", "Controls poison critical hp."),
    caution_poison_critical_hp_divisor => (20, "divisor", 1, 100, "borg_caution.rs:746", "Controls poison critical hp divisor."),
    caution_poison_reserve_hp_divisor => (20, "divisor", 1, 100, "borg_caution.rs:784", "Controls poison reserve hp divisor."),
    caution_cut_cure_hp_divisor => (3, "divisor", 1, 100, "borg_caution.rs:799", "Controls cut cure hp divisor."),
    caution_cut_cure_chance => (20, "percent", 0, 100, "borg_caution.rs:800", "Controls cut cure chance."),
    caution_cut_critical_ccw_hp => (10, "hp", 0, 10000, "borg_caution.rs:805", "Controls cut critical ccw hp."),
    caution_cut_critical_hp => (2, "hp", 0, 10000, "borg_caution.rs:814", "Controls cut critical hp."),
    caution_cut_critical_hp_divisor => (20, "divisor", 1, 100, "borg_caution.rs:816", "Controls cut critical hp divisor."),
    caution_cut_reserve_hp_divisor => (20, "divisor", 1, 100, "borg_caution.rs:839", "Controls cut reserve hp divisor."),
    caution_flee_stairs_before_recall => (0, "flag", 0, 1, "borg_caution.rs:5145", "#539 (owner): 1 = a fleeing Borg that can walk to a free staircase (the flee flows' stair flow reaches it) does not read Word of Recall (the flee flow walks it to the stairs); 0 = Recall first (C's behaviour)."),
    caution_flee_recall => (1, "flag", 0, 1, "borg_caution.rs:5258", "#557 (owner): 1 = C: a fleeing Borg reads Word of Recall as its escape (after #539's stairs and ranged checks), and Recall counts as an easy escape for the #503/#530 stand tests; 0 = only on a level swamped by breeders (caution_flee_recall_breeders) or when chased by several monsters (caution_flee_recall_chasers), else the flight walks."),
    caution_flee_recall_breeders => (5, "monsters", 0, 100, "borg_caution.rs:5294", "#557: with caution_flee_recall 0, this many breeders known on the level make Word of Recall an escape again; 0 = never for breeders."),
    caution_flee_recall_chasers => (3, "monsters", 0, 100, "borg_caution.rs:5295", "#557: with caution_flee_recall 0, this many awake monsters that can move (not RF_NEVER_MOVE) within caution_flee_recall_chase_distance make Word of Recall an escape again; 0 = never for chasers."),
    caution_flee_recall_chase_distance => (6, "grids", 1, 30, "borg_caution.rs:5278", "#557: the distance within which an awake monster counts as a chaser for caution_flee_recall_chasers."),
    caution_flee_recall_gate_below_depth => (0, "dungeon level", 0, 127, "borg_caution.rs:5260", "#557 (owner): a ceiling. With caution_flee_recall 1, a flight at a depth shallower than this (2 = DL1 only) reads Word of Recall only with the breeder or chaser exception of caution_flee_recall 0, and Recall there is no easy escape in the stand tests; the stairs are a short walk. 0 = off (C at every depth); 1 is off too (the town never asks)."),
    caution_flee_recall_danger_percent => (0, "percent of HP", 0, 10000, "borg_caution.rs:5269", "#587 (owner): where the depth gate or caution_flee_recall 0 holds a flight's Word of Recall back, the danger here at or above this percent of the current HP allows it (with the breeder and chaser exceptions); only the flight's read, not the stand tests. 0 = off; at the shipped gate (1, off) and caution_flee_recall 1 nothing is held back, so it acts only with the gate on or caution_flee_recall 0."),
    caution_scary_ranged_no_stairs => (0, "flag", 0, 1, "borg_caution.rs:5222", "#539 (owner): 1 = with no staircase the Borg can walk to, a scary level lets borg_attack target an awake monster in view at range (the best ranged attack), and Word of Recall and Phase Door wait while there is one (none next to the Borg) and the attack planner has something to shoot or throw at it; 0 = off."),
    flow_leash_per_level => (3, "steps per level", 0, 100, "borg_flow.rs:2529", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: borg_get_leash's leash = character level x this + flow_leash_base steps from the stairs (C: 3)."),
    flow_leash_base => (9, "steps", 0, 250, "borg_flow.rs:2530", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: borg_get_leash's leash = character level x flow_leash_per_level + this (C: 9)."),
    flow_leash_unbounded => (250, "steps", 0, 250, "borg_flow.rs:2531", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: the leash when none applies (C: 250, the flow's whole reach)."),
    flow_leash_pickup_max_level => (20, "level", 0, 51, "borg_flow.rs:2532", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: the leash for picking up items applies below this character level (C: 20)."),
    flow_leash_depth_margin => (5, "levels", 0, 127, "borg_flow.rs:2533", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: the exploring leash, and the distance-to-stairs check, apply while the depth is at least the character level less this (C: 5); a larger margin keeps the leash deeper."),
    flow_leash_twitch_after => (21, "twitches", 0, 1000, "borg_flow.rs:2534", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: once the Borg has twitched (run out of things to do) more than this many times, the leash grows by the twitch count (C: 21)."),
    flow_leash_stairs_max_level => (20, "level", 0, 51, "borg_flow.rs:2535", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: borg_flow_far_from_stairs_dist checks the distance to the stairs below this character level (C: 20)."),
    flow_leash_take_near_max_level => (10, "level", 0, 51, "borg_flow.rs:2536", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: take_select skips an item far from the stairs while the Borg is near them, below this character level (C: 10)."),
    flow_leash_take_nearness => (5, "steps", 0, 250, "borg_flow.rs:2537", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: take_select checks an item's steps to the stairs against the leash when the item search reaches beyond this nearness (C: 5)."),
    flow_leash_take_max_level => (20, "level", 0, 51, "borg_flow.rs:2538", "#732 (owner, 2026-11-20: \"expose the leash of C with configuration parameters\"). C: take_select checks an item's steps to the stairs below this character level (C: 20)."),
    explore_breadth_first => (0, "flag", 0, 1, "borg_flow_dark.rs:890", "#1080, #1085 (owner, 2026-11-23: \"it seems to me that the borg explores depth-first, but a safer way would be breadth-first\"): 1 = below explore_bfs_max_hp, the exploration target is the interesting grid nearest the nearest known staircase of either kind (flow steps over known grids), not the one nearest the Borg: the ring at d steps (to d + explore_bfs_slack) is finished before any grid farther out, ties go to the grid nearest the Borg, and a path may not cross a ring farther out than its goals' or the Borg's own, nor pass the return leash. Both exploration calls use it; when the Borg has no route to a known staircase, C's methods run, and a staircase newly seen re-plans. 0 = C's depth-first order, today's play."),
    explore_bfs_max_hp => (25, "hp", 0, 10000, "borg_flow_dark.rs:893", "#1085: explore_breadth_first applies while the Borg's maximum HP is below this (#1061's stair_scum threshold, a hypothesis: arms at 20, 25 and 30). A value per class (`__<class>`). Used with explore_breadth_first."),
    explore_bfs_slack => (0, "steps", 0, 250, "borg_flow_dark.rs:1248", "#1085: with explore_breadth_first, the goals of one try are the candidates from the nearest ring d to d + this, and their paths may cross rings up to d + this (0 = strict rings; 1-2 avoid zig-zagging between equal rings). Used with explore_breadth_first."),
    explore_bfs_hp_per_level => (0, "hp per level", -100, 100, "borg_flow_dark.rs:894", "#1085 (owner, 2026-11-23: the threshold by level and class): explore_breadth_first applies while max HP < explore_bfs_max_hp + this x character level. A value per class (`__<class>`); 0 = explore_bfs_max_hp alone. A level gains randint1(hitdie) HP (player-birth.c:295), half of it on average plus one half, so a delta near a quarter of that keeps the switch on a level or so longer for the frailer races. Used with explore_breadth_first."),
    explore_bfs_food_percent => (100, "percent", 100, 10_000, "borg_flow_dark.rs:908", "#1110 (owner, 2026-11-23: \"plan a little more food and light if we do that\"): while breadth-first exploration would apply to the character (explore_breadth_first on, max HP below its explore_bfs_max_hp + explore_bfs_hp_per_level x CL; asked in town too), the live town planner's food target, the five-ration floor included, is this percent of today's (100 = today; at least 100, so the five rations borg_prepared asks for stay planned: review of #1113). A value per class."),
    explore_bfs_light_percent => (100, "percent", 100, 10_000, "borg_flow_dark.rs:908", "#1110: as explore_bfs_food_percent, for the planner's light goal (the trip's turns or town_light_reserve_turns, whichever is larger): this percent of today's while breadth-first exploration would apply (100 = today; at least 100). The torch replenishment, the oil-throw guard and the fuel-junking guard keep the same goal (borg_light::fuel_goal). A value per class."),
    explore_bfs_leave_when_done => (0, "flag", 0, 1, "borg_flow_dark.rs:1258", "#1098 (owner, 2026-11-23: \"once there is nothing to explore, take the stairs\"): 1 = with explore_breadth_first, nothing reachable left within the leashes marks the think exhausted, and borg_leave_level takes the stairs then (as when the level timer is up), before the secret-door search and the direct monster flow; a random direction it cannot reach tries the other, and when no staircase can be taken in any allowed direction, breadth-first stands down for the level and C explores. 0 = an ordinary miss: the Borg stays until a reason to leave (the level timer, depth, town). The mixed walk always leaves (its design). Used with explore_breadth_first."),
    explore_c_leave_when_done => (0, "flag", 0, 1, "borg_flow_dark.rs:1526", "#1098 (owner, 2026-11-23: \"once there is nothing to explore, take the stairs\"; option 2 of the secret-door searches): 1 = when C's own exploration (breadth-first and mixed not applying) finds nothing it may reach (the far methods 4 and 5: method 5 scans the whole known map within the leash), the think is marked exhausted and the Borg takes the stairs (borg_leave_level, as when the level timer is up) before the door-finding spell and the secret-door search; a random direction it cannot reach tries the other; when no staircase can be taken in any allowed direction (rising with no up staircase known, say), the level is released and the secret-door search runs as before. 0 = C: stay and search until a reason to leave (the level timer, depth, town)."),
    explore_stepable_goals => (0, "flag", 0, 1, "borg_flow_dark.rs:120", "#1189: the DL1 Half-Troll Warrior looped through 444666 for 4,000 turns. #1227 extends the filter to legacy exploration. 1 = both planners target only unknown grids, rubble and closed doors. 0 = C also considers eligible veins and vault walls as exploration goals. GOAL_DARK refuses to dig either terrain. Intentional vein digging uses a separate flow at both settings."),
    explore_mixed => (0, "flag", 0, 1, "borg_flow_dark.rs:1309", "#1098 (second piece of #1080; owner, 2026-11-23: \"breadth-first, then mixed (fall back when they get too far and explore another branch), then when they are on a level for which they are fully prepared they can go depth-first\"): 1 = when breadth-first does not apply and the Borg is not yet prepared for the depth (until #1080's readiness model exists: where C's exploring leash applies), both exploration calls go depth-first (the interesting grid nearest the Borg) but only among grids within a radius R of the nearest known staircase of either kind (flow steps over known grids); a branch whose frontier lies beyond R is parked and the Borg walks back to another branch inside R; R starts at explore_mix_radius and grows by explore_mix_step only when nothing is left inside it, up to the escape model's limit (explore_mix_sight, explore_mix_speed_ratio, explore_mix_contact_damage, explore_mix_max_radius, never more than C's leash); with nothing left within the limit the level is left. A path may not cross a ring beyond R or the Borg's own, nor pass the return leash. When the Borg has no route to a known staircase, C's methods run. 0 = C's depth-first order, today's play."),
    explore_mix_radius => (5, "steps", 1, 250, "borg_flow_dark.rs:1387", "#1098: with explore_mixed, the radius (flow steps from the nearest known staircase) the walk starts at, and the floor of its limit. A hypothesis. Used with explore_mixed."),
    explore_mix_step => (2, "steps", 1, 250, "borg_flow_dark.rs:1388", "#1098: with explore_mixed, how much the radius grows once no interesting grid the Borg can reach is left inside it. Used with explore_mixed."),
    explore_mix_sight => (5, "grids", 0, 250, "borg_flow_dark.rs:1465", "#1098: with explore_mixed, the escape model's D: how far a monster is first seen, at least; the Borg's own light radius is the floor (in a corridor it sees no farther). The limit is floor(D / (explore_mix_speed_ratio - 1)) + floor(max HP / c). A hypothesis. Used with explore_mixed."),
    explore_mix_speed_ratio => (2, "ratio", 2, 10, "borg_flow_dark.rs:1467", "#1098: with explore_mixed, the escape model's r: how many times as fast as the Borg the depth's fast monsters move (2 at DL1-5: Grip, Fang and cave spiders are speed 120, energy 20 a game turn against 10 at normal speed, game-world.c:68, monster.txt:1064,1083,1119). Used with explore_mixed."),
    explore_mix_contact_damage => (5, "hp per turn", 1, 1000, "borg_flow_dark.rs:1469", "#1098: with explore_mixed, the escape model's c at DL1-5: the damage one player turn of contact costs (a speed-120 1d4 biter, Fang or a cave spider, bites twice a turn: 5 on average, monster.txt:1083,1119); multiplied by the depth band (2 at DL6-10, 3 at DL11-15, ...). k = floor(max HP / c) turns of contact are added to the limit. Arms tight and loose in the zoo study. Used with explore_mixed."),
    explore_mix_max_radius => (30, "steps", 1, 250, "borg_flow_dark.rs:1389", "#1098: with explore_mixed, the cap of the radius's limit (C's own leash, where it applies, caps it too). Used with explore_mixed."),
    explore_mix_max_clevel => (50, "level", 1, 50, "borg_flow_dark.rs:1313", "#1098 (owner, 2026-11-23: \"explore_mix_max_clevel by class\"): with explore_mixed, the highest mixed exploration level at which the walk applies; above it C's depth-first order explores. See explore_mix_depth_adjusted_level. A value per class (`__<class>`, the class files): the level before the first CL whose DL1 death rate, and every later one, is at most 0.25 per 100 games in #1080's pooled table (a hypothesis). 50 = no cap: the walk ends where C's leash does (CL20, flow_leash_stairs_max_level). Used with explore_mixed."),
    explore_mix_depth_adjusted_level => (0, "flag", 0, 1, "borg_flow_dark.rs:1320", "#1220: 1 = use max(1, character level - dungeon depth + 1) for explore_mix_max_clevel and the growth of the starting radius, expansion step and maximum radius. 0 = use character level. The leash and the escape model still read actual character level and dungeon depth where required."),
    explore_mix_radius_dp => (0, "per mille per level", 0, 10000, "borg_flow_dark.rs:1387", "#1160 (owner, 2026-11-23: \"all of them should grow in decpercentile by level (and class)\"): explore_mix_radius grows by this per mille per mixed exploration level above 1, linearly and rounded: radius x (1000 + this x (level - 1)) / 1000. See explore_mix_depth_adjusted_level. A value per class (`__<class>`). 0 = no growth. Used with explore_mixed."),
    explore_mix_step_dp => (0, "per mille per level", 0, 10000, "borg_flow_dark.rs:1388", "#1160: explore_mix_step grows by this per mille per mixed exploration level above 1, as explore_mix_radius_dp. A value per class (`__<class>`). 0 = no growth. Used with explore_mixed."),
    explore_mix_max_radius_dp => (0, "per mille per level", 0, 10000, "borg_flow_dark.rs:1389", "#1160: explore_mix_max_radius grows by this per mille per mixed exploration level above 1, as explore_mix_radius_dp. A value per class (`__<class>`). 0 = no growth. Used with explore_mixed."),
    flow_take_phase_short_below => (0, "count", 0, 20, "borg_flow.rs:2522", "#761 (owner: \"make sure it picks it up\"): while the Borg holds fewer Scrolls of Phase Door than this, take_select does not skip a Phase Door on the floor for being far from the stairs (the two leash checks, flow_leash_take_*); the count is the pack's aware scrolls (carried_phase_scrolls): a Phase Door spell or an equipped activation does not count (review of #778); 0 = off, as before (case 425904011 left a Phase Door 12 or more steps from the stairs at CL1). Per class with @<class> or __<class>."),
    flow_take_phase_first => (0, "flag", 0, 1, "borg_flow.rs:2522", "#761: with flow_take_phase_short_below, while short of Phase Doors the object flow goes only to Phase Doors when one is among the objects it may take (an armour piece nearer no longer wins); when that Phase-only flow reaches nothing (past the flow's depth or behind danger), the dropped candidates return and the flow is tried once more (review of #778); 0 = off, as before."),
    flow_leash_any_stair => (0, "flag", 0, 1, "borg_flow.rs:2558", "#1062: 1 = the stair leashes measure the nearest known staircase of either kind, a down staircase included (it leaves the level just as well at CL1-5), not only the up ones: the exploration leash (borg_flow_far_from_stairs_dist, and take_select's and the vein flow's stair checks) takes the walking steps from the nearest of them over known terrain (monsters, traps and danger ignored; an unknown target grid is one step past its nearest reached neighbour; one search a think), so C bug 31 (borg_flow_dark keeps the farthest up staircase) goes too, and an unreachable staircase no longer stands for all; the return leash (borg_think_dungeon's 'Return to Stair', 3 x CL + 14) starts on the walking steps to the nearest of them, not grids (a staircase across a wall is not near), ends ('Close enough', under 3) on them only for a return it started (another goal.less, the depth correction's climb, ends on C's distance to an up staircase), and its walk back flows to the nearest staircase of either kind (borg_flow_stair_both, before the up staircases). In the dungeon only: town has no up staircase, so C never leashes there, and the town's down staircase does not start a leash. Kobold Mage 592754909 and Dwarf Mage 645909552 went up and arrived on a down staircase with no up one known: no leash, killed 26 and 30 grids away. 0 = C (up staircases only). Hypothesis until the zoo measures it."),
    flow_leash_unknown_stair_steps => (0, "steps", 0, 250, "borg_flow.rs:2559", "#1062: with no known staircase the exploration leash counts (with flow_leash_any_stair, none of either kind), or none reachable, the leash is this many walking steps from an anchor: the grid where the Borg arrived on the level (its first update there) while the Borg's area is connected to it, else the oldest landing of a jump with no known route back that the area is connected to (at most 16, the newest always kept), over known terrain (monsters, traps and danger ignored, so a monster at the arrival grid or in the corridor back does not lift it); a grid the arrival grid does not reach is not judged far, not unbounded: C's borg_flow_cost_stair returns 0 for no staircase and for an unreachable one ('go off leash'). In the dungeon only (town is C's). Applies where the exploration leash applies (below flow_leash_stairs_max_level, at depth at least CL - flow_leash_depth_margin, or a short kit). 0 = C. Hypothesis: no measurement chose a value."),
    flow_leash_teleported_recall => (0, "flag", 0, 1, "borg_flow.rs:2560", "#1062: 1 = up to character level flow_leash_recall_max_level, in the dungeon, a teleport between two updates, by its cause: a teleport trap ('You hit a teleport trap!') or the Borg's own Teleportation scroll, staff or activation, Teleport Self, Portal or Shadow Shift always; its own Phase Door (scroll, spell, activation) or Dimension Door never; a jump of unknown cause from flow_leash_recall_min_jump grids that leaves every known staircase, of either kind, more than flow_leash_recall_steps walking steps away over known terrain (or none known or reachable) reads Word of Recall at the first dungeon think at danger 0 after it, once the fights that own the action are done (on DL1-4 a trip to town, not a dive). Judged once, at the first think after the jump; forgotten on a new level, when Recall is already active or none is carried. The Dunadan Druid 1024822556 was thrown 84 squares by a teleport trap, 79 flow steps from the up staircase, walked back and met Fang 38 steps short. 0 = C (no such read)."),
    flow_leash_recall_max_level => (5, "level", 0, 51, "borg_flow.rs:2561", "#1062: flow_leash_teleported_recall applies up to this character level (the issue's CL1-5). Hypothesis."),
    flow_leash_recall_steps => (30, "steps", 0, 250, "borg_flow.rs:2562", "#1062: flow_leash_teleported_recall reads Recall when the jump leaves the nearest known staircase more than this many flow steps away (C's return leash at CL5 is 29 grids: 3 x CL + 14). Hypothesis: no measurement chose 30."),
    flow_leash_recall_min_jump => (12, "grids", 2, 250, "borg_flow.rs:2563", "#1062 (review of #1087, finding 4): flow_leash_teleported_recall counts a jump whose cause the Borg does not know (not its own action, not a teleport trap: a monster's teleport-to, ...) as a teleport from this many grids (engine::distance). Phase Door asks for 9 to 11 grids and lands on the eligible grid nearest that (TELEPORT:10, object.txt, randomised by a quarter in effect_handler_TELEPORT, effect-handler-general.c); the teleport trap and the Teleportation scroll are TELEPORT:M80 and :M60, percentages of the distance to the level's edge, so 12 counts them and not Phase Door, as the issue asks (a trap or a Teleport scroll). 2 = any jump of more than one grid."),
    kill_live_min_power => (0, "hp", 0, 10, "borg_flow_kill.rs:402", "#546 (owner): a living monster (hp above 0) is estimated at least this many hit points; the estimate truncates twice and puts a monster left with 1 HP of 9 at 0, which every attack's overkill cap then scores 0; 0 = off (C's estimate)."),
    kill_live_zero_hp => (0, "flag", 0, 1, "borg_flow_kill.rs:403", "#975: kill_live_min_power's floor also applies to a monster at exactly 0 HP, which the engine leaves alive (mon_take_hit kills only at hp < 0, mon-util.c:1300); a 2-HP grey mold hit for 2 read 0 HP, so every ranged attack on it scored 0 (the overkill cap) and the Borg walked next to it; needs kill_live_min_power above 0; 0 = off, as before (hp 0 is estimated at 0)."),
    kill_grouped_names => (0, "flag", 0, 1, "borg_flow_kill.rs:1301", "#915: 1 = read the engine's grouped monster subjects, \"N <plural>\" (mon-msg.c:333-345: \"3 cave spiders start moving faster.\"), as that race (its monster.txt plural, or plural_aux undone, mon-desc.c:27-36) instead of guessing the player ghost (in 4.2 the last race, Morgoth); \"N monsters\" is an unseen group; a grouped death or blink (\"3 cave spiders die.\") removes up to N known monsters of the race (borg_update.rs, review of #943). A grouped name of no known race is noted and still left unused. 0 retains today's play."),
    caution_desperate_stair_distance => (3, "grids", 1, 100, "borg_caution.rs:1104", "Controls desperate stair distance."),
    trait_timed_sync => (0, "flag", 0, 1, "borg_trait.rs:2814", "#941: 1 = the Borg's speed flag (temp.fast) follows the engine timer (TMD_FAST, TMD_SPRINT, TMD_TERROR) both ways, and the player's own \"You feel ...\" messages skip the monster pain table (\"You feel yourself slow down.\" matched \"slow[s] down.\" and never cleared the flag). 0 = C: the timer only sets the flag, and the end of haste is lost, so the first Potion of Speed leaves the Borg \"fast\" for the rest of the game (seed 2008390085). Compiled 0 (C); shipped 1 in harness/borg.txt (P8)."),
    messages_player_not_pain => (0, "flag", 0, 1, "borg_messages.rs:1300", "#1032: 1 = a message whose subject is the player (\"You ...\") skips the monster pain table: \"You are no longer confused.\", \"You are no longer stunned.\" and \"You can move again.\" (player_timed.txt:227, 283, 219) matched \"[is|are] no longer confused.\" (etc.), and the Borg reacted to pain of a monster named \"You\" (a player-ghost guess, and target_worked). No monster message has \"You\" as its subject. The states themselves are read from the timers either way. 0 = C. Compiled 0 and shipped 0 (owner, 2026-11-23)."),
    trait_speed_timed => (0, "flag", 0, 1, "borg_trait.rs:885", "#1033: 1 = BI_SPEED adds the timed effects as the engine does (player-calcs.c:2176-2187): +10 for FAST or SPRINT, -10 for SLOW, +10 for TERROR on top of either. 0 = C: +10 for FAST or SPRINT, else +5 for TERROR, and SLOW ignored, so a slowed Borg counts itself 10 faster than it is. Residual at either value: borg_danger.rs adds +10 more whenever temp.fast is set, and TERROR sets temp.fast, so TERROR alone at base 110 is modelled 125 at 0 and 130 at 1 against the engine's 120; a fix there must keep borg_fight_defend.rs's deliberate hypothetical haste (it sets temp.fast for one probe). Compiled 0 and shipped 0 (owner, 2026-11-23)."),
    caution_desperate_adjacent_up_distance => (2, "grids", 1, 100, "borg_caution.rs:1114", "Controls desperate adjacent up distance."),
    caution_stairs_clean_path => (0, "choice", 0, 3, "borg_caution.rs:1025", "#736 (owner): 1 = a desperate walk to the stairs (borg_caution's 'Desperate for Stairs') needs a clean path: an awake monster on, or next to, the flow path whose danger is at least caution_stairs_block_danger_percent of the HP makes the stairs not reachable, and the escapes and the attack decide instead. 2 (#751, owner: \"stop flight when other monsters are IN BETWEEN the stairs, not close\") = only a monster on the path (a grid of the walk, the staircase included) blocks it, not one next to it. 3 (#751, owner on 93034700: \"he has to actually walk past his opponent\") = on the path, or next to it (one of the 8 grids around a grid of the path) and nearer the staircase at the path's end than the Borg, in grid steps (king moves): a monster the walk has to pass. 0 = C."),
    caution_stairs_block_danger_percent => (50, "percent", 0, 10_000, "borg_caution.rs:1026", "#736: with caution_stairs_clean_path, the danger (in percent of the current HP) at which a monster on or next to the path to the stairs blocks it."),
    caution_stairs_clean_path_flee => (0, "flag", 0, 1, "borg_caution.rs:1001", "#743 (owner, the follow-up of #736): 1 = every flight to the stairs (GOAL_FLEE: a scary level, too much danger, 'Looking for any stair') takes a staircase only by a clean path: the flight's own flow is walked before it is committed (#736's path_blocker), and an awake monster on, or next to, the path whose danger is at least caution_stairs_block_danger_percent of the HP refuses it; a flight to any stairs then tries each kind on its own; with none clean the flight does not start and the escapes and the attack decide (#736's 'Stairs not reachable', noted once per level). The desperate walk keeps its own check (caution_stairs_clean_path). 0 = C."),
    caution_desperate_ranged_first_reach => (0, "grids", 0, 10, "borg_caution.rs:1039", "#751 (owner on 247597266: the Mage walked toward a cutpurse and an apprentice between it and the stairs, with 22 Flasks of oil and Magic Missile): above 0, while the Borg has a ranged attack (a bow with ammunition or something to throw; spells are not counted) and no awake monster is adjacent, the desperate walk to the stairs does not start toward an awake monster within this many grids of its path and nearer the stairs than the Borg, whose danger is at least caution_stairs_block_danger_percent of the HP; the attack at range decides. 0 = C."),
    caution_desperate_fast_danger_percent => (0, "percent of HP", 0, 10_000, "borg_caution.rs:7124", "#747: 'Desperate for Stairs (three)' does not walk when an adjacent monster is faster than the Borg; above 0, only when that monster's danger is also at least this percent of the current HP (a harmless fruit bat no longer cancels the walk). 0 = C (any faster monster)."),
    caution_desperate_fast_haste => (0, "flag", 0, 1, "borg_caution.rs:1127", "#907: 1 compares current engine player speed with the adjacent monster's base speed plus last observed Haste minus last observed Slow in Desperate for Stairs (three), as in mon-move.c:1929-1934 (574b618c). The danger gate still prices the monster from its base speed and can allow the walk. 0 uses the legacy speed comparison."),
    caution_desperate_walk_cost_percent => (0, "percent of HP", 0, 10_000, "borg_caution.rs:7147", "#908: above 0, refuse branch (three) when the selected route's steps times expected excess melee damage from awake adjacent mobile pursuers reaches this percent of current HP. Each pursuer spends one action following. Current monster speed includes observed Haste and Slow. 0 disables the budget."),
    caution_stairs_flight_ignore_danger => (0, "flag", 0, 1, "borg_caution.rs:7077", "#750: 1 = a flight's stair flow (GOAL_FLEE, or any GOAL_BORE stair flow while fleeing or leaving: the bored walk up, the bored walks down and the powerdive) refuses no grid for its danger, as the desperate walk; a monster near a clear corridor no longer closes it (807683971: a black naga two grids beside the corridor). Whether a monster stands on the path is caution_stairs_clean_path's test. 0 = C (grids above the flow's fear are refused)."),
    caution_retreat_reentry_ticks => (0, "Borg thinks", 0, 128, "borg_caution.rs:7674", "#932: during a dungeon departure, exclude recent tactical retreat grids from stair routes for up to this many thinks while it remains dangerous. Zero leaves flight unchanged. Try 128 as a pilot hypothesis."),
    caution_stairs_flight_threat_weight => (0, "extra steps at current HP", 0, 100, "borg_caution.rs:7668", "#765: stair flights (GOAL_FLEE, including desperate walks, or a bored ascent while fleeing or leaving) price each entered grid at one step plus this weight times danger divided by current HP. Awake monster grids are blocked. A clean-path veto can trigger a search for a clean detour. 0 = existing stair flows."),
    caution_immobile_step_to_exit => (0, "flag", 0, 1, "borg_caution.rs:2438", "#736 (owner): 1 = the step away from an immobile attacker (caution_ranged_vs_immobile 1-3) takes, among the grids it may step to, the one with the fewest steps to a staircase, then the least danger. 0 = the least danger only."),
    caution_confused_clw_cure => (0, "flag", 0, 1, "borg_caution.rs:2192", "#837 (owner): 1 = a Potion of Cure Light Wounds is a confusion cure for the Borg: it takes 20 turns off confusion (object.txt TIMED_DEC:CONFUSED, dice 20), enough for a grey mushroom patch's 3 + 1d(level) turns. Added after Clear Mind, Cure Serious and Critical Wounds to the confusion chains of borg_heal ('Fixing Confusion', levels 2 and 5) and of the cures at the end of borg_caution, before a Healing potion. 0 = C (CLW cures blindness only)."),
    caution_stairs_flight_unknown => (0, "mode", 0, 2, "borg_caution.rs:7694", "#881 (Priest 399497898: in a one-exit lattice room 58 of 68 flight paths crossed unseen rock north of it, so the Borg walked into the dead end 25 times): how a weighted stair flight (caution_stairs_flight_threat_weight above 0) treats a grid its map holds as unknown (FEAT_NONE). 0 = one step, like floor (today's play); 1 = one step plus caution_stairs_flight_unknown_steps; 2 = refused: the flight walks known grids only, and a staircase reachable only through unknown grids is not reachable for the flight, nor for the flee tests that ask whether a stair flow reaches the Borg; with no staircase reachable through known grids that turns on Recall in flight (caution_flee_stairs_before_recall) and ranged attacks on a scary level (caution_scary_ranged_no_stairs), review of #897. Acts on the down flight; the up flight below CL 36 in the light and the desperate walk already refuse unknown grids. Hypothesis: no measurement chose a mode."),
    caution_stairs_flight_unknown_steps => (5, "steps", 0, 100, "borg_caution.rs:7694", "#881: with caution_stairs_flight_unknown 1, the extra steps an unknown grid costs a weighted stair flight. The issue's range is 3-10. Hypothesis: no measurement chose 5."),
    caution_confused_cure_before_walk => (0, "mode", 0, 2, "borg_caution.rs:1087", "#837 (owner): 1 = a confused Borg with a confusion cure in the pack does not start a 'Desperate for Stairs' walk (75% of its steps go astray, player_confuse_dir); the cures at the end of borg_caution come first. Not while the cure is held next to an immobile attacker (#646). 2 = no 'Desperate for Stairs' walk at all while confused, cure or not. 0 = C."),
    caution_confusion_cure_when_stuck => (0, "flag", 0, 1, "borg_caution.rs:2329", "#837 (owner): 1 = next to an immobile confuser, the confusion cure is held only while a step out of its reach can still be taken (a qualifying grid, tries left); with none, cure anyway (#646's hold left the Borg neither stepping nor curing, and it went on to melee confused). Needs caution_ranged_vs_immobile >= 1. 0 = #646's hold whether or not a step is possible."),
    caution_immobile_step_mobile_first => (0, "flag", 0, 1, "borg_caution.rs:2440", "#837 (owner): 1 = the step away from an immobile attacker (caution_ranged_vs_immobile 1-3) ranks the grids out of its reach by the danger from the monsters that move within two grids of the grid first (lowest wins), then the fewest steps to a staircase (caution_immobile_step_to_exit), then the least danger; the stairs decide only between equally safe grids. The first key is the full damage of the movers (borg_danger_one_kill, full), the last the average danger of the grid (borg_danger), so they are on different scales. Acts only with caution_ranged_vs_immobile >= 1 (shipped 0). 0 = the stairs first."),
    caution_immobile_no_return => (0, "mode", 0, 2, "borg_caution.rs:2054", "#837 (borg-zoo#576: the step away from a grey mold was undone by the next flow, 'Flowing toward dark-3', in two deaths): after a step away from an immobile attacker (caution_ranged_vs_immobile 1-3), the flows (borg_flow_spread) do not enter its grid or the grids next to it while it is still known there. 1 = only while the Borg is below its full HP; 2 = for the rest of the level. A flight to the stairs is not stopped. 0 = the flows may lead back."),
    caution_poisoned_no_cure_retreat_hp_percent => (0, "percent of max HP", 0, 100, "borg_caution.rs:2514", "#934 (seed 829474344: 12 CLW drunk against poison, two Phase Doors never read, dead at HP 4 still retreating one grid): poisoned, with no usable poison cure (Neutralize Poison or curing food, Cure Critical Wounds, Healing, *Healing*, Life, or a castable Cure Poison or Herbal Curing) and below this percent of the maximum HP, with an escape at hand (a staircase underfoot, or a Phase Door it can read or cast), the escapes are tried before the strategic retreat; when they decline, the retreat goes on as usual (review of #968). Uses only the Poisoned flag: the screen shows no amount (player_timed.txt:255). Hypothesis: 75. 0 retains today's play."),
    caution_retreat_immobile_kill_turns => (0, "turns", 0, 100, "borg_caution.rs:2587", "#933 (seed 829474344: a 25-HP white jelly next to the route never attacked; melee kills it in two turns): the strategic retreat is not taken when the only known monster next to the Borg is an immobile attacker (it cannot follow) that the Borg can melee (not afraid nor cursed with fear; the record no older than caution_one_blow_monster_age and attack_max_target_age) and whose estimated HP its expected melee damage a turn (the real hit chance and blows, without targeting bonuses) takes in at most this many turns, while the danger here from every monster over those turns stays below caution_retreat_immobile_danger_percent of the HP. The attack decides. Hypothesis: 3. 0 retains today's play."),
    caution_retreat_immobile_danger_percent => (100, "percent of HP", 1, 1000, "borg_caution.rs:2595", "#933 (review of #967): with caution_retreat_immobile_kill_turns, the danger here from every monster (borg_danger, one turn) times the turns of the kill must stay below this percent of the current HP. 100: the whole fight costs less than the HP left. Hypothesis."),
    caution_retreat_oscillation_limit => (0, "retreats", 0, 1000, "borg_caution.rs:2663", "#932 (seed 829474344: 74 retreats from a white jelly, each undone by the next stair-flight step): after this many strategic retreats in a row from the same place (the grid the last one left or one next to it, each within caution_retreat_oscillation_thinks thinks of the last, on one level), the next is not taken; the escapes, the outmatched cure and the attack decide. Hypothesis: 3. 0 retains today's play."),
    caution_retreat_oscillation_thinks => (3, "thinks", 1, 100, "borg_caution.rs:2671", "#932: with caution_retreat_oscillation_limit, a retreat continues the run when it comes at most this many thinks after the last one (the loop of 829474344 retreats every second think). Hypothesis."),
    caution_retreat_oscillation_backaway => (0, "flag", 0, 1, "borg_caution.rs:4580", "#1023 (seed 1512540769: a Human Priest backs up from white worm masses, \"Backing up to 88,45 (32 > 1)\", and the stair flight walks back, mixed with retreats from one grid until death): with caution_retreat_oscillation_limit, 1 = the back-away (borg_caution's step to the adjacent grid of least danger) counts in the same run as the strategic retreat and is vetoed by it too. 0 = only strategic retreats count (the arms of borg-zoo#654)."),
    caution_stairs_immobile_reach => (0, "flag", 0, 1, "borg_caution.rs:2802", "#931 (seed 829474344: the stair flight passed a white jelly 77 times): 1 = the weighted stair flight (caution_stairs_flight_threat_weight > 0) first runs its whole route search, the clean-path check included, refusing every square next to a known immobile attacker (a NEVER_MOVE monster with blows: it hits only its eight neighbours, mon-move.c:1586, 1621-1644); when that finds no usable route it runs the ordinary search. Sleeping attackers count (they wake as the Borg comes near) and a record counts until the usual bookkeeping drops it. The other immobile switches (caution_immobile_no_return, caution_ranged_vs_immobile 2) leave stair flights alone. 0 retains today's play."),
    caution_stairs_immobile_reach_extra_steps => (20, "steps", 0, 250, "borg_caution.rs:2808", "#931 (second review of #965): with caution_stairs_immobile_reach, the reach-free route is taken only when it costs at most this many steps more than the ordinary route (StairPath cost, weighted danger included); a costlier detour leaves the ordinary route. 0 = any cost. 20 is a hypothesis (RULES 14)."),
    caution_note_standing_on_stairs => (0, "flag", 0, 1, "borg_caution.rs:930", "#1176 (from the #1173 investigation; C's defect, borg-caution.c:805/983/996): note_standing's on_upstair/on_dnstair are declared false and only ever assigned false, even standing on the tracked staircase, so the danger+standing-on-stair trigger of choose_stairs (borg-caution.c:1121) and the `|| on_*stair` disjuncts of the standing stair takes (1176, 1194) are dead code; the Rust mirrors C. 1 = the flags mean their evident intent (true when the Borg stands on the tracked staircase), which makes the danger arm of the choose_stairs trigger live: a low-level character in real danger on an up staircase takes the stairs up. 0 retains today's play. Shipped 0 per the P9 rule (PLAN.md)."),
    caution_standing_descent_gated => (0, "flag", 0, 1, "borg_caution.rs:4703", "#1178 (from the #1173 investigation; C's own bookkeeping: stair_more is set before the descent flows it belongs to — the leaving block, the power dive, borg_leave_level's dive — and survives their failure until a new level clears it, borg-update.c:2172): 1 = caution's standing \"Take stairs down\" applies more_refused's descent gates (not prepared for the next depth unless brave, hungry or weak or low on food, a held sale, no light), so a stale flag can complete a descent only when the gates agree; a flag the same think's own dive set passes them again, and the hungry standing descent C allows is refused. 0 retains today's play. Shipped 0 per the P9 rule (PLAN.md)."),
    caution_paralyser_step_away => (0, "flag", 0, 1, "borg_caution.rs:2963", "#1125 (owner: \"from paralyzing monsters it is necessary to walk away\"; a floating eye killed Necromancer 453254028 at DL1, borg-cemetery#141): 1 = without Free Action (BI_FRACT 0), next to a known monster that never moves and has a PARALYZE blow (floating eye, Old Man Willow and one deeper of the 27 paralysers in monster.txt), awake or asleep, the first action of the dungeon think is a step to a grid out of its reach (out of every immobile attacker's reach, not more dangerous than here, whatever the anti-bounce count; toward the nearest staircase with caution_immobile_step_to_exit): before an early fight, a rest, a cure or a Phase Door. One step out of a never-moving monster's reach ends its threat (mon-move.c:1586, 1621-1644). No step count: a flow can lead back unless caution_paralyser_no_reach is on. 0 retains today's play."),
    caution_paralyser_no_reach => (0, "flag", 0, 1, "borg_caution.rs:2913", "#1125: 1 = without Free Action, every flow (exploration, breadth-first and mixed included, kills, takes, stair flights) treats a known paralyser's grid and the eight around it as closed (spread_blocked, and no flow destination is seeded there: borg_flow_enqueue_grid), moving or not, awake or asleep; the Borg's own grid stays open so a flow can lead out. A shot from two grids or more still reaches it. 0 retains today's play."),
    caution_walk_away_immobile => (0, "flag", 0, 1, "borg_caution.rs:2986", "#1147 (owner: \"a list of 'walk_away immobile monster' ... with the eye, the patch and the jelly\"): 1 = a walk-away monster (it never moves, and it is a `ranged` entry of the scary list for this character, the floating eye, the mushroom patches and the jellies of harness/scary_monsters.txt, or it paralyses and the Borg has no Free Action) is never meleed, never walked to, and no flow enters its grid or the eight around it, stair flights included; next to one, awake or asleep, the first action of the dungeon think is to leave its reach: up (or down, to a depth already reached) a staircase the Borg stands on, else a step to the square ranked best by the owner's order (a clean way to a staircase, then the least danger from monsters that move, then the nearest room exit, then the most free space, then the least danger); with no such square, the think is cornered and a blow at the monster is the last resort. Kill-on-sight from two grids or more is unchanged. 0 retains today's play."),
    caution_walk_away_phase_fail => (30, "percent", 0, 100, "borg_caution.rs:3234", "#1147 (review of #1148, finding 2): with caution_walk_away_immobile, the most failure rate at which a Borg cornered next to a walk-away monster, not confused, casts Phase Door (after an activation and a scroll) before a blow is allowed. 30 as caution_recall_phase_fail; a hypothesis."),
    caution_remember_never_move => (0, "flag", 0, 1, "borg_caution.rs:2924", "#1125 (seed 453254028: the floating eye was forgotten by the anti-bounce reset at count 500 and the breadth-first flow then stepped next to it): 1 = a NEVER_MOVE monster's record survives the anti-bounce reset (borg_check_bounciness) and the expiry of records not seen for 2000 turns (expire_kills); it goes when it is seen dead or gone, or with the level. 0 retains today's play."),
    flow_brave_walk_commit => (0, "flag", 0, 1, "borg_flow.rs:1955", "#1130 (owner, 2026-11-23: \"it must definitely be fixed\"; Blackguard 312081895: 60,000 turns between two grids): 1 = a stair walk started under boosted bravery (the stuck think's 'Goal_less, brave') is committed: the next thinks continue it (borg_flow_old) with the avoidance it was spread with, while its last step is at most 3 thinks old, it has steps left (the route's cost plus 10), it is on its own level, and the danger here is at most half the HP; at the normal avoidance its next step was refused and exploration took the step back, so the stuck think boosted again. 0 = C: the boost lasts one think."),
    stairs_descent_gates_without_upstair => (0, "flag", 0, 1, "borg_flow_stairs.rs:1001", "#1177 (from the #1173 investigation; C's defect, borg-flow-stairs.c:193, tracked upstream as better-borg-c #540 and neo-angband-tracker #629): 1 = the four descent refusals of more_refused (not prepared for the next depth unless brave, hungry or weak or low on food, a held sale, no light) apply whether or not an up staircase has been seen; C wraps them in 'if there are no down stairs, don't filter use of up stairs' and skips them all when the Borg has seen no up staircase, so a character that has seen none dives past gates that would have stopped it. Shipped 0 per the P9 rule (PLAN.md): each value is turned on after its own zoo comparison."),
    attack_mobile_threat_first => (0, "flag", 0, 1, "borg_fight_attack.rs:5433", "#736 (owner): 1 = while a mobile monster among borg_attack's targets is a threat (danger at least attack_mobile_threat_percent of the HP), immobile monsters are not targets. 0 = C."),
    attack_mobile_threat_percent => (50, "percent", 0, 10_000, "borg_fight_attack.rs:5434", "#736: with attack_mobile_threat_first, the danger (in percent of the current HP) at which a mobile target is a threat."),
    loop_breaker => (0, "flag", 0, 1, "borg_loop.rs:119", "#1153 (owner, 2026-11-23: \"why would you ship it off by default\"): 1 = a movement loop is detected and broken: the last 16 moves among at most 3 grids, the map learning nothing, no danger here above a quarter of the HP; then, the first that applies: a walk to a staircase at the normal avoidance, a committed brave walk to one (#1130), else the loop's grids other than staircases are closed to every step for 20 thinks. The note names the last decisions chosen. The safety net under the per-pair guards (#1108, #1133, #1152, #932). 0 = C: no detector."),
    loop_moves => (16, "moves", 1, 250, "borg_loop.rs:136", "#1190: with loop_breaker, the number of quiet moves the loop window is judged on (16, #1153's guess, now named so the zoo can measure a wider window). Supported region: loop_grids < loop_moves; at or above it the grid test is vacuous and every quiet window is a loop (borg_loop.rs's is_loop_with). Used with loop_breaker."),
    loop_grids => (3, "grids", 1, 250, "borg_loop.rs:75", "#1190: with loop_breaker, the most distinct grids the window's moves may visit (3, #1153's guess). #1189's DL1 cycle walked 444666 among four grids and the net could not see it; the zoo measures the arm at 4 against 3 (borg-zoo issue of #1190) before this moves. Supported region: loop_grids < loop_moves; at or above it the grid test is vacuous and every quiet window is a loop (borg_loop.rs's is_loop_with). Used with loop_breaker."),
    caution_desperate_scary_distance => (20, "grids", 1, 100, "borg_caution.rs:1088", "Controls desperate scary distance."),
    caution_desperate_scary_level => (20, "level", 1, 51, "borg_caution.rs:1092", "Controls desperate scary level."),
    caution_desperate_unique_depth => (5, "depth", 0, 127, "borg_caution.rs:1086", "Controls desperate unique depth."),
    caution_desperate_adjacent_down_distance => (2, "grids", 1, 100, "borg_caution.rs:1123", "Controls desperate adjacent down distance."),
    caution_retreat_max_distance => (10, "grids", 1, 100, "borg_caution.rs:1229", "Controls retreat max distance."),
    caution_retreat_dead_end => (0, "mode", 0, 2, "borg_caution.rs:1509", "#881 part 2 (Priest 399497898: in a one-exit lattice room of white worm masses, 47 retreats and 27 back-ups moved the Borg about inside the room, 25 times into its dead-end tip, never out): with a known breeder within danger_monster_max_distance of the Borg, retreat and back-up refuse a grid by the known way out: the known floor grids at least caution_retreat_dead_end_distance from every known breeder, and the known staircases (paths over known floor; unknown grids are walls). 1 = refuse a dead end, a grid whose every known path out passes through the Borg's own grid: only where the Borg's grid is the only link (a corridor's end, a pocket behind a single square; in a lattice room every square has two links, so mode 1 refuses nothing there, review of #953); 2 = also refuse a grid farther from the way out than the Borg's (in the priest's lattice room, the mode that acts). Both refuse a grid the Borg sees but has not mapped (unknown, so not known floor). Nothing is refused when the Borg has no known path out or stands on it. 0 = today's play. Hypothesis: no measurement chose a mode."),
    caution_retreat_dead_end_distance => (6, "grids", 1, 100, "borg_caution.rs:1510", "#881 part 2: the way out for caution_retreat_dead_end is the known floor at least this many grids (the engine's distance) from every known breeder, as caution_breeder_exit_distance for the breeder exit. Hypothesis: no measurement chose 6."),
    caution_retreat_step_hp_tenths => (4, "tenths", 0, 100, "borg_caution.rs:1276", "Controls retreat step hp tenths."),
    caution_retreat_step_danger_tenths => (5, "tenths", 0, 100, "borg_caution.rs:1285", "Controls retreat step danger tenths."),
    caution_retreat_surrounded_level => (20, "level", 1, 51, "borg_caution.rs:1296", "Controls retreat surrounded level."),
    caution_retreat_surrounded_distance => (5, "grids", 1, 100, "borg_caution.rs:1297", "Controls retreat surrounded distance."),
    caution_retreat_surrounded_hp_tenths => (15, "tenths", 0, 100, "borg_caution.rs:1309", "Controls retreat surrounded hp tenths."),
    caution_retreat_low_level_hp_multiplier => (4, "multiplier", 0, 20, "borg_caution.rs:1342", "Controls retreat low level hp multiplier."),
    caution_retreat_monster_age => (2, "turns", 0, 100000, "borg_caution.rs:1377", "Controls retreat monster age."),
    caution_back_surrounded_danger_tenths => (12, "tenths", 0, 100, "borg_caution.rs:1686", "Controls back surrounded danger tenths."),
    caution_back_monster_age => (2, "turns", 0, 100000, "borg_caution.rs:1741", "Controls back monster age."),
    caution_back_fast_monster_distance => (2, "grids", 1, 100, "borg_caution.rs:1767", "Controls back fast monster distance."),
    caution_back_high_level => (35, "level", 1, 51, "borg_caution.rs:1695", "Controls back high level."),
    caution_back_high_level_danger_tenths => (6, "tenths", 0, 100, "borg_caution.rs:1696", "Controls back high level danger tenths."),
    caution_back_low_level => (35, "level", 1, 51, "borg_caution.rs:1705", "Controls back low level."),
    caution_back_low_level_danger_tenths => (8, "tenths", 0, 100, "borg_caution.rs:1707", "Controls back low level danger tenths."),
    caution_back_injured_level => (10, "level", 1, 51, "borg_caution.rs:1729", "Controls back injured level."),
    caution_back_freedom_depth => (85, "depth", 0, 127, "borg_caution.rs:1738", "Controls back freedom depth."),
    caution_move_antisummon_delay => (50, "turns", 0, 100000, "borg_caution.rs:2496", "Controls move antisummon delay."),
    caution_move_max_hp => (500, "hp", 0, 10000, "borg_caution.rs:2482", "Controls move max hp."),
    caution_one_blow_monster_age => (2, "turns", 0, 100000, "borg_caution.rs:2591", "Controls one blow monster age."),
    caution_nasty_light_timeout => (250, "turns", 0, 100000, "borg_caution.rs:4363", "Controls nasty light timeout."),
    caution_questor_arrival_turns => (10, "turns", 0, 100000, "borg_caution.rs:4377", "Controls questor arrival turns."),
    caution_escape_limit => (3, "count", 0, 1000, "borg_caution.rs:4411", "Controls escape limit."),
    caution_escape_absolute_limit => (55, "count", 0, 1000, "borg_caution.rs:4412", "Controls escape absolute limit."),
    caution_escape_limit_max_depth => (98, "depth", 0, 127, "borg_caution.rs:4414", "Controls escape limit max depth."),
    caution_escape_flee_limit => (3, "count", 0, 1000, "borg_caution.rs:4420", "Controls escape flee limit."),
    caution_swap_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:4568", "Controls swap danger divisor."),
    caution_recall_prep_turns => (2, "turns", 0, 100000, "borg_caution.rs:4641", "Controls recall prep turns."),
    caution_restock_town_delay => (200, "turns", 0, 100000, "borg_caution.rs:4354", "Controls restock town delay."),
    caution_restock_ccw_count => (2, "count", 0, 1000, "borg_caution.rs:4368", "Controls restock ccw count."),
    caution_restock_food_count => (3, "count", 0, 1000, "borg_caution.rs:4369", "Controls restock food count."),
    caution_restock_fuel_count => (2, "count", 0, 1000, "borg_caution.rs:4370", "Controls restock fuel count."),
    caution_restock_level_delay => (400, "turns", 0, 100000, "borg_caution.rs:4371", "Controls restock level delay."),
    caution_flee_danger_hp_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:4211", "Controls flee danger hp multiplier."),
    caution_flee_max_level => (50, "level", 1, 51, "borg_caution.rs:4387", "Controls flee max level."),
    caution_town_flee_max_level => (50, "level", 1, 51, "borg_caution.rs:4401", "Controls town flee max level."),
    town_recall_reserve => (0, "count", 0, 5, "borg_town_capability.rs:1114", "#557: the live town planner (the capability model, and borg_town_utility.rs's need model) values this many Word of Recall as a requirement at the full trip weight each, at any depth (the walk a scroll saves is always worth less than that), so a shallow Borg buys and carries one; 0 = off (Recall valued only as the walk it saves, which at DL1-3 never beats saving gold)."),
    buy_recall_from_gold => (0, "gold", 0, 10_000_000, "borg_store_buy.rs:366", "#810: minimum gold that must remain after a Word of Recall purchase. Below this amount the reserve target gives way to ordinary walk-saving value; 0 disables the purchase floor."),
    town_recall_value_percent => (100, "percent", 0, 10000, "borg_town_capability.rs:2912", "#557 (owner): the town planner values a Word of Recall at this percent of the walk it saves (both the capability model and borg_town_utility.rs's need model); 100 = as computed; higher makes a shallow Borg buy and keep one."),
    caution_retreat_from_faster => (0, "flag", 0, 1, "borg_caution.rs:1216", "#916: 1 = the strategic retreat (borg_caution's step to a happier grid in view) is not taken while an awake monster record next to the Borg (no freshness or mobility check) has a haste forecast at least caution_retreat_faster_margin above player->state.speed; the forecast (forecast_monster_speed) is the observed mspeed plus 10 for an observed FAST and omits the engine's SLOW adjustment (mon-move.c:1928-1938), so it is not the engine's net speed (base 116 with SLOW level 2 against 110 vetoes at +6 though the engine's net difference is +2). Also, the selected destination's danger at or above HP vetoes the whole retreat (checked after the selection: no other candidate is tried and the first step's danger is not checked). The escapes (Phase Door) and the outmatched cure decide instead. C's own test for a grid next to a faster monster has no effect (kept as in C). 0 retains today's play."),
    caution_retreat_faster_margin => (5, "speed points", 0, 40, "borg_caution.rs:1218", "#916: with caution_retreat_from_faster, a monster counts as faster at this many speed points above the Borg (at least 1). Monsters spread by about two points around their race's speed (mon-make.c:1158-1164). The difference is the haste forecast (SLOW not subtracted) minus player->state.speed. Hypothesis: as fast_fragile_speed_margin, no measurement chose 5."),
    town_recall_urgent => (0, "flag", 0, 1, "borg_town_utility.rs:256", "#859 (owner: \"Word of recall should have a high value\"): 1 = when the next trip reaches prepared_recall_depth and no Word of Recall is carried, the first scroll is an urgent need: the live town planner buys it first, before weighing anything else (as missing light or food); the buy_recall_from_gold floor does not hold that first scroll back, but still caps the spares and the town_recall_reserve requirement in the same visit. 0 = off."),
    town_recall_urgent_depth => (0, "depth", 0, 127, "borg_town_utility.rs:255", "#984: the trip depth from which town_recall_urgent makes the first Word of Recall urgent when none is carried. 0 = prepared_recall_depth (#859, today); N > 0 = from DL N (1: any trip into the dungeon), so a shallow Borg that has read its last scroll buys one first, before the solver weighs anything, and the buy_recall_from_gold floor does not hold it back. The dive gate (prepared_recall_depth) is not moved. Read only with town_recall_urgent on."),
    town_recall_curve_scale_percent => (0, "percent", 0, 10_000, "borg_town_capability.rs:1040", "#859 (owner: \"a curve for word of recall starting from 1 and make it be 110 by default, so that they buy a few\"; \"use the trip weight\"): above 0, the live town planner values every Word of Recall on a diminishing curve on the trip weight (borg_town_w_trip), weight and demand scaled by this percent as town_heal_curve_scale_percent scales healing's: each scroll is worth less than the last and never nothing, so a Borg with gold and room carries spares. The town_recall_reserve requirement stays on top (its own row, as town_phase_reserve's); town_recall_value_percent (the walk a scroll saves) is then not read. 0 = #557 (the reserve alone, nothing past it; below the gold floor, the walk a scroll saves)."),
    prepared_recall_depth => (5, "depth", 0, 127, "borg_prepared.rs:236", "#557 (owner): a descent to this depth or deeper needs one Word of Recall carried (C's \"1 recall\", from DL5); lower moves it shallower; 0 = never. It only gates descents: it buys nothing."),
    // #768: C Borg dive gates. Class arrays follow CLASSES (class.txt order).
    prepared_clevel_depth_margin => (0, "levels", -50, 50, "borg_prepared.rs:747", "Before CL50, require maximum character level to reach depth minus this margin. Positive lets the Borg dive earlier."),
    prepared_dl2_min_clevel => (0, "level", 0, 50, "borg_prepared.rs:265", "#1217: Minimum maximum character level for a DL2 proposal, skipped in risky play. 0 disables this gate. Shipped class values are one level above explore_mix_max_clevel."),
    prepared_dl2_min_hp => (30, "hp", 0, 10000, "borg_prepared.rs:257", "Minimum maximum HP from DL2, skipped in risky play."),
    prepared_dl34_min_hp => (50, "hp", 0, 10000, "borg_prepared.rs:281", "Minimum maximum HP from DL3, skipped in risky play.", [50, 60, 40, 40, 60, 50, 50, 50, 50]),
    prepared_dl34_min_clevel => (4, "level", 0, 50, "borg_prepared.rs:285", "Minimum maximum character level from DL3, skipped in risky play.", [4, 11, 9, 9, 11, 4, 8, 4, 4]),
    prepared_cure_min => (2, "potions", 0, 99, "borg_prepared.rs:293", "Minimum sum of CLW, CSW and CCW from DL3 while below prepared_cure_until_clevel."),
    prepared_cure_until_clevel => (30, "level", 0, 50, "borg_prepared.rs:294", "Maximum character level at which the early combined-cure requirement stops applying."),
    prepared_dl59_min_hp => (60, "hp", 0, 10000, "borg_prepared.rs:309", "Minimum maximum HP from DL5 while already in the dungeon, skipped in risky play.", [60, 80, 60, 60, 80, 60, 60, 60, 60]),
    prepared_dl59_min_clevel => (6, "level", 0, 50, "borg_prepared.rs:313", "Minimum maximum character level from DL5 while already in the dungeon, skipped in risky play.", [6, 15, 15, 15, 15, 6, 10, 6, 6]),
    prepared_mid_clevel_offset => (0, "levels", -50, 50, "borg_prepared.rs:382", "From DL10, require maximum character level to reach depth plus this offset when the mid-depth class gate applies.", [-4, 5, 0, 0, 5, 0, 0, 0, -4]),
    prepared_mid_max_depth => (19, "depth", 0, 127, "borg_prepared.rs:380", "Last depth at which the mid-depth offset applies for classes other than Mage and Necromancer."),
    prepared_mid_caster_max_clevel => (28, "level", 0, 50, "borg_prepared.rs:378", "Last maximum character level at which the mid-depth offset applies for Mage and Necromancer."),
    prepared_deep_clevel_offset => (5, "levels", -50, 50, "borg_prepared.rs:450", "From DL21, require maximum character level to reach depth plus this offset within the deep character-level range.", [5, 8, 13, 13, 8, 7, 10, 8, 5]),
    prepared_deep_min_clevel => (0, "level", 0, 50, "borg_prepared.rs:451", "First maximum character level at which the deep offset applies (C: 29 for Ranger).", [0, 0, 0, 0, 0, 0, 0, 29, 0]),
    prepared_deep_max_clevel => (38, "level", 0, 50, "borg_prepared.rs:449", "Last maximum character level at which the deep offset applies. The Mage and Necromancer depth ramp starts above this level.", [38, 38, 46, 46, 38, 40, 43, 41, 38]),
    prepared_deep_caster_ramp_max_clevel => (44, "level", 0, 50, "borg_prepared.rs:457", "Last maximum character level at which the Mage and Necromancer depth ramp applies."),
    prepared_deep_caster_ramp_base_depth => (30, "depth", 0, 127, "borg_prepared.rs:458", "Base depth of the Mage and Necromancer ramp from DL21."),
    prepared_deep_caster_ramp_depth_per_level => (2, "levels", 0, 127, "borg_prepared.rs:459", "Extra depth per character level above prepared_deep_max_clevel on the Mage and Necromancer ramp."),
    prepared_dl34_plus_min_clevel => (40, "level", 0, 50, "borg_prepared.rs:503", "Minimum maximum character level from DL34, skipped in risky play."),
    town_gold_reserve => (0, "gold", 0, 10_000_000, "borg_town_budget.rs:268", "#493: gold of the purse at arrival the live town planner keeps outside the urgent needs (the larger of this and town_gold_reserve_percent); the visit's sale proceeds are spendable on top of it."),
    town_gold_reserve_percent => (0, "percent", 0, 100, "borg_town_budget.rs:264", "#493: percent of the purse at arrival the live town planner keeps outside the urgent needs (the larger of this and town_gold_reserve); the visit's sale proceeds are spendable on top of it."),
    town_fear_cure_count => (0, "count", 0, 100, "borg_town_capability.rs:1457", "#516: while the character is not afraid, fear cures (Potion of Boldness, CURE:AFRAID) are valued linearly up to this many in the pack and not beyond (each of the first ones is worth more than on the curve, a further one nothing); 0 = the diminishing curve (the planner kept buying Boldness: 35 bought, 7 drunk on 18 games)."),
    town_restore_life_spell_fail => (20, "percent", 0, 100, "borg_town_capability.rs:3139", "#978: maximum failure chance for a learned restoration spell to replace the potion reserve."),
    town_restore_life_min_depth => (12, "level", 1, 127, "borg_town_capability.rs:3133", "#1103: deepest dungeon level reached before the Restore Life Levels reserve applies. 12 is the first native depth of an experience-draining blow (monster.txt: grape jelly and nether worm mass)."),
    town_restore_life_min_gold => (0, "gold", 0, 10_000_000, "borg_town_capability.rs:1044", "#1103: purse floor for preventive purchases of Restore Life Levels. Owned potions, free Home withdrawals, and purchases to treat current experience drain are independent of this floor. 0 = no floor."),
    town_restore_life_reserve => (0, "count", 0, 10, "borg_town_capability.rs:3132", "#978: reserve of Restore Life Levels potions from maximum depth 12 without repeatable restoration. The town planner buys and keeps the reserve, dungeon junking protects it unless a pack slot is urgently needed, and extras have diminishing purchase value. 0 = off."),
    town_confusion_cure_reserve => (0, "count", 0, 10, "borg_town_capability.rs:1131", "#837 (owner): while the character is not confused, the first this many confusion cures (Cure Serious or Critical Wounds, Clear Mind, a Curing staff) in the pack are a requirement, as Recall's reserve: each worth town_confusion_cure_reserve_percent of the restore weight, nothing past them, so the first trip buys one and the weight repair does not store it at the Home. Only a full cure counts (Cure Serious or Critical Wounds, Clear Mind, a Curing staff: one unit each): not confused, a Cure Light Wounds supplies none, as its TIMED_DEC is clamped to the timer, which is 0. 0 = the saturating curve."),
    town_confusion_cure_reserve_percent => (100, "percent", 0, 1000, "borg_town_capability.rs:1134", "#837: each reserved confusion cure's weight, in percent of the town restore weight (BORG_TOWN_W_RESTORE). Only with town_confusion_cure_reserve > 0."),
    town_csw_reserve_percent => (0, "percent", 0, 1000, "borg_town_capability.rs:1420", "#1162: after light, food, Phase Door and any due Recall are covered, bonus for a shop Cure Serious Wounds as percent of the town restore weight. A carried full confusion cure satisfies the first copy. The second copy gets one quarter of the weight, with no bonus past two. 0 = off. Hypothesis pending Zoo measurement."),
    town_store_only_overweight => (0, "flag", 0, 1, "borg_town_utility.rs:312", "#711 (owner, 2026-11-20: \"only when it weighs too much they should stash the object with less value at home, until they are not overcarrying again\"): 1 = the live town planner stores a pack stack in the Home only while the pack is over its weight allowance, the lowest keep value per weight first, until it is under; 0 = today (a stack also goes home when its value kept for later, borg_town_home_discount, beats its value carried now: 329 stores in 18 classes-18 games, 130 of them fight consumables)."),
    town_departure_free_weight => (0, "tenths of a pound", 0, 10_000, "borg_town_utility.rs:284", "#859 (owner: \"every time they leave they should leave with some space for a few objects of a different kind\"): the live town planner stores pack stacks at the Home until this much weight is free under the carry allowance (BI_CARRY / 2), plus the room the Word of Recall reserve still needs, and its purchases and Home withdrawals leave it free. 0 = off (#711: storing stops at the bare allowance, so a pack at the allowance cannot take a 0.5 lb Recall)."),
    town_departure_free_slots => (0, "slots", 0, 23, "borg_town_utility.rs:285", "#859: with the same rule, the pack slots left free on a town departure (plus a slot for the Recall reserve when no Recall is carried). Storing frees whole stacks for slots, the least valuable per weight first. 0 = off."),
    town_shop_overloaded => (1, "flag", 0, 1, "borg_town_live.rs:334", "#1239: 1 shops under the regular gold and slot rules while overweight, then prepares departure using schema 21. 0 keeps the schema 20 carry limits and town routing for historical fixtures or a control arm."),
    town_quiver_trim_reserve => (0, "plain quivered ammunition", 0, 999, "borg_town_live.rs:410", "#1168: when the town planner would dispose of pack stock or leave above the carry allowance, drop complete stacks of fully known, unenchanted launcher ammunition with no brace annotation while keeping at least this many plain shots in the quiver. Zero disables the check."),
    town_home_reject_unwanted => (1, "flag", 0, 1, "borg_town_capability.rs:815", "#1049: sell or drop known kinds explicitly excluded from purchases, and clear them from Home. Zero retains the earlier storage policy."),
    town_home_stack_limit => (0, "Home stacks per known consumable kind", 0, 24, "borg_town_utility.rs:317", "#1048: stop depositing additional Home stacks and dispose of surplus when the carried reserve is covered. Zero disables the limit."),
    town_store_keep_min => (0, "copies", 0, 99, "borg_town_utility.rs:313", "#859 (owner: \"prefer objects that they have an abundance of, typically healing potions and phase doors, ideally we want to have variety in the backpack\"): when the pack must free weight, the copies of a stack past this many go to the Home first, the least valuable copy per weight first (the tenth Cure Light Wounds before the only Detect Invisible); a stack that serves a need (potions, scrolls, torches, food) or of flasks of oil is kept whole at or below this many copies (not stored, sold or dropped), so the live executor, which moves one copy per solve, stops there; an item that serves no need has no floor, and short of slots whole stacks go. 0 = off (whole stacks, lowest keep value per weight)."),
    town_variety_min_depth => (0, "depth", 0, 127, "borg_town_capability.rs:2953", "#863 (owner: \"it is important that in the first level we have the staples\"): the town planner's variety logic (the town_staple_* decay and the town_prepared_* remedies) starts once the deepest level reached (BI_MAXDEPTH) is at least this; before, the first visits buy the staples alone as before #863 (healing, Phase Door, oil, food, light, missiles, Recall). 0 = always on. Per class with @<class> or __<class>."),
    town_device_percent => (0, "percent", 0, 10_000, "borg_town_capability.rs:1205", "#895 (owner: \"we are going to need space for other devices\"): above 0, the live town planner values the charges of a learned wand, staff or rod on one requirement row per situation it answers (a ranged attack, an escape, detection, monster control), the first town_device_<situation> charges together worth this percent of the trip weight (borg_town_w_trip), so a charged device is kept and bought as variety and not only as one more source on a shared curve. Charges count scaled by town_devices_percent (per class; a different setting, the device skill). Starts with town_variety_min_depth. 0 = off. Per class with @<class> or __<class>."),
    town_device_attack => (10, "charges", 1, 99, "borg_town_capability.rs:1206", "#895: with town_device_percent, the charges of ranged attack devices (Magic Missile, Stinking Cloud, damage rods) wanted. Per class with @<class> or __<class>."),
    town_device_escape => (5, "charges", 1, 99, "borg_town_capability.rs:1206", "#895: with town_device_percent, the charges of escape devices (a Staff of Teleportation) wanted. Per class with @<class> or __<class>."),
    town_device_detect => (5, "charges", 1, 99, "borg_town_capability.rs:1207", "#895: with town_device_percent, the charges of detection devices (Detect Invisible, Detect Evil, Mapping) wanted. Per class with @<class> or __<class>."),
    town_device_control => (5, "charges", 1, 99, "borg_town_capability.rs:1207", "#895: with town_device_percent, the charges of monster control devices (Slow, Hold, Confuse Monster) wanted. Per class with @<class> or __<class>."),
    town_device_escape_min_gold => (0, "gold", 0, 10_000_000, "borg_town_capability.rs:1202", "#895 (owner: a Staff of Teleportation only with spare gold): below this purse after the urgent needs, a Staff of Teleportation on offer supplies nothing (neither its device row nor the shared teleport curve), so it is not bought; one already carried is never touched, so it is not sold for a small purse. The threshold is on the purse, not the price. 0 = no floor."),
    town_black_market_min_gold => (0, "gold", 0, 10_000_000, "borg_town_budget.rs:326", "#1103 (owner: \"keep and buy whenever it finds on the black market up from a certain budget\"; \"flat 6000 starting with a level negative delta of 100\"): below this purse (less town_black_market_level_delta a character level) the live town planner buys nothing at the Black Market but an urgent need (its spending share for ordinary purchases is 0), above it the borg_black_market_max_percentage cap applies as before, so a ware the planner values there - the potions, staves, rods and wands the core uses, and a wearable that raises borg_power - can be bought, at the engine's 3x Black Market price, if it fits the cap, the room and the other purchase rules. The urgent needs keep the cap below the threshold. 0 = no threshold (the cap alone)."),
    town_black_market_level_delta => (0, "gold per level", 0, 100_000, "borg_town_budget.rs:327", "#1103 (owner: a level negative delta of 100): the Black Market threshold town_black_market_min_gold falls by this much for each character level, max(0, min_gold - delta x level): 6000 and 100 is 3000 at level 30 and 1000 at level 50. 0 = the flat threshold."),
    town_restore_stat_percent => (0, "percent", 0, 10_000, "borg_town_capability.rs:1156", "#1103: while no stat is drained, give the town planner one copy of a Vigor or Purging mushroom as a reserve at this percent of one stat restoration. The dungeon has its own disposal policy. 0 = off."),
    town_restore_stat_min_depth => (0, "level", 0, 127, "borg_town_capability.rs:2961", "#1103: minimum maximum depth for the town restoring-mushroom reserve. 0 = starts with town_variety_min_depth."),
    town_second_stat_potions_by_class => (0, "flag", 0, 1, "borg_town_capability.rs:924", "#1103: only value shop offers of mixed gain/loss stat potions the class will drink. 0 = off."),
    town_prepared_percent => (0, "percent", 0, 10_000, "borg_town_capability.rs:1228", "#863 (owner: \"Better 4 cure poison, 4 detect invisible and 3 remove curse in the pack and 20 light wounds in the pack than 40 light wound potions\"; \"the point is being prepared to a variety of situations\"): above 0, the live town planner values each situation's remedy (Neutralize Poison, Resist Poison or Cure Critical Wounds; Remove Curse; Detect Invisible; Cure Serious or Critical Wounds for confusion; Boldness, Heroism or Berserk Strength for fear) at this percent of the trip weight (borg_town_w_trip) a copy, for the first town_prepared_<remedy> copies and nothing past them, even while the condition is absent, so the first copies of each remedy beat the 21st Cure Light Wounds. The condition-present curves stay on top. 0 = off (a remedy is worth nothing, or 1 percent, until the condition shows). Per class with @<class> or __<class>."),
    town_prepared_poison => (3, "copies", 1, 20, "borg_town_capability.rs:1246", "#863 (owner: \"if I have 20 CLW I would have say 4 detect invisible and 2 remove curse\"): with town_prepared_percent, the copies of the poison remedy (Neutralize Poison, Resist Poison, Cure Critical Wounds) the live town planner wants carried: each is worth town_prepared_percent of the trip weight, a copy past them nothing. Per class with @<class> or __<class>."),
    town_prepared_invisible => (4, "copies", 1, 20, "borg_town_capability.rs:1248", "#863 (owner: \"if I have 20 CLW I would have say 4 detect invisible and 2 remove curse\"): with town_prepared_percent, the copies of the Detect Invisible the live town planner wants carried: each is worth town_prepared_percent of the trip weight, a copy past them nothing. Per class with @<class> or __<class>."),
    town_prepared_curse => (2, "copies", 1, 20, "borg_town_capability.rs:1247", "#863 (owner: \"if I have 20 CLW I would have say 4 detect invisible and 2 remove curse\"): with town_prepared_percent, the copies of the Remove Curse the live town planner wants carried: each is worth town_prepared_percent of the trip weight, a copy past them nothing. Per class with @<class> or __<class>."),
    town_prepared_confusion => (2, "copies", 1, 20, "borg_town_capability.rs:1248", "#863 (owner: \"if I have 20 CLW I would have say 4 detect invisible and 2 remove curse\"): with town_prepared_percent, the copies of the confusion remedy (Cure Serious or Critical Wounds) the live town planner wants carried: each is worth town_prepared_percent of the trip weight, a copy past them nothing. Per class with @<class> or __<class>."),
    town_prepared_fear => (2, "copies", 1, 20, "borg_town_capability.rs:1249", "#863 (owner: \"if I have 20 CLW I would have say 4 detect invisible and 2 remove curse\"): with town_prepared_percent, the copies of the fear remedy (Boldness, Heroism, Berserk Strength) the live town planner wants carried: each is worth town_prepared_percent of the trip weight, a copy past them nothing. Per class with @<class> or __<class>."),
    town_prepared_poison_percent => (0, "percent", 0, 10_000, "borg_town_capability.rs:1246", "#1134 (owner, 2026-11-23: \"make the town planner start buying CCW and cure poison in the same way that they buy detect invisible or other when they have money to spare\"): with town_prepared_percent on, above 0 each of the first town_prepared_poison copies of the poison remedy (Neutralize Poison, Resist Poison, Cure Critical Wounds) is worth this percent of the trip weight instead of town_prepared_percent. The rows are priced per copy whatever the potion costs, so at the shared percent a 15 gold Detect Invisible beats a 75 gold Neutralize Poison; about 5x town_prepared_percent gives the two the same value per gold. 0 = town_prepared_percent. Hypothesis, not measured. Per class with @<class> or __<class>."),
    town_prepared_ccw => (0, "copies", 0, 20, "borg_town_capability.rs:1254", "#1134 (owner, 2026-11-23, as town_prepared_poison_percent): with town_prepared_percent on, the Potions of Cure Critical Wounds (60 HP; cures poison, cuts, stun, confusion, blindness, amnesia: object.txt) the live town planner wants carried as a remedy of their own: each of the first this many copies is worth town_prepared_ccw_percent of the trip weight, a copy past them nothing, so a Borg with gold to spare after the urgent needs and the staples buys them. 0 = off (no row; the model keeps its shape). Hypothesis, not measured. Per class with @<class> or __<class>."),
    town_prepared_ccw_percent => (0, "percent", 0, 10_000, "borg_town_capability.rs:1254", "#1134: with town_prepared_ccw above 0, each of its copies is worth this percent of the trip weight. 0 = town_prepared_percent. A 200 gold potion against a 15 gold scroll: 14x town_prepared_percent (200/15 = 13.3) gives at least the same value per gold. Hypothesis, not measured. Per class with @<class> or __<class>."),
    town_prepared_neutralize => (0, "copies", 0, 20, "borg_town_capability.rs:1255", "#1137 (owner, 2026-11-23: \"40 just Neutralize Poison\"): with town_prepared_percent on, the Potions of Neutralize Poison (cure poison: object.txt) the live town planner wants carried on a row of their own, beside the shared poison row: each of the first this many copies is worth town_prepared_neutralize_percent of the trip weight, a copy past them nothing. Unlike town_prepared_poison_percent it raises Neutralize Poison alone, not Resist Poison, Mushroom of Purging or CCW. 0 = off (no row; the model keeps its shape). Hypothesis, not measured. Per class with @<class> or __<class>."),
    town_prepared_neutralize_percent => (0, "percent", 0, 10_000, "borg_town_capability.rs:1255", "#1137: with town_prepared_neutralize above 0, each of its copies is worth this percent of the trip weight. 0 = town_prepared_percent. A 75 gold potion against a 15 gold scroll: 5x town_prepared_percent gives the same value per gold. Hypothesis, not measured. Per class with @<class> or __<class>."),
    town_prepared_dear_after_recall => (0, "flag", 0, 1, "borg_town_capability.rs:1242", "#1137 (owner, 2026-11-23: \"try to use the existing model and push it in a way that when he is rich enough to get a WoR he starts buying antipoison and CCW\"): 1 = the dear prepared rows (town_prepared_ccw, town_prepared_neutralize) count only once the purse after the urgent needs reaches buy_recall_from_gold, the Recall reserve's own floor, and the Recall reserve (town_recall_reserve, at least one scroll) is carried; before, they are worth nothing and town_prepared_poison_percent falls back to town_prepared_percent, so the gold goes to Recall and the staples first. 0 = no gate. Hypothesis, not measured. Per class with @<class> or __<class>."),
    town_staple_clw => (0, "copies", 0, 99, "borg_town_capability.rs:1345", "#863 (owner: \"make staple objects start losing value\"; \"it must be a decaying curve\"): the live town planner's Potion of Cure Light Wounds (20 HP) keeps its value for the first this many copies carried or bought and values each later copy at town_staple_decay_percent of the copy before it (the curve then sees the decayed amount). 0 = no decay for it. Per class with @<class> or __<class>."),
    town_staple_csw => (0, "copies", 0, 99, "borg_town_capability.rs:1346", "#863 (owner: \"make staple objects start losing value\"; \"it must be a decaying curve\"): the live town planner's Potion of Cure Serious Wounds (40 HP) keeps its value for the first this many copies carried or bought and values each later copy at town_staple_decay_percent of the copy before it (the curve then sees the decayed amount). 0 = no decay for it. Per class with @<class> or __<class>."),
    town_staple_ccw => (0, "copies", 0, 99, "borg_town_capability.rs:1112", "#863 (owner: \"make staple objects start losing value\"; \"it must be a decaying curve\"): the live town planner's Potion of Cure Critical Wounds (60 HP) keeps its value for the first this many copies carried or bought and values each later copy at town_staple_decay_percent of the copy before it (the curve then sees the decayed amount). 0 = no decay for it. Per class with @<class> or __<class>."),
    town_staple_phase => (0, "copies", 0, 99, "borg_town_capability.rs:1153", "#863 (owner: \"make staple objects start losing value\"; \"it must be a decaying curve\"): the live town planner's Scroll of Phase Door keeps its value for the first this many copies carried or bought and values each later copy at town_staple_decay_percent of the copy before it (the curve then sees the decayed amount). 0 = no decay for it. Per class with @<class> or __<class>."),
    town_staple_oil => (0, "copies", 0, 99, "borg_town_capability.rs:1154", "#863 (owner: \"make staple objects start losing value\"; \"it must be a decaying curve\"): the live town planner's Flask of Oil kept to throw (with town_oil_apart) keeps its value for the first this many copies carried or bought and values each later copy at town_staple_decay_percent of the copy before it (the curve then sees the decayed amount). 0 = no decay for it. Per class with @<class> or __<class>."),
    town_staple_food => (0, "copies", 0, 99, "borg_town_capability.rs:1155", "#863 (owner: \"make staple objects start losing value\"; \"it must be a decaying curve\"): the live town planner's food (a copy is a Ration, 30 nutrition; a Remove Hunger scroll counts as the food it adds) keeps its value for the first this many copies carried or bought and values each later copy at town_staple_decay_percent of the copy before it (the curve then sees the decayed amount). 0 = no decay for it. Per class with @<class> or __<class>."),
    town_staple_light => (0, "copies", 0, 99, "borg_town_capability.rs:1156", "#863 (owner: \"make staple objects start losing value\"; \"it must be a decaying curve\"): the live town planner's torch (a copy is borg_town_light_turns of light); with a lantern it is ignored keeps its value for the first this many copies carried or bought and values each later copy at town_staple_decay_percent of the copy before it (the curve then sees the decayed amount). 0 = no decay for it. Per class with @<class> or __<class>."),
    town_staple_decay_percent => (80, "percent", 0, 100, "borg_town_capability.rs:1341", "#863: with the town_staple_* free copies, the percent of the previous copy's value each staple copy past them keeps (100 = no decay; 0 = a copy past them adds nothing). Per class with @<class> or __<class>."),
    town_heal_curve_scale_percent => (100, "percent", 1, 10_000, "borg_town_capability.rs:755", "#491: stretches the live town planner's diminishing healing curve: weight and demand scale together, keeping the first HP's value and moving the half-value point to this percent of today's (100 = today)."),
    town_phase_curve_scale_percent => (100, "percent", 1, 10_000, "borg_town_capability.rs:1114", "#682 (owner): stretches the live town planner's Phase Door curve, as town_heal_curve_scale_percent does healing's: weight and demand scale together (100 = today). Per class with town_phase_curve_scale_percent@<class> or the class files."),
    town_oil_curve_scale_percent => (100, "percent", 1, 10_000, "borg_town_capability.rs:878", "#682 (owner): weights a flask of oil's thrown damage in the live town planner (100 = today), so a class buys more or fewer flasks to throw; flasks kept as lantern fuel are not changed. Per class with town_oil_curve_scale_percent@<class> or the class files."),
    town_ranger_arrows_max => (0, "count", 0, 1000, "borg_town_capability.rs:1275", "#716: the soft cap: the town planner's missile reserve (Ranger usable shots) as a requirement: each shot up to this many is worth the same, past it nothing. 0 = C (a curve that grows with the trip and never stops paying). Per class with @<class>."),
    town_ranger_arrows_curve_scale_percent => (0, "percent", 0, 10_000, "borg_town_capability.rs:1276", "#716: with town_ranger_arrows_max (the soft cap), shots past the cap count on C's shot curve with weight and demand scaled by this percent, as town_heal_curve_scale_percent does healing: each worth less than the last, never nothing. 0 = nothing past the cap. Per class with __<class>."),
    town_oil_apart => (0, "flag", 0, 1, "borg_town_capability.rs:704", "#712: 1 = the town planner values each flask of oil kept to throw on a curve of its own (thrown oil), beside its damage on the shared ATTACK curve that launcher ammunition saturates; 0 = C (ATTACK only). Per class with @<class>."),
    town_invisible_count => (0, "reads", 0, 10, "borg_town_capability.rs:1428", "#831 (owner): the town planner values Detect Invisible (scrolls, or a staff's charges) on a diminishing curve like Phase Door's and the flasks': the first read is worth about town_invisible_percent of the trip's escape weight, the curve reaches half its value at this many reads, and nothing is a requirement (no reserve): it competes with the rest of the purse. 0 = #663: worth 1% of the support weight, never bought."),
    town_invisible_percent => (20, "percent", 0, 10_000, "borg_town_capability.rs:2889", "#831: with town_invisible_count, the curve's first value per read (a read's realised first value is this times count / (count + 1)) for a class that can neither cast Detect Evil nor a spell that reveals invisible monsters (Warriors, Blackguards, and others until they learn one), in percent of the trip's escape weight. A hypothesis."),
    town_invisible_evil_percent => (10, "percent", 0, 10_000, "borg_town_capability.rs:2888", "#831: as town_invisible_percent for a class that can cast Detect Evil (it reveals the evil invisible monsters, ghosts and a poltergeist, not a clear icky thing) and no spell that reveals invisible monsters. A hypothesis."),
    town_invisible_spell_percent => (3, "percent", 0, 10_000, "borg_town_capability.rs:2887", "#831: as town_invisible_percent for a class that can cast Sense Invisible, Detection or Reveal Monsters (borg_spell_legal). A hypothesis."),
    town_phase_reserve => (0, "count", 0, 10, "borg_town_capability.rs:1120", "#761 (owner: \"we need to value phase doors much more, especially for rangers\"): the live town planner values this many Phase Door scrolls as a requirement on top of the Phase Door curve, each worth town_phase_reserve_percent of the trip's escape weight, nothing past the reserve, so the first ones are bought before arrows and potions (0 = off, as before). Per class with town_phase_reserve@<class> or __<class>."),
    town_phase_reserve_percent => (100, "percent", 0, 10_000, "borg_town_capability.rs:1122", "#761: with town_phase_reserve, what each reserved Phase Door is worth, in percent of the trip's escape weight (the Phase Door curve's first scroll is about a third of that at 150 percent scale). Per class with @<class> or __<class>."),
    town_first_cure => (0, "count", 0, 10, "borg_town_capability.rs:3047", "#987 (a 127-gold Ranger bought 6 Phase Door and no Cure Light Wounds, then died to a crow with nothing to drink): the live town planner values the first this many healing potions (Cure Light, Serious or Critical Wounds and the other potions that heal) as a requirement, each worth town_first_cure_percent of the trip's escape weight, nothing past them, on top of the healing curve and reserve. The purchase order is decided on the whole wares within the purse (a cure's curve and reserve shares against a reserved scroll's), so the percent is a margin, not a switch: at the #987 record (14 HP) a first cure at 200 is worth 410,588 against a reserved Phase Door's 219,130, and a percent below town_phase_reserve_percent can still put the cure first. 0 = off (today: each reserved Phase Door outbids the first cure). Per class with @<class> or __<class>."),
    town_first_cure_percent => (200, "percent", 0, 10_000, "borg_town_capability.rs:1361", "#987: with town_first_cure, what each of the first healing potions is worth, in percent of the trip's escape weight, the unit of town_phase_reserve_percent. A margin on the cure's whole value, not a threshold against the reserve's percent. Per class with @<class> or __<class>."),
    town_ranged_reserve => (0, "count", 0, 100, "borg_town_capability.rs:1118", "#789: first N carried flasks of oil available to throw or usable launcher missiles, priced as a town shopping requirement. With town_ranged_readiness, this is a floor on the size of the shared reserve. Per class with @<class> or __<class>. 0 disables the legacy-only reserve."),
    town_ranged_readiness => (0, "flag", 0, 1, "borg_town_capability.rs:505", "#1184: value a usable supply of ranged attacks for the next trip within the existing shopping budget, after urgent purchases. Honor town_ranged_reserve as a lower bound. Experimental."),
    trip_ranged_readiness => (0, "flag", 0, 1, "borg_trip.rs:52", "#1184: seek ranged supplies in the dungeon and return to town when short and resupply is feasible. Experimental."),
    ranged_readiness_target => (6, "uses", 1, 100, "borg_ranged.rs:57", "#1184: starting target for effective ranged uses per trip. Renewable spells or devices lower the finite reserve. Below half the target, rounded up, recovery can start."),
    ranged_readiness_damage_depth => (2, "levels", 1, 100, "borg_ranged.rs:58", "#1184: dungeon levels per point of minimum average ranged damage, with a minimum of one damage."),
    ranged_readiness_trace => (0, "flag", 0, 1, "borg_ranged.rs:400", "#1184: record ranged capability transitions in ranged-readiness.jsonl for the Zoo. Observation only."),
    town_unseen_percent => (100, "percent", 0, 10_000, "borg_town_capability.rs:839", "#761 (owner: \"unknown flavours ... a coefficient per class\"): the live town planner values a shop ware whose flavour the player has not seen (shown {unseen}: a Potion or Scroll at birth, so Cure Light Wounds too) at this percent of what it supplies, Phase Door excepted (town_phase_*); 100 = today. Not a buy-to-identify weight: that one (borg_town_w_knowledge) is forced to 0 in the plan (borg_town_shadow.rs effective_params). Per class with @<class> or __<class>."),
    town_food_percent => (100, "percent", 0, 10_000, "borg_town_capability.rs:1182", "#761 (owner): the live town planner's food weight, in percent of borg_town_w_food (100 = today); per class with @<class> or __<class>, which borg_town_w_food is not."),
    town_light_reserve_turns => (0, "turns", 0, 60000, "borg_town_capability.rs:1128", "#810: value enough equipped and spare fuel to cover this many dungeon turns. While using a torch, replenish from Home or the General Store after immediate needs. Zero preserves ordinary fuel shopping."),
    town_lantern_upgrade_power => (0, "power", 0, 100000, "borg_town_shadow.rs:180", "#810: experimental equipment preference for a fueled lantern replacing a torch. Zero leaves normal try-on scoring."),
    town_light_percent => (100, "percent", 0, 10_000, "borg_town_capability.rs:1124", "#761 (owner): the live town planner's light weight (torches, fuel), in percent of borg_town_w_light (100 = today); per class with @<class> or __<class>, which borg_town_w_light is not."),
    town_oil_flasks => (10, "count", 1, 255, "borg_town_capability.rs:1406", "#712: with town_oil_apart, the thrown-oil curve is at half its value with this many flasks kept to throw."),
    town_heal_speed_percent => (0, "percent of max HP", 0, 100, "borg_town_capability.rs:610", "#491: healing speed. A HEAL_HP use that restores at least this percent of max HP counts its HP in full in the live town planner's healing and healing reserve; a smaller one counts heal x heal / (this percent of max HP) (0 = HP alone). With town_heal_reserve_real_hp = 1 speed weights only the healing curve; with 0 also the reserve."),
    town_heal_reserve_hp_percent => (0, "percent of max HP", 0, 10_000, "borg_town_capability.rs:764", "#491: healing, as a percent of max HP, the live town planner values linearly at the curve's first-HP value on top of the curve; bought first, never sold below (0 = off)."),
    town_heal_reserve_real_hp => (0, "flag", 0, 1, "borg_town_capability.rs:668", "#487: 1 = the healing reserve counts the real HP a potion restores; town_heal_speed_percent then weights only the healing curve (which kind is worth more), not how many potions fill the reserve; 0 = speed-weighted HP (#491 as first shipped)."),
    town_digger_min_gold => (0, "gold", 0, 10_000_000, "borg_town_capability.rs:1399", "#487: below this much gold the live town planner values no digging (a Shovel only clears rubble, which a weapon also digs); 0 = off."),
    town_missiles_percent => (-1, "percent_override", -1, 1000, "borg_town_capability.rs:713", "#666 (owner): the town plan's reserve of shots for the class's launcher, in percent of borg_town_w_ranger_missiles, for any class (set per class by shooting skill in harness/borg_<class>.txt). -1 = C: the Ranger's reserve only."),
    town_missiles_other_ranged_percent => (100, "percent", 0, 100, "borg_town_capability.rs:1263", "#666 (owner): \"projectile buying is more important if they cannot get flasks of oil or wands\". With at least 5 flasks of oil and attack-wand charges (Magic Missile, Stinking Cloud) in the pack, the reserve of shots weighs this percent. 100 = no change."),
    town_devices_percent => (100, "percent", 0, 1000, "borg_town_capability.rs:967", "#666 (owner): a wand, rod or staff is worth this percent of what it supplies in the town plan (set per class by device skill in harness/borg_<class>.txt). 100 = as computed."),
    caution_rising_explore_ticks => (300, "ticks", 0, 100000, "borg_caution.rs:7048", "#483: while rising to town, Borg ticks (one per think; restarted on each new level) during which a descent other than a desperate one is held so the Borg explores for an up staircase; 0 = off (C's behaviour)."),
    caution_rising_descend_danger_tenths => (10, "tenths", 0, 1000, "borg_caution.rs:7044", "#483: while rising, danger here above avoidance x this / 10 lets a descent through again."),
    caution_rising_skip_held_stairs => (0, "flag", 0, 1, "borg_caution.rs:7060", "#1159: 1 = skip known down-stair targets the rising hold would refuse on arrival, and replan when a committed stair route becomes held. 0 retains the older route selection."),
    caution_stair_low_level => (5, "level", 1, 51, "borg_caution.rs:4371", "Controls stair low level."),
    caution_stair_danger_divisor => (2, "divisor", 1, 100, "borg_caution.rs:4372", "Controls stair danger divisor."),
    caution_cornered => (0, "flag", 0, 1, "borg_caution.rs:6728", "#504: 1 = cornered (no Phase Door, teleport or escape, a monster near, danger at or above the HP) the Borg uses every buff it has (fear cure, Speed, Heroism, Berserk Strength, Blessing) regardless of their usual danger windows, reads Monster Confusion against an adjacent big monster, and a melee class neither retreats nor backs away; 0 = off."),
    caution_cornered_danger_tenths => (10, "tenths", 0, 1000, "borg_caution.rs:6729", "#504: cornered when danger here x 10 >= current HP x this (10 = danger at or above the HP)."),
    caution_cornered_radius => (2, "grids", 0, 100, "borg_caution.rs:6731", "#504: cornered only with a known monster within this many grids (a fight is on)."),
    caution_breeder_boost_danger_tenths => (0, "tenths", 0, 1000, "borg_caution.rs:6783", "#504 (owner): standing against breeders with no easy escape, danger here at or above the HP x this / 10 makes the Borg use the cornered boosts (fear cure, Speed, Heroism, Berserk Strength, Blessing); 0 = off."),
    caution_cornered_spell_fail => (25, "percent", 0, 100, "borg_caution.rs:6872", "#504: cornered, a buff spell (Haste Self, Heroism, Berserk Strength, Bless) is cast only at or below this failure percent, so a failing cast is not retried turn after turn under fire."),
    caution_emergency_speed_danger_percent => (0, "percent of HP", 0, 10_000, "borg_caution.rs:6859", "#913: above 0, borg_caution drinks a Potion of Speed (or casts Haste Self, at most caution_cornered_spell_fail percent failure) after the healing paths (and, where they act, the #1140 poison race, the immobile step-away and borg_defend) and before the swap, the stair walks, the retreat and the escapes, fleeing or not, when the summed danger of the awake known monsters within caution_emergency_speed_radius that are faster than the Borg by caution_emergency_speed_margin (a haste forecast: an observed FAST counts +10, a monster's SLOW is not subtracted, mon-move.c:1928-1938) reaches this percent of the current HP. borg_defend's Speed is never asked while fleeing, #469's only inside borg_attack, #504's only without Phase Door. 0 retains today's play (compiled). Shipped 0 (owner, 2026-11-23); 50, the #747 stair test's share (caution_desperate_fast_danger_percent), is the value to try (hypothesis). It reads st.borg.temp.fast, not trait_timed_sync: with #941 off it can fire before the game's first Potion of Speed, and after it the stuck flag makes it return at the already-fast guard; only a drink after the game's first Speed (any rule's) needs trait_timed_sync = 1 (#941). A failed Haste Self cast leaves the Borg eligible again."),
    caution_emergency_speed_radius => (2, "grids", 1, 20, "borg_caution.rs:6865", "#913: with caution_emergency_speed_danger_percent, the distance within which a faster monster counts. Hypothesis."),
    caution_emergency_speed_margin => (5, "speed points", 0, 40, "borg_caution.rs:6866", "#913: with caution_emergency_speed_danger_percent, a monster is faster at this many speed points above the Borg (at least 1). Hypothesis: as fast_fragile_speed_margin."),
    caution_cornered_throw_first => (0, "choice", 0, 2, "borg_caution.rs:7001", "#719 (owner): \"if I am cornered I should throw missile and then boost myself\". 1 = while no monster is adjacent and an attack from range would do damage (borg_attack_estimate), the cornered boost waits and the Borg attacks first; it buffs once a monster is next to it or nothing reaches. 2 = the same for the hard breeder fight's boost (the Phase Door rule's boost already follows its own attack, so mode 2 adds nothing there). 0 = buff first (today)."),
    caution_cornered_big_level_margin => (0, "levels", 0, 100, "borg_caution.rs:6812", "#504: an adjacent monster is big (Monster Confusion) when unique or its level exceeds the Borg's by more than this."),
    caution_breeder_flee_max_level => (15, "level", 1, 51, "borg_caution.rs:4622", "Controls breeder flee max level."),
    caution_breeder_fight => (0, "flag", 0, 1, "borg_caution.rs:5371", "#503: 1 = with breeders on the level and no easy escape the Borg stands and fights (no breeder flee, no breeder scary level, no breeder 'no recall' flee), and below clevel 10 it hunts breeders while few (caution_breeder_hunt_max_count, per class since #558); 0 = off (C's behaviour), which also turns off recover_breeder_hunt_hp_percent and danger_hunt_breeder_no_acid_fear."),
    caution_breeder_easy_escape_steps => (5, "grids", 0, 100, "borg_caution.rs:5372", "#503: a free staircase within this many grids (the engine's distance) is an easy escape from breeders (as is a Phase Door, teleport, Teleport Level, escape or Word of Recall)."),
    caution_breeder_stand_radius => (5, "grids", 0, 100, "borg_caution.rs:5753", "#503: standing against breeders, with a breeder within this many grids and no other known monster within danger_monster_max_distance (on the larger axis distance, borg_danger's reach), the Borg neither retreats nor backs away from their danger (being surrounded still moves it)."),
    caution_scary_named_monsters => (1, "flag", 0, 1, "borg_caution.rs:6476", "#503: 1 = the C Borg's hard-coded scary monsters by character level (squint-eyed rogue, Grip, Fang, small kobold, soldier, cutpurse, ..., Bullroarer, breeders, cave spider, ..., hounds, ancient dragons, Beorn, Dread) and the low-level door scares mark the level scary (leave and flee) until a new level; 0 = off (the resist-based scares stay). The data file's ignore lines (#621) don't depend on it."),
    caution_town_never_scary => (0, "flag", 0, 1, "borg_caution.rs:6510", "#604 (owner): 1 = in town no monster makes the level scary (the named list, the damage rule, the resist scares): the Borg meets town monsters by its danger rules instead of fleeing to DL1; 0 = C."),
    town_hunt_min_hp_percent => (0, "percent", 0, 100, "borg_town_hunt.rs:140", "#789: retreat during a town hunt when current HP is below this percentage of maximum HP. 0 disables the HP floor; the fight forecast still applies."),
    caution_town_scary_levels_less => (0, "levels", 0, 51, "borg_caution.rs:6469", "#622 (owner): a town monster (race level 0) on the data-file scary list (borg_scary_list; not the compiled C list) is scary for this many fewer character levels: clevel <= listed level - this. The list's order stays; 11 or more ends the town's list scares at once (the damage rule and resist scares still apply); 0 = the list as it is."),
    caution_town_flee_no_worse => (0, "flag", 0, 1, "borg_caution.rs:6493", "#647: 1 = in town, while the level is scary or the Borg flees or leaves, the flow refuses no grid at most as dangerous as the Borg's own (its fear is at least the danger here): a low-level Borg next to a filthy street urchin found every path to the stairs refused and twitched for 70,000 turns. 0 = C (the flow's fear alone)."),
    caution_scary_damage_percent => (0, "percent of max HP", 0, 1000, "borg_caution.rs:6540", "#505: a monster on the level is scary (leave and flee, as the named list) when one character turn of its melee (the average of its blows' random_value, base + dice + level bonus as randcalc AVERAGE, times its moves per character turn) takes at least this percent of max HP (owner: scary is a monster that can one- or two-shot you). Spells are not counted. 0 = off."),
    caution_scary_stand => (0, "flag", 0, 1, "borg_caution.rs:5409", "#530: 1 = in the dungeon a monster that never moves is never scary, and a scary level is fled only while the escape is real: a free staircase within caution_breeder_easy_escape_steps or an escape item, or no scary monster awake and at least as fast as the character; otherwise the Borg stands and fights (the scare's leave and flee are withdrawn). 0 = C (flee every scary level)."),
    caution_scary_latch => (1, "flag", 0, 1, "borg_caution.rs:5436", "#515: with caution_scary_stand, 1 = the C latch: a scary level stays scary until a new level; 0 = the flag is re-checked each scan of the known monsters, and when no scary monster remains the level stops being scary and the goals the scare set (leave, flee, flee to town) are withdrawn. Door scares stay latched."),
    caution_stairs_leave_on_monster => (0, "flag", 0, 1, "borg_caution.rs:7312", "#614: 1 = in the dungeon, a Borg standing on a staircase that still needs to recover (HP or SP below the maximum) takes the stairs as soon as a monster threatens it (danger here at least caution_stairs_leave_danger_percent of the current HP), instead of fighting or reading an escape; never in town (review of #616: the town's down staircase made it dive into DL1); 0 = C."),
    caution_stairs_leave_danger_percent => (10, "percent of current HP", 0, 1000, "borg_caution.rs:7311", "#614 (review of #616): with caution_stairs_leave_on_monster, the danger here, as a percent of the current HP, that counts as a monster threatening (at least 1): any danger above 0 fired at danger 1, a distant or harmless monster."),
    caution_stairs_leave_down => (1, "flag", 0, 1, "borg_caution.rs:7318", "#614 (review of #616): with caution_stairs_leave_on_monster, 1 = take the staircase the Borg stands on, up or down (the owner: climb up/down); 0 = an up staircase only (a down one goes deeper)."),
    caution_scary_unlatch_kill => (0, "flag", 0, 1, "borg_caution.rs:5437", "#590 (owner): with caution_scary_stand (the scary flag is decided after each scan), 1 = the level's scary flag clears once no scary monster is known and the last one known was killed (a monster of a race that made the level scary died since a scan last found one known), with caution_scary_latch on or off; forgetting the monster is not enough (it has not left the level). 0 = the latch decides (1: never on the level; 0: whenever none is known)."),
    caution_scary_unlatch_calmed => (0, "flag", 0, 1, "borg_caution.rs:5438", "#593 (owner): with caution_scary_stand, 1 = the level's scary flag clears once no scary monster is known and a named-list or damage-rule scary monster is known hurt or afraid in this scan (caution_scary_hurt_percent, caution_scary_afraid; resist-based scares are never calmed), or one was killed, with caution_scary_latch on or off; forgetting the monster is not enough. Only a race that made the level scary counts (#590's record): a monster hurt or afraid from first sight never made it scary. 0 = off."),
    caution_scary_hurt_percent => (0, "percent of HP", 0, 100, "borg_caution.rs:6564", "#539: a named-list or damage-rule scary monster that has lost at least this percent of its HP (kill.injury, from the health bar) makes nothing scary; resist-based and door scares are not affected. With caution_scary_latch 0 a level whose scary monsters are all hurt stops being scary. 0 = off (C)."),
    caution_scary_afraid => (0, "flag", 0, 1, "borg_caution.rs:6564", "#539: 1 = a named-list or damage-rule scary monster that is afraid (kill.afraid) makes nothing scary; resist-based and door scares are not affected. 0 = C."),
    caution_flee_known_down_stairs => (0, "flag", 0, 1, "borg_caution.rs:1159", "#544 (A), narrowed in #548: 1 = fleeing (or the level scary) with no free up staircase known, the Borg walks to a known down staircase with the desperate flow (danger ignored, monster grids routed around), only when cornered (caution_flee_known_down_cornered) and within caution_flee_known_down_max_cost grids of flow; 0 = off (C's behaviour). Keeping the stairs through the stuck jumpstart is caution_jumpstart_keep_stairs."),
    caution_flee_known_down_cornered => (1, "flag", 0, 1, "borg_caution.rs:1164", "#548: with caution_flee_known_down_stairs, 1 = only when cornered: a known monster that can move is adjacent (distance 1, diagonals included; #504 uses caution_cornered_radius), no Phase Door, teleport, Teleport Level or escape in the pack (items; spells are not counted), and the danger here at or above the HP or the Borg surrounded; 0 = any flight (#545 as merged: it fired on 13 of 18 classes-18 games, up to 721 times)."),
    caution_flee_known_down_max_cost => (40, "steps of flow", 0, 250, "borg_caution.rs:1165", "#548: with caution_flee_known_down_stairs, the known down staircase is taken only within this many steps of flow (the flow's depth from the staircases: the walk around walls, not the straight-line distance); 0 = any distance (#545 as merged)."),
    caution_jumpstart_keep_stairs => (0, "flag", 0, 1, "borg_caution.rs:7711", "#548 (K of #544, split from caution_flee_known_down_stairs): 1 = the stuck jumpstart (bravery boost 3) keeps the known up and down staircases instead of forgetting them; 0 = C (forgets them). Usually wanted with caution_flee_known_down_stairs: in the worm room (priest-1055640847) the narrowed flight escapes only with it. Alone it did not save that game (still dead at turn 91,153)."),
    caution_twitch_avoid_lava => (0, "flag", 0, 1, "borg_caution.rs:5600", "#597: 1 = the twitch's random step (borg_twitchy, borg_flow_misc.rs) never chooses lava without fire immunity (100 + 1d100 fire damage per step); if every direction is wall or lava the Borg rests. 0 = C (only walls are refused)."),
    caution_stair_parked_fallback => (0, "flag", 0, 1, "borg_caution.rs:5607", "#597: 1 = a stair flow that finds a monster marked on every known staircase of its kind (enqueue_stairs, borg_flow_stairs.rs) flows to them anyway, so the only known staircase is never refused for a monster on it, harmless or long gone. 0 = C (a parked staircase is skipped)."),
    caution_breeder_exit => (0, "flag", 0, 1, "borg_caution.rs:5767", "#544 (B): 1 = only breeders near, one adjacent, and danger here at or above the HP or surrounded: the Borg walks with the desperate flow to the nearest known chokepoint (a grid whose passable known neighbours form at least two separate groups: a corridor or doorway; unknown grids count as walls) at least caution_breeder_exit_distance grids from every known breeder; 0 = off."),
    caution_breeder_exit_distance => (6, "grids", 1, 100, "borg_caution.rs:5770", "#544 (B): an exit grid is at least this many grids (the engine's distance) from every known breeder."),
    caution_slow_crowd_exit => (0, "flag", 0, 1, "borg_caution.rs:5830", "#690 (generalising #544's breeder exit): 1 = surrounded (borg_surrounded), the danger here at or above the HP, a monster adjacent, and every awake monster within danger_monster_max_distance either never moving or slower than the Borg by caution_slow_crowd_exit_margin_percent (energy per game turn, extract_energy): the Borg walks with the desperate flow to the nearest known chokepoint or staircase at least caution_slow_crowd_exit_distance grids from every monster within that reach, or else to the nearest other known open grid that far; with neither reachable, the distance comes down by one to 2. Held while a faster or equally fast awake monster is near. 0 = off (C's behaviour)."),
    caution_slow_crowd_exit_distance => (6, "grids", 1, 100, "borg_caution.rs:5861", "#690: with caution_slow_crowd_exit, the first distance tried (the engine's distance) from every monster within danger_monster_max_distance; when no grid that far can be reached, it comes down by one to 2."),
    caution_slow_crowd_exit_margin_percent => (25, "percent", 0, 1000, "borg_caution.rs:5839", "#690: with caution_slow_crowd_exit, a monster is slower when the Borg's energy per game turn is at least 100 + this percent of the monster's (speed 110 against 100 is 10 against 5: 100% faster; 110 against 105, 10 against 7: 43%)."),
    caution_breeder_avoid => (0, "flag", 0, 1, "borg_caution.rs:6024", "#556: 1 = every grid a breeder is seen on (a moving breeder leaves a trail of marks, one per grid) is remembered for caution_breeder_avoid_turns; every flow but a kill flow (the stair and flee flows, exploration, items, the breeder exit) treats a grid within caution_breeder_avoid_radius of one as impassable, except the grids next to the Borg (its first step), so it neither walks into a colony nor back into one it lost from view; and a Phase Door landing grid there counts as scary. 0 = off (C's behaviour)."),
    caution_breeder_avoid_radius => (1, "grids", 0, 10, "borg_caution.rs:6045", "#556: with caution_breeder_avoid, a grid within this many grids (the engine's distance) of a remembered breeder is avoided; 0 = only the breeder's own grid."),
    caution_breeder_avoid_turns => (50, "borg turns", 1, 100000, "borg_caution.rs:6028", "#556: with caution_breeder_avoid, a breeder's grid is remembered this many Borg turns (`time.now`, one per think) after the breeder was last known there."),
    caution_breeder_avoid_fight_through => (1, "flag", 0, 1, "borg_caution.rs:6289", "#556 (owner 2026-11-19: \"fallback to fight our way out\"): with caution_breeder_avoid, 1 = a flight whose stair flow finds no route around the remembered breeders takes the plain route through them, and for caution_breeder_through_turns after, retreat and back-away stand down while the danger here is below the HP (the Borg kills what blocks the way and goes on); 0 = no route, the flight looks elsewhere."),
    caution_breeder_through_turns => (5, "borg turns", 0, 1000, "borg_caution.rs:6298", "#556: with caution_breeder_avoid_fight_through, retreat and back-away stand down this many Borg turns after the flight last took the route through the breeders (the flow is recomputed after every attack)."),
    caution_breeder_stuck_turns => (20, "borg turns", 0, 1000, "borg_caution.rs:6272", "#556: with caution_breeder_avoid, when the Borg is stuck (the dungeon think reaches its bravery boosts) while remembered breeders block its flows, the marks are ignored for this many Borg turns, so it can fight its way out instead of staying boxed in; 0 = never (the marks hold even when stuck)."),
    caution_breeder_policy => (-1, "mode", -1, 5, "borg_caution.rs:6086", "#722 (owner): the breeder policy, one setting for the breeder switches. -1 = the switches as set (caution_breeder_fight, danger_hunt_breeder_no_acid_fear, recover_rest_breeders, caution_breeder_stoppable, caution_breeder_avoid, caution_breeder_exit). 0 = C (all off). 1 = stand without an easy escape and hunt few breeders below CL10 (#503, #558). 2 = and no rest while breeders are known (#644). 3 = and kill them first while the kills outpace the breeding, at any count and level (#556); the level is lost once more breeders are known than caution_breeder_hunt_max_count (per class) or the breeding outpaces the kills; on a lost level the Borg searches for stairs, and reads Recall after caution_breeder_lost_recall_turns. 4 = and paths avoid colonies (#556). 5 = and walk out of a breeder room (#544)."),
    caution_breeder_lost_recall_turns => (500, "game turns", 0, 100000, "borg_caution.rs:6178", "#722 (owner: \"search for stairs first, but if paths are blocked by breeders, recall at some point\"): with caution_breeder_policy 3 or more, this many game turns after the level was lost to breeders the Borg reads Word of Recall, stairs known or not; until then a lost level's flight searches for stairs and does not read Recall. 0 = Recall at once."),
    caution_breeder_stoppable => (0, "flag", 0, 1, "borg_caution.rs:6061", "#556 (owner 2026-11-19: \"kill them first and only escape if it is not possible\"): 1 = on a level with breeders the Borg stands and hunts them (breeder_stand and breeder_hunt, at any count and character level, escape items or not) while the breeding can be stopped: the known breeders' expected births a turn (the engine's 1 in 8k while k < 4, monster speed against the character's), times caution_breeder_stoppable_births_percent, below the Borg's kills a turn (every attack it has: the attack planner's dry run, borg_attack_estimate, and the kills of the attack it would choose, for each monster hit its true damage over that monster's HP at most 1; the random state is restored after the dry run). A think is judged only when the planner has something to attack. With the danger where it stands at or above the HP it does not stand that think. While no staircase is known it fights on even then (a flight with nowhere to go). Once the breeding outpaces the kills, the level is lost to them until a new one, and the breeder flees act (with caution_breeder_avoid, the flight avoids the colony). 0 = #503's stand (no easy escape) and #558's hunt (caution_breeder_hunt_max_count, below clevel 10)."),
    caution_breeder_stoppable_births_percent => (200, "percent", 1, 10000, "borg_caution.rs:6109", "#556: with caution_breeder_stoppable, the expected births a turn are weighed by this percent against the kills a turn (200 = the Borg must kill twice as fast as they breed: it walks between them and does not see them all)."),
    caution_breeder_stoppable_confirm => (5, "thinks", 1, 1000, "borg_caution.rs:6113", "#556: with caution_breeder_stoppable, the level is lost to the breeders only after the breeding outpaced the kills on this many judged thinks in a row (a think is judged only when the attack planner has something to attack)."),
    caution_attack_price_status_blows => (0, "flag", 0, 1, "borg_caution.rs:4220", "#1010: 1 = attack on sight's target test (category_attack_safe) prices the blows that hurt without stopping the Borg fighting: POISON (its dice, plus its timer: 1 HP a world tick, 10 / energy a Borg turn, unless resisted), LOSE_*, EXP_*, EAT_*, DISENCHANT, DRAIN_CHARGES (their dice), instead of refusing the target as unknown damage. PARALYZE, CONFUSE, BLIND, TERRIFY, HALLU, SHATTER, BLACK_BREATH and unknown spells still refuse it. Lets the scary list's white and yellow worm mass entries act. The winning fight and the town hunt (which also need a HURT-only certificate) are unchanged. 0 retains today's play."),
    caution_stair_scum_threat => (0, "flag", 0, 1, "borg_caution.rs:6396", "#549: 1 = after fleeing town, no stair scum (grabbing DL1 items near the stairs before going back up) while the level is scary, the danger here is above caution_stair_scum_danger, or the HP is below the maximum: the Borg goes straight back up (and the scum's own GOAL_TAKE flow is cancelled). The HP condition is the broad one: any injury stops the scum; 0 = off (C's behaviour)."),
    caution_stair_scum_danger => (0, "danger", 0, 10000, "borg_caution.rs:6400", "#549: with caution_stair_scum_threat, a danger here above this stops the stair scum."),
    caution_rising_hold_flee_descent => (0, "flag", 0, 1, "borg_caution.rs:6427", "#557: 1 = while rising (a return trip on foot) in the dungeon, a flight takes no down staircase (stair_more, the both-stairs flee flow, the down-stairs flee flow) unless the danger here is at or above the HP; with birth_connect_stairs each such flight lands on DL+1's up staircase and the climb restarts on a fresh level; 0 = off (C's behaviour)."),
    caution_rising_hold_min_hp_percent => (0, "percent of max HP", 0, 100, "borg_caution.rs:6433", "#753: with caution_rising_hold_flee_descent, a fleeing Borg with less than this percent of its max HP is not held off a descent while rising (214960633: held at 2 of 47 HP on a down staircase, the danger there 0, next to a grey mold). 0 = today."),
    caution_breeder_hunt_max_count => (4, "count", 0, 1000, "borg_caution.rs:6085", "#503, #558: below clevel 10 the Borg hunts breeders while at most this many are known on the level (with caution_breeder_fight); a class value per class (the class files), 0 = that class does not hunt. Before #558 only Warriors, Paladins, Rogues and Blackguards hunted."),
    caution_breeder_danger_damage_only => (0, "flag", 0, 1, "borg_caution.rs:6089", "#1009: 1 = the breeder stand's one-think pause (caution_breeder_stoppable: danger here, horizon 1, at or above the HP) reads the danger without its fear prices (#878's params::damage_only: the danger_*_fear terms, the breeder and friends bonuses and the fear maps at 0). One white worm mass next to the CL3 Mage 461220874: 32 with fear, 2 without, so the stand holds and the retreat and back-away stay off. The boost has its own switch (caution_breeder_boost_damage_only). 0 retains today's play."),
    caution_breeder_boost_damage_only => (0, "flag", 0, 1, "borg_caution.rs:6789", "#1009 (review of #1012): 1 = the hard-breeder-fight boost (caution_breeder_boost_danger_tenths: danger here, at caution_danger_horizon, at or above HP x tenths / 10) reads the danger without its fear prices. It can only lower the danger, so the boost fires less: against one white worm mass (32 with fear, 2 without, HP 20, 5 tenths) it stops firing. Measured apart from caution_breeder_danger_damage_only. 0 retains today's play."),
    recover_breeder_hunt_hp_percent => (0, "percent", 0, 100, "borg_recover.rs:1310", "#558 (owner): when a rest is due, HP is at least this percent of max HP and the Borg is fit to hunt breeders now (the hunt applies: caution_breeder_fight on and caution_breeder_hunt_max_count; the Borg is not ignoring the breeders, fleeing or leaving, and no scary monster is on the level; a Mage or Necromancer has half its mana; it is not hungry, confused, blind, hallucinating, afraid or stunned; a known breeder passes the hunt walk's danger test, but not its leash to the stairs or reachability), the Borg does not rest (breeders multiply while it rests) and hunts them first. The HP rest is skipped only when it is due and those hold; the mana rest is skipped whenever they hold. For an acid breeder (a worm mass) the danger test passes only with danger_hunt_breeder_no_acid_fear on; 0 = off (C's rest)."),
    recover_unseen_attack_turns => (0, "game turns", 0, 100000, "borg_recover.rs:812", "#697 (owner): an attack by a monster the Borg cannot see (\"Something/Someone ... you\") counts for this many game turns: no rest within recover_unseen_attack_distance of where it struck; hurt, the Borg walks to a corridor (else any known grid) at least that far and rests there; struck again that far from the last attack (it followed), or nowhere to walk, it leaves the level. 0 = C (the attacker is ignored: no monster, no danger)."),
    recover_unseen_attack_distance => (10, "grids", 1, 100, "borg_recover.rs:815", "#697: the distance from an unseen attack within which the Borg does not rest, the distance it walks away, and the distance at which a later attack means the attacker followed it."),
    recover_unseen_heal_hp_percent => (0, "percent", 0, 100, "borg_recover.rs:953", "#697 (owner): while an unseen attack is recent (recover_unseen_attack_turns), below this share of the maximum HP the Borg drinks Cure Light, Cure Serious, then Cure Critical Wounds. 0 = off."),
    recover_unseen_detect_reads => (0, "reads", 0, 10, "borg_recover.rs:1138", "#831 (owner): in the invisible-attacker mode (recover_unseen_attack_turns), a blow that has just struck makes the Borg read Detect Invisible, quaff True Seeing, use a staff or activation or cast a spell that reveals invisible monsters, at most this many times in the episode; it then fights what it sees. 0 = never."),
    recover_unseen_probe_tries => (0, "attacks", 0, 100, "borg_recover.rs:1106", "#831 (owner): in the invisible-attacker mode, with nothing to detect it with, a blow that has just struck makes the Borg attack (alter) the open adjacent grids in turn, remembering the one where it struck something, at most this many times in the episode; when they are spent and it still strikes, the Borg leaves the level. 0 = never."),
    recover_unseen_probe_max_grids => (3, "grids", 1, 8, "borg_recover.rs:1159", "#831: the blind attacks are made only while at most this many adjacent grids may hold the attacker (a corridor or a doorway, not the open, where it would be guesswork); more, and the Borg walks away or leaves instead."),
    recover_unseen_act_age => (30, "game turns", 0, 1000, "borg_recover.rs:1099", "#831: detection and blind attacks answer a blow this recent (game turns; 10 are a player turn at normal speed): the attacker is next to the Borg now. 0 = never."),
    recover_unseen_shot => (0, "flag", 0, 1, "borg_recover.rs:794", "#981: 1 = \"You are hit by ...!\", the engine's message for any projection (an arrow, a bolt, a breath) that hits the Borg from a monster it cannot see, or any projection while it is blind, is an unseen attack, as a blow is (recover_unseen_attack_turns): no rest near where it struck, Detect Invisible, walk away, below recover_unseen_heal_hp_percent drink a cure. Unlike a blow, a hit is not taken to mean the attacker is adjacent (no blind attack in an episode of hits only, unless blind or without light) or followed the Borg (no leave for it); the episode ends when a known monster that casts or shoots dies. 0 = off (such hits are ignored, as before)."),
    recover_rest_cure_hp_percent => (0, "percent", 0, 100, "borg_recover.rs:985", "#982: when an HP rest is due below this share of the maximum HP (strictly), after the dark-room light, and an awake monster that can move is known within recover_rest_cure_distance grids (in view or not; awake is the last state seen), faster than the Borg by at least recover_rest_cure_speed_edge, the Borg drinks one potion instead of resting: the first it has of Cure Light, Cure Serious and Cure Critical Wounds (out of view the monster's danger is 0, so no other rule drinks). With none, it rests. 0 = off (C: rest)."),
    recover_rest_cure_speed_edge => (1, "speed", 0, 50, "borg_recover.rs:989", "#982: with recover_rest_cure_hp_percent, how much faster than the Borg the monster must be, in the engine's speed: the race's speed plus 10 if it was seen faster, against the Borg's current speed (+10 = twice the turns at normal speed). 1 = any faster monster (the crow of #982 is +10); 10 = at least one speed step; 0 = as fast."),
    recover_rest_cure_distance => (20, "grids", 1, 100, "borg_recover.rs:990", "#982: with recover_rest_cure_hp_percent, how near the awake fast monster must be known (the engine's distance), as recover_rest_monster_distance does for #615 (the crow of #982 was 9 grids away)."),
    recover_rest_breeders => (0, "flag", 0, 1, "borg_recover.rs:692", "#644 (owner): 1 = in the dungeon, while any breeder is known on the level, no rod, HP/SP or mana rest: the Borg hunts the breeders when #558's hunt applies, otherwise leaves the level; kept when confused, blind, hallucinating or stunned. 0 = rest as C."),
    caution_desperate_max_level => (35, "level", 1, 51, "borg_caution.rs:4478", "Controls desperate max level."),
    caution_retreat_danger_divisor => (3, "divisor", 1, 100, "borg_caution.rs:7680", "Controls retreat danger divisor."),
    caution_back_danger_tenths => (4, "tenths", 0, 100, "borg_caution.rs:4913", "Controls back danger tenths."),
    caution_confused_emergency_loss => (300, "hp", 0, 10000, "borg_caution.rs:4606", "Controls confused emergency loss."),
    caution_fear_cure_chance => (70, "percent", 0, 100, "borg_caution.rs:4691", "Controls fear cure chance."),
    caution_fear_berserk_fail => (25, "percent", 0, 100, "borg_caution.rs:4798", "Controls fear berserk fail."),
    caution_fear_hero_fail => (25, "percent", 0, 100, "borg_caution.rs:4799", "Controls fear hero fail."),
    caution_fear_holy_fail => (25, "percent", 0, 100, "borg_caution.rs:4806", "Controls fear holy fail."),
    caution_flee_low_hp_divisor => (3, "divisor", 1, 100, "borg_caution.rs:4814", "Controls flee low hp divisor."),
    caution_flee_moderate_hp_divisor => (2, "divisor", 1, 100, "borg_caution.rs:4815", "Controls flee moderate hp divisor."),
    caution_flee_hp_per_level => (3, "hp_per_level", 0, 100, "borg_caution.rs:4816", "Controls flee hp per level."),
    caution_flee_ccw_count => (3, "count", 0, 1000, "borg_caution.rs:4817", "Controls flee ccw count."),
    caution_flee_heal_count => (1, "count", 0, 1000, "borg_caution.rs:4818", "Controls flee heal count."),
    caution_flee_low_hp_chance => (25, "percent", 0, 100, "borg_caution.rs:4821", "Controls flee low hp chance."),
    caution_flee_status_hp_divisor => (2, "divisor", 1, 100, "borg_caution.rs:4840", "Controls flee status hp divisor."),
    caution_flee_status_chance => (25, "percent", 0, 100, "borg_caution.rs:4843", "Controls flee status chance."),
    caution_reserve_critical_hp_divisor => (10, "divisor", 1, 100, "borg_caution.rs:4981", "Controls reserve critical hp divisor."),
    caution_reserve_escape_count => (2, "count", 0, 1000, "borg_caution.rs:4983", "Controls reserve escape count."),
    caution_reserve_danger_hp_divisor => (4, "divisor", 1, 100, "borg_caution.rs:4984", "Controls reserve danger hp divisor."),
    caution_reserve_heal_count => (5, "count", 0, 1000, "borg_caution.rs:4986", "Controls reserve heal count."),
    caution_reserve_surplus_hp_divisor => (4, "divisor", 1, 100, "borg_caution.rs:4986", "Controls reserve surplus hp divisor."),
    caution_reserve_unique_loss => (600, "hp", 0, 10000, "borg_caution.rs:4988", "Controls reserve unique loss."),
    caution_reserve_unique_depth => (85, "depth", 0, 127, "borg_caution.rs:4990", "Controls reserve unique depth."),
    caution_recall_danger_multiplier => (2, "multiplier", 0, 20, "borg_caution.rs:5027", "Controls recall danger multiplier."),
    caution_recall_wait_as_usual => (0, "flag", 0, 1, "borg_caution.rs:5625", "#603 (owner): 1 = while Word of Recall counts down the Borg plays by its ordinary rules: it flees and leaves by stairs or Teleport Level (the countdown keeps running on the new level), takes stairs down in caution, counts a staircase as an escape, uses the ordinary escape and attack decisions instead of buy_time_for_recall, and handles a scary level as without Recall; only on DL2 and deeper (in town and on DL1 a staircase crosses between town and dungeon, and the Recall would take the Borg back); resting or waiting when safe is kept. 0 = C (the Recall-only rules; the caution_recall_* buy-time settings apply only at 0)."),
    caution_recall_pending_escape => (0, "flag", 0, 1, "borg_caution.rs:5464", "#754 (owner): 1 = a Word of Recall already counting down is an escape that is running: on a scary level the Borg keeps its flight instead of #530's 'no escape: standing and fighting' (#530 counts only a Recall still in the pack); #603's gates still hold the flee flows and down stairs, but an up staircase reached on foot is taken (on DL1: town, where the Recall fires and returns the Borg to its maximum depth). 0 = #530."),
    caution_recall_wait_no_explore => (0, "flag", 0, 1, "borg_caution.rs:5660", "#754 (owner): 1 = while Word of Recall counts down in the dungeon the Borg does not explore or chase far monsters and objects (the 'Explore the dungeon' block of borg_think_dungeon): the think ends there; it rests for Recall where it stands (caution_recall_rest_turns) when borg_check_rest allows it and nothing threatens it there, else stays still one turn. 0 = C (explore while waiting; the wait comes after exploration)."),
    caution_stair_danger_max_level => (49, "level", 1, 51, "borg_caution.rs:7359", "Controls stair danger max level."),
    caution_stair_min_food => (2, "count", 0, 1000, "borg_caution.rs:7364", "Controls stair min food."),
    caution_sell_max_level => (25, "level", 1, 51, "borg_caution.rs:7372", "Controls sell max level."),
    caution_sell_max_gold => (25000, "gold", 0, 1000000, "borg_caution.rs:7373", "Controls sell max gold."),
    caution_sell_min_count => (13, "count", 0, 1000, "borg_caution.rs:7374", "Controls sell min count."),
    caution_recall_mana_maxsp => (60, "sp", 0, 10000, "borg_caution.rs:7544", "Controls recall mana maxsp."),
    caution_recall_mana_divisor => (4, "divisor", 1, 100, "borg_caution.rs:7545", "Controls recall mana divisor."),
    caution_recall_phase_tolerance => (50, "percent", 0, 100, "borg_caution.rs:7558", "Controls recall phase tolerance."),
    caution_recall_phase_horizon => (1, "turns", 1, 100000, "borg_caution.rs:7559", "Controls recall phase horizon."),
    caution_recall_phase_fail => (30, "percent", 0, 100, "borg_caution.rs:7539", "Controls recall phase fail."),
    caution_recall_portal_fail => (30, "percent", 0, 100, "borg_caution.rs:7540", "Controls recall portal fail."),
    caution_recall_cure_loss => (100, "hp", 0, 10000, "borg_caution.rs:7553", "Controls recall cure loss."),
    caution_recall_cure_min_loss_percent => (0, "percent of max HP", 0, 100, "borg_caution.rs:7554", "#749: waiting for Recall in danger, C buys time with a Cure Light or Serious Wounds while less than caution_recall_cure_loss (100) HP are lost, which at low level includes full HP; above 0, only with at least this percent of the max HP lost. 0 = C."),
    caution_recall_heal_loss => (150, "hp", 0, 10000, "borg_caution.rs:7563", "Controls recall heal loss."),
    caution_retreat_danger_horizon => (1, "turns", 1, 10, "borg_caution.rs:1238", "Number of Borg turns of danger evaluated along a retreat."),
    caution_back_danger_horizon => (1, "turns", 1, 10, "borg_caution.rs:1683", "Number of Borg turns of danger evaluated for a back-away grid."),
    caution_danger_horizon => (1, "turns", 1, 10, "borg_caution.rs:4391", "Number of Borg turns of danger evaluated at the current grid."),
    caution_recall_rest_turns => (500, "turns", 2, 9999, "borg_caution.rs:4556", "Number of rest turns queued while waiting safely for recall."),
    caution_recall_rest_monster_turns => (0, "game turns", 0, 100000, "borg_caution.rs:7445", "#567: while Word of Recall is active, no long rest (Resting for Recall, Waiting for Recall, or the wait with no light) if an awake monster was known within caution_recall_rest_monster_distance in the last this many game turns; the Borg stays still one turn at a time instead (','), so each think sees a returning monster. 0 = off (C)."),
    caution_recall_rest_monster_distance => (20, "grids", 1, 100, "borg_caution.rs:7448", "#567: with caution_recall_rest_monster_turns, how near an awake monster must be (the engine's distance) to hold the Recall rest."),
    caution_unseen_bump_turns => (0, "game turns", 0, 100000, "borg_caution.rs:9193", "#781: a queued step with unseen melee feedback and no movement marks its destination occupied for this many game turns. All flows refuse it. 0 = off."),
    caution_unseen_bump_response => (0, "flag", 0, 1, "borg_caution.rs:9318", "#781: after ordinary healing, defence, Recall and food, before stair walking, try one carried Detect Invisible scroll under the existing detection danger/darkness gates, bounded directional oil throws with normal fuel reserves, then Phase Door under the emergency landing test. Requires caution_unseen_bump_turns. Counts successful phases toward level escape accounting. 0 = off."),
    caution_unseen_bump_heal_hp_percent => (0, "percent", 0, 100, "borg_caution.rs:9310", "#781: below this HP percentage with a recent adjacent unseen bump, drink CLW, CSW, then CCW after ordinary healing/defence/Recall but before stair walking, even while fleeing. Intentionally ignores danger, including survivable-healing and no-rope-a-doping gates: regional fear does not quantify the unseen blow. Requires caution_unseen_bump_turns; independent of response. 0 = off."),
    caution_unseen_bump_throw_tries => (3, "throws per grid", 0, 100, "borg_caution.rs:9304", "#781: maximum queued oil attempts at one remembered bump grid before trying Phase Door. Cancels also spend an attempt; only one command is queued per response. Expiry or a confirmed kill clears that grid and its budget. 0 = phase after detection; hypothesis, not calibrated."),
    caution_recall_dl1_hold_up => (0, "flag", 0, 1, "borg_caution.rs:9648", "#984: 1 = while a Word of Recall counts down on DL1, borg_caution does not take the up staircase (its 'Take stairs up'), unless the danger is above the avoidance (the stairs rule's own boundary, pos_danger > avoidance): the staircase leads to town, where the pending Recall fires after a few world ticks and takes the Borg back to its maximum depth before it can shop (312081895: 42 game turns in town, then DL2 with no Recall). Held, the Recall fires on DL1 and the Borg reaches town with nothing pending. The danger escapes (borg_escape_stair) are not held. The up half of the town/DL1 crossing: recall_restricts (#603, #627) already holds the down half. Arena levels have no staircase (arena_gen), so the hold never applies there. 0 = C (borg-caution.c:1171)."),
    // #457: outmatched healing, a study flag. 0 = the vendored behaviour exactly.
    caution_combat_heal => (0, "flag", 0, 1, "borg_combat_heal.rs:70", "#1194: choose ordinary wound potions against the next monster reply, including during Recall, and try a survivable potion before an unchecked critical Phase Door or a combat action. A poison-only cure also yields when that reply could be lethal. 0 retains today's play."),
    caution_combat_heal_min_loss => (20, "percent", 0, 100, "borg_combat_heal.rs:108", "#1194: hypothesis for sustained combat with caution_combat_heal on. Minimum percent HP lost before drinking when the projected reply is survivable at current HP."),
    caution_heal_when_outmatched => (0, "flag", 0, 1, "borg_caution.rs:229", "1 = after every escape declined, a hurt Borg whose danger reaches its HP drinks CLW/CSW/CCW instead of attacking; 0 = off."),
    caution_heal_outmatched_min_loss => (50, "percent", 0, 100, "borg_caution.rs:233", "Minimum percent of maximum HP lost before outmatched healing applies (only when caution_heal_when_outmatched is 1)."),
    // borg-reborn-rs (#220): source lines in danger/escape/recovery at 30ab9d66.
    danger_monblow_hurt_fear => (400, "danger", 0, 5000, "borg_danger.rs:208", "Additional melee danger for MONBLOW_HURT (n += 400;)."),
    danger_monblow_hurt_hit_power => (60, "hit_power", 0, 1000, "borg_danger.rs:210", "Legacy hit-power estimate used for MONBLOW_HURT in partial melee danger."),
    danger_monblow_poison_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:218", "Legacy hit-power estimate used for MONBLOW_POISON in partial melee danger."),
    danger_monblow_poison_fear => (10, "danger", 0, 5000, "borg_danger.rs:226", "Additional melee danger for MONBLOW_POISON (z += 10;)."),
    danger_monblow_disenchant_hit_power => (20, "hit_power", 0, 1000, "borg_danger.rs:234", "Legacy hit-power estimate used for MONBLOW_DISENCHANT in partial melee danger."),
    danger_monblow_disenchant_fear => (500, "danger", 0, 5000, "borg_danger.rs:239", "Additional melee danger for MONBLOW_DISENCHANT (z += 500;)."),
    danger_monblow_drain_charges_fear => (20, "danger", 0, 5000, "borg_danger.rs:248", "Additional melee danger for MONBLOW_DRAIN_CHARGES (z += 20;)."),
    danger_monblow_drain_charges_hit_power => (15, "hit_power", 0, 1000, "borg_danger.rs:249", "Legacy hit-power estimate used for MONBLOW_DRAIN_CHARGES in partial melee danger."),
    danger_monblow_eat_gold_fear => (50, "danger", 0, 5000, "borg_danger.rs:259", "Additional melee danger for MONBLOW_EAT_GOLD (z += 50;)."),
    danger_monblow_eat_gold_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:261", "Legacy hit-power estimate used for MONBLOW_EAT_GOLD in partial melee danger."),
    danger_monblow_eat_gold_fear_2 => (5, "danger", 0, 5000, "borg_danger.rs:274", "Additional melee danger for MONBLOW_EAT_GOLD (z += 5;)."),
    danger_monblow_eat_item_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:282", "Legacy hit-power estimate used for MONBLOW_EAT_ITEM in partial melee danger."),
    danger_monblow_eat_item_fear => (5, "danger", 0, 5000, "borg_danger.rs:289", "Additional melee danger for MONBLOW_EAT_ITEM (z += 5;)."),
    danger_monblow_eat_food_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:297", "Legacy hit-power estimate used for MONBLOW_EAT_FOOD in partial melee danger."),
    danger_monblow_eat_food_fear => (5, "danger", 0, 5000, "borg_danger.rs:302", "Additional melee danger for MONBLOW_EAT_FOOD (z += 5;)."),
    danger_monblow_eat_light_hit_power => (5, "hit_power", 0, 1000, "borg_danger.rs:310", "Legacy hit-power estimate used for MONBLOW_EAT_LIGHT in partial melee danger."),
    danger_monblow_eat_light_fear => (5, "danger", 0, 5000, "borg_danger.rs:319", "Additional melee danger for MONBLOW_EAT_LIGHT (z += 5;)."),
    danger_monblow_acid_fear => (200, "danger", 0, 5000, "borg_danger.rs:359", "Additional melee danger for MONBLOW_ACID (z += 200; // We don't want our armour corroded.)."),
    danger_hunt_breeder_no_acid_fear => (0, "flag", 0, 1, "borg_danger.rs:154", "#558 (owner): 1 = while the Borg hunts the level's breeders (the hunt applies: caution_breeder_fight on and caution_breeder_hunt_max_count; it is not ignoring them, fleeing or leaving, and no scary monster is on the level), every breeder on the level is judged on its bite, without danger_monblow_acid_fear (a green worm mass's 1d3 acid bite read 630 danger at CL6). The condition is level-wide, not per monster, so the regional fear map and retreat grids see it too; 0 = off."),
    danger_monblow_elec_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:348", "Legacy hit-power estimate used for MONBLOW_ELEC in partial melee danger."),
    danger_monblow_elec_damage_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:356", "Multiplies damage to account for MONBLOW_ELEC side effects."),
    danger_monblow_fire_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:367", "Legacy hit-power estimate used for MONBLOW_FIRE in partial melee danger."),
    danger_monblow_fire_damage_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:375", "Multiplies damage to account for MONBLOW_FIRE side effects."),
    danger_monblow_cold_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:386", "Legacy hit-power estimate used for MONBLOW_COLD in partial melee danger."),
    danger_monblow_cold_damage_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:394", "Multiplies damage to account for MONBLOW_COLD side effects."),
    danger_monblow_blind_hit_power => (2, "hit_power", 0, 1000, "borg_danger.rs:402", "Legacy hit-power estimate used for MONBLOW_BLIND in partial melee danger."),
    danger_monblow_blind_fear => (10, "danger", 0, 5000, "borg_danger.rs:407", "Additional melee danger for MONBLOW_BLIND (z += 10;)."),
    danger_monblow_blind_fear_2 => (75, "danger", 0, 5000, "borg_danger.rs:409", "Additional melee danger for MONBLOW_BLIND (z += 75;)."),
    danger_monblow_confuse_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:418", "Legacy hit-power estimate used for MONBLOW_CONFUSE in partial melee danger."),
    danger_monblow_confuse_fear => (200, "danger", 0, 5000, "borg_danger.rs:423", "Additional melee danger for MONBLOW_CONFUSE (z += 200;)."),
    danger_monblow_confuse_fear_2 => (200, "danger", 0, 5000, "borg_danger.rs:425", "Additional melee danger for MONBLOW_CONFUSE (z += 200;)."),
    danger_monblow_terrify_hit_power => (10, "hit_power", 0, 1000, "borg_danger.rs:434", "Legacy hit-power estimate used for MONBLOW_TERRIFY in partial melee danger."),
    danger_monblow_terrify_damage_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:439", "Multiplies damage to account for MONBLOW_TERRIFY side effects."),
    danger_monblow_paralyze_hit_power => (2, "hit_power", 0, 1000, "borg_danger.rs:447", "Legacy hit-power estimate used for MONBLOW_PARALYZE in partial melee danger."),
    danger_monblow_paralyze_fear => (200, "danger", 0, 5000, "borg_danger.rs:451", "Additional melee danger for MONBLOW_PARALYZE (z += 200;)."),
    danger_monblow_lose_str_fear => (150, "danger", 0, 5000, "borg_danger.rs:474", "Additional melee danger for MONBLOW_LOSE_STR (z += 150;)."),
    danger_monblow_lose_str_fear_2 => (100, "danger", 0, 5000, "borg_danger.rs:477", "Additional melee danger for MONBLOW_LOSE_STR (z += 100;)."),
    danger_monblow_lose_dex_fear => (150, "danger", 0, 5000, "borg_danger.rs:498", "Additional melee danger for MONBLOW_LOSE_DEX (z += 150;)."),
    danger_monblow_lose_dex_fear_2 => (100, "danger", 0, 5000, "borg_danger.rs:501", "Additional melee danger for MONBLOW_LOSE_DEX (z += 100;)."),
    danger_monblow_lose_con_fear => (150, "danger", 0, 5000, "borg_danger.rs:526", "Additional melee danger for MONBLOW_LOSE_CON (z += 150;)."),
    danger_monblow_lose_con_fear_2 => (100, "danger", 0, 5000, "borg_danger.rs:529", "Additional melee danger for MONBLOW_LOSE_CON (z += 100;)."),
    danger_monblow_lose_int_fear => (150, "danger", 0, 5000, "borg_danger.rs:553", "Additional melee danger for MONBLOW_LOSE_INT (z += 150;)."),
    danger_monblow_lose_int_fear_2 => (50, "danger", 0, 5000, "borg_danger.rs:556", "Additional melee danger for MONBLOW_LOSE_INT (z += 50;)."),
    danger_monblow_lose_wis_fear => (150, "danger", 0, 5000, "borg_danger.rs:577", "Additional melee danger for MONBLOW_LOSE_WIS (z += 150;)."),
    danger_monblow_lose_wis_fear_2 => (50, "danger", 0, 5000, "borg_danger.rs:580", "Additional melee danger for MONBLOW_LOSE_WIS (z += 50;)."),
    danger_monblow_lose_all_hit_power => (2, "hit_power", 0, 1000, "borg_danger.rs:589", "Legacy hit-power estimate used for MONBLOW_LOSE_ALL in partial melee danger."),
    danger_monblow_shatter_hit_power => (60, "hit_power", 0, 1000, "borg_danger.rs:596", "Legacy hit-power estimate used for MONBLOW_SHATTER in partial melee danger."),
    danger_monblow_shatter_fear => (150, "danger", 0, 5000, "borg_danger.rs:598", "Additional melee danger for MONBLOW_SHATTER (z += 150;)."),
    danger_monblow_hallu_fear => (250, "danger", 0, 5000, "borg_danger.rs:636", "Additional melee danger for MONBLOW_HALLU (z += 250;)."),
    danger_rsf_shriek_fear => (5, "danger", 0, 5000, "borg_danger.rs:795", "Additional spell danger for RSF_SHRIEK (p += 5;)."),
    danger_rsf_br_acid_fear => (40, "danger", 0, 5000, "borg_danger.rs:838", "Additional spell danger for RSF_BR_ACID (p += 40;)."),
    danger_rsf_br_elec_fear => (20, "danger", 0, 5000, "borg_danger.rs:857", "Additional spell danger for RSF_BR_ELEC (p += 20;)."),
    danger_rsf_br_fire_fear => (40, "danger", 0, 5000, "borg_danger.rs:876", "Additional spell danger for RSF_BR_FIRE (p += 40;)."),
    danger_rsf_br_cold_fear => (20, "danger", 0, 5000, "borg_danger.rs:895", "Additional spell danger for RSF_BR_COLD (p += 20;)."),
    danger_rsf_br_pois_fear => (20, "danger", 0, 5000, "borg_danger.rs:917", "Additional spell danger for RSF_BR_POIS (p += 20;)."),
    danger_rsf_br_neth_fear => (125, "danger", 0, 5000, "borg_danger.rs:931", "Additional spell danger for RSF_BR_NETH (p += 125;)."),
    danger_rsf_br_light_fear => (20, "danger", 0, 5000, "borg_danger.rs:947", "Additional spell danger for RSF_BR_LIGHT (p += 20;)."),
    danger_rsf_br_light_fear_2 => (20, "danger", 0, 5000, "borg_danger.rs:949", "Additional spell danger for RSF_BR_LIGHT (p += 20;)."),
    danger_rsf_br_dark_fear => (20, "danger", 0, 5000, "borg_danger.rs:968", "Additional spell danger for RSF_BR_DARK (p += 20;)."),
    danger_rsf_br_dark_fear_2 => (20, "danger", 0, 5000, "borg_danger.rs:970", "Additional spell danger for RSF_BR_DARK (p += 20;)."),
    danger_rsf_br_soun_fear => (500, "danger", 0, 5000, "borg_danger.rs:988", "Additional spell danger for RSF_BR_SOUN (z += 500;)."),
    danger_rsf_br_soun_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:991", "Additional spell danger for RSF_BR_SOUN (z += 1000;)."),
    danger_rsf_br_soun_fear_3 => (50, "danger", 0, 5000, "borg_danger.rs:994", "Additional spell danger for RSF_BR_SOUN (p += 50;)."),
    danger_rsf_br_chao_fear => (100, "danger", 0, 5000, "borg_danger.rs:1007", "Additional spell danger for RSF_BR_CHAO (p += 100;)."),
    danger_rsf_br_chao_fear_2 => (200, "danger", 0, 5000, "borg_danger.rs:1011", "Additional spell danger for RSF_BR_CHAO (p += 200;)."),
    danger_rsf_br_dise_fear => (500, "danger", 0, 5000, "borg_danger.rs:1026", "Additional spell danger for RSF_BR_DISE (p += 500;)."),
    danger_rsf_br_nexu_fear => (100, "danger", 0, 5000, "borg_danger.rs:1042", "Additional spell danger for RSF_BR_NEXU (p += 100;)."),
    danger_rsf_br_time_fear => (250, "danger", 0, 5000, "borg_danger.rs:1052", "Additional spell danger for RSF_BR_TIME (p += 250;)."),
    danger_rsf_br_iner_fear => (100, "danger", 0, 5000, "borg_danger.rs:1062", "Additional spell danger for RSF_BR_INER (p += 100;)."),
    danger_rsf_br_grav_fear => (100, "danger", 0, 5000, "borg_danger.rs:1072", "Additional spell danger for RSF_BR_GRAV (p += 100;)."),
    danger_rsf_br_grav_fear_2 => (500, "danger", 0, 5000, "borg_danger.rs:1078", "Additional spell danger for RSF_BR_GRAV (z += 500;)."),
    danger_rsf_br_grav_fear_3 => (1000, "danger", 0, 5000, "borg_danger.rs:1081", "Additional spell danger for RSF_BR_GRAV (z += 1000;)."),
    danger_rsf_br_shar_fear => (50, "danger", 0, 5000, "borg_danger.rs:1098", "Additional spell danger for RSF_BR_SHAR (p += 50;)."),
    danger_rsf_br_plas_fear => (100, "danger", 0, 5000, "borg_danger.rs:1112", "Additional spell danger for RSF_BR_PLAS (p += 100;)."),
    danger_rsf_br_plas_fear_2 => (500, "danger", 0, 5000, "borg_danger.rs:1115", "Additional spell danger for RSF_BR_PLAS (z += 500;)."),
    danger_rsf_br_plas_fear_3 => (1000, "danger", 0, 5000, "borg_danger.rs:1118", "Additional spell danger for RSF_BR_PLAS (z += 1000;)."),
    danger_rsf_br_wall_fear => (100, "danger", 0, 5000, "borg_danger.rs:1133", "Additional spell danger for RSF_BR_WALL (z += 100;)."),
    danger_rsf_br_wall_fear_2 => (500, "danger", 0, 5000, "borg_danger.rs:1136", "Additional spell danger for RSF_BR_WALL (z += 500;)."),
    danger_rsf_br_wall_fear_3 => (50, "danger", 0, 5000, "borg_danger.rs:1139", "Additional spell danger for RSF_BR_WALL (p += 50;)."),
    danger_rsf_ba_acid_fear => (40, "danger", 0, 5000, "borg_danger.rs:1172", "Additional spell danger for RSF_BA_ACID (p += 40;)."),
    danger_rsf_ba_elec_fear => (20, "danger", 0, 5000, "borg_danger.rs:1187", "Additional spell danger for RSF_BA_ELEC (p += 20;)."),
    danger_rsf_ba_fire_fear => (40, "danger", 0, 5000, "borg_danger.rs:1202", "Additional spell danger for RSF_BA_FIRE (p += 40;)."),
    danger_rsf_ba_cold_fear => (20, "danger", 0, 5000, "borg_danger.rs:1217", "Additional spell danger for RSF_BA_COLD (p += 20;)."),
    danger_rsf_ba_pois_fear => (20, "danger", 0, 5000, "borg_danger.rs:1235", "Additional spell danger for RSF_BA_POIS (p += 20;)."),
    danger_rsf_ba_shar_fear => (20, "danger", 0, 5000, "borg_danger.rs:1247", "Additional spell danger for RSF_BA_SHAR (p += 20;)."),
    danger_rsf_ba_neth_fear => (250, "danger", 0, 5000, "borg_danger.rs:1259", "Additional spell danger for RSF_BA_NETH (p += 250;)."),
    danger_rsf_ba_wate_fear => (500, "danger", 0, 5000, "borg_danger.rs:1269", "Additional spell danger for RSF_BA_WATE (p += 500;)."),
    danger_rsf_ba_wate_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:1272", "Additional spell danger for RSF_BA_WATE (p += 1000;)."),
    danger_rsf_ba_wate_fear_3 => (50, "danger", 0, 5000, "borg_danger.rs:1278", "Additional spell danger for RSF_BA_WATE (p += 50;)."),
    danger_rsf_ba_wate_fear_4 => (20, "danger", 0, 5000, "borg_danger.rs:1280", "Additional spell danger for RSF_BA_WATE (p += 20;)."),
    danger_rsf_ba_mana_fear => (50, "danger", 0, 5000, "borg_danger.rs:1287", "Additional spell danger for RSF_BA_MANA (p += 50;)."),
    danger_rsf_ba_holy_fear => (50, "danger", 0, 5000, "borg_danger.rs:1293", "Additional spell danger for RSF_BA_HOLY (p += 50;)."),
    danger_rsf_ba_dark_fear => (20, "danger", 0, 5000, "borg_danger.rs:1308", "Additional spell danger for RSF_BA_DARK (p += 20;)."),
    danger_rsf_ba_dark_fear_2 => (20, "danger", 0, 5000, "borg_danger.rs:1310", "Additional spell danger for RSF_BA_DARK (p += 20;)."),
    danger_rsf_ba_light_fear => (20, "danger", 0, 5000, "borg_danger.rs:1326", "Additional spell danger for RSF_BA_LIGHT (p += 20;)."),
    danger_rsf_ba_light_fear_2 => (20, "danger", 0, 5000, "borg_danger.rs:1328", "Additional spell danger for RSF_BA_LIGHT (p += 20;)."),
    danger_rsf_storm_fear => (500, "danger", 0, 5000, "borg_danger.rs:1339", "Additional spell danger for RSF_STORM (p += 500;)."),
    danger_rsf_storm_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:1342", "Additional spell danger for RSF_STORM (p += 1000;)."),
    danger_rsf_drain_mana_fear => (100, "danger", 0, 5000, "borg_danger.rs:1350", "Additional spell danger for RSF_DRAIN_MANA (p += 100;)."),
    danger_rsf_brain_smash_fear => (200, "danger", 0, 5000, "borg_danger.rs:1362", "Additional spell danger for RSF_BRAIN_SMASH (p += 200 - 2 * st.borg.tr(BI_SAV);)."),
    danger_rsf_bo_acid_fear => (40, "danger", 0, 5000, "borg_danger.rs:1390", "Additional spell danger for RSF_BO_ACID (p += 40;)."),
    danger_rsf_bo_elec_fear => (20, "danger", 0, 5000, "borg_danger.rs:1406", "Additional spell danger for RSF_BO_ELEC (p += 20;)."),
    danger_rsf_bo_fire_fear => (40, "danger", 0, 5000, "borg_danger.rs:1422", "Additional spell danger for RSF_BO_FIRE (p += 40;)."),
    danger_rsf_bo_cold_fear => (20, "danger", 0, 5000, "borg_danger.rs:1438", "Additional spell danger for RSF_BO_COLD (p += 20;)."),
    danger_rsf_bo_neth_fear => (200, "danger", 0, 5000, "borg_danger.rs:1465", "Additional spell danger for RSF_BO_NETH (p += 200;)."),
    danger_rsf_bo_wate_fear => (500, "danger", 0, 5000, "borg_danger.rs:1476", "Additional spell danger for RSF_BO_WATE (p += 500;)."),
    danger_rsf_bo_wate_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:1479", "Additional spell danger for RSF_BO_WATE (p += 1000;)."),
    danger_rsf_bo_wate_fear_3 => (20, "danger", 0, 5000, "borg_danger.rs:1485", "Additional spell danger for RSF_BO_WATE (p += 20;)."),
    danger_rsf_bo_wate_fear_4 => (20, "danger", 0, 5000, "borg_danger.rs:1487", "Additional spell danger for RSF_BO_WATE (p += 20;)."),
    danger_rsf_bo_mana_fear => (50, "danger", 0, 5000, "borg_danger.rs:1495", "Additional spell danger for RSF_BO_MANA (p += 50;)."),
    danger_rsf_bo_plas_fear => (500, "danger", 0, 5000, "borg_danger.rs:1506", "Additional spell danger for RSF_BO_PLAS (z += 500;)."),
    danger_rsf_bo_plas_fear_2 => (1000, "danger", 0, 5000, "borg_danger.rs:1509", "Additional spell danger for RSF_BO_PLAS (z += 1000;)."),
    danger_rsf_bo_ice_fear => (20, "danger", 0, 5000, "borg_danger.rs:1517", "Additional spell danger for RSF_BO_ICE (p += 20;)."),
    danger_rsf_bo_ice_fear_2 => (50, "danger", 0, 5000, "borg_danger.rs:1523", "Additional spell danger for RSF_BO_ICE (z += 50;)."),
    danger_rsf_bo_ice_fear_3 => (1000, "danger", 0, 5000, "borg_danger.rs:1526", "Additional spell danger for RSF_BO_ICE (z += 1000;)."),
    danger_rsf_scare_fear => (10, "danger", 0, 5000, "borg_danger.rs:1560", "Additional spell danger for RSF_SCARE (p += 10;)."),
    danger_rsf_blind_fear => (10, "danger", 0, 5000, "borg_danger.rs:1572", "Additional spell danger for RSF_BLIND (p += 10;)."),
    danger_rsf_conf_fear => (10, "danger", 0, 5000, "borg_danger.rs:1583", "Additional spell danger for RSF_CONF (p += 10;)."),
    danger_rsf_slow_fear => (5, "danger", 0, 5000, "borg_danger.rs:1594", "Additional spell danger for RSF_SLOW (p += 5;)."),
    danger_rsf_hold_fear => (150, "danger", 0, 5000, "borg_danger.rs:1604", "Additional spell danger for RSF_HOLD (p += 150;)."),
    danger_rsf_haste_fear => (10, "danger", 0, 5000, "borg_danger.rs:1609", "Additional spell danger for RSF_HASTE (p += 10;)."),
    danger_rsf_heal_fear => (10, "danger", 0, 5000, "borg_danger.rs:1614", "Additional spell danger for RSF_HEAL (p += 10;)."),
    danger_rsf_tport_fear => (10, "danger", 0, 5000, "borg_danger.rs:1623", "Additional spell danger for RSF_TPORT (p += 10;)."),
    danger_rsf_tele_to_fear => (20, "danger", 0, 5000, "borg_danger.rs:1628", "Additional spell danger for RSF_TELE_TO (p += 20;)."),
    danger_rsf_tele_self_to_fear => (20, "danger", 0, 5000, "borg_danger.rs:1633", "Additional spell danger for RSF_TELE_SELF_TO (p += 20;)."),
    danger_rsf_tele_away_fear => (10, "danger", 0, 5000, "borg_danger.rs:1638", "Additional spell danger for RSF_TELE_AWAY (p += 10;)."),
    danger_rsf_tele_level_fear => (50, "danger", 0, 5000, "borg_danger.rs:1644", "Additional spell danger for RSF_TELE_LEVEL (p += 50;)."),
    danger_rsf_darkness_fear => (5, "danger", 0, 5000, "borg_danger.rs:1650", "Additional spell danger for RSF_DARKNESS (p += 5;)."),
    danger_rsf_traps_fear => (50, "danger", 0, 5000, "borg_danger.rs:1655", "Additional spell danger for RSF_TRAPS (p += 50;)."),
    danger_rsf_forget_fear => (500, "danger", 0, 5000, "borg_danger.rs:1662", "Additional spell danger for RSF_FORGET (p += 500;)."),
    danger_rsf_forget_fear_2 => (30, "danger", 0, 5000, "borg_danger.rs:1664", "Additional spell danger for RSF_FORGET (p += 30;)."),
    danger_rsf_shapechange_fear => (200, "danger", 0, 5000, "borg_danger.rs:1671", "Additional spell danger for RSF_SHAPECHANGE (p += 200;)."),
    danger_rsf_s_kin_power_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:1688", "Spell-power risk multiplier in RSF_S_KIN, branch 1."),
    danger_rsf_s_kin_power_multiplier_2 => (7, "multiplier", 0, 50, "borg_danger.rs:1691", "Spell-power risk multiplier in RSF_S_KIN, branch 2."),
    danger_summon_unique_percent => (75, "percent", 0, 100, "borg_danger.rs:1696", "Percentage of summon fear retained when the summoner is a unique."),
    danger_rsf_s_hi_demon_power_multiplier => (6, "multiplier", 0, 50, "borg_danger.rs:1706", "Spell-power risk multiplier in RSF_S_HI_DEMON, branch 1."),
    danger_rsf_s_hi_demon_power_multiplier_2 => (12, "multiplier", 0, 50, "borg_danger.rs:1709", "Spell-power risk multiplier in RSF_S_HI_DEMON, branch 2."),
    danger_rsf_s_monster_power_multiplier => (5, "multiplier", 0, 50, "borg_danger.rs:1723", "Spell-power risk multiplier in RSF_S_MONSTER, branch 1."),
    danger_rsf_s_monsters_power_multiplier => (7, "multiplier", 0, 50, "borg_danger.rs:1733", "Spell-power risk multiplier in RSF_S_MONSTERS, branch 1."),
    danger_rsf_s_animal_power_multiplier => (5, "multiplier", 0, 50, "borg_danger.rs:1747", "Spell-power risk multiplier in RSF_S_ANIMAL, branch 1."),
    danger_rsf_s_hydra_power_multiplier => (2, "multiplier", 0, 50, "borg_danger.rs:1762", "Spell-power risk multiplier in RSF_S_HYDRA, branch 1."),
    danger_rsf_s_hydra_power_multiplier_2 => (5, "multiplier", 0, 50, "borg_danger.rs:1765", "Spell-power risk multiplier in RSF_S_HYDRA, branch 2."),
    danger_rsf_s_ainu_power_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:1780", "Spell-power risk multiplier in RSF_S_AINU, branch 1."),
    danger_rsf_s_ainu_power_multiplier_2 => (7, "multiplier", 0, 50, "borg_danger.rs:1783", "Spell-power risk multiplier in RSF_S_AINU, branch 2."),
    danger_rsf_s_demon_power_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:1798", "Spell-power risk multiplier in RSF_S_DEMON, branch 1."),
    danger_rsf_s_demon_power_multiplier_2 => (7, "multiplier", 0, 50, "borg_danger.rs:1801", "Spell-power risk multiplier in RSF_S_DEMON, branch 2."),
    danger_rsf_s_hi_undead_power_multiplier => (6, "multiplier", 0, 50, "borg_danger.rs:1816", "Spell-power risk multiplier in RSF_S_HI_UNDEAD, branch 1."),
    danger_rsf_s_hi_undead_power_multiplier_2 => (12, "multiplier", 0, 50, "borg_danger.rs:1819", "Spell-power risk multiplier in RSF_S_HI_UNDEAD, branch 2."),
    danger_rsf_s_hi_dragon_power_multiplier => (6, "multiplier", 0, 50, "borg_danger.rs:1834", "Spell-power risk multiplier in RSF_S_HI_DRAGON, branch 1."),
    danger_rsf_s_hi_dragon_power_multiplier_2 => (12, "multiplier", 0, 50, "borg_danger.rs:1837", "Spell-power risk multiplier in RSF_S_HI_DRAGON, branch 2."),
    danger_rsf_s_unique_power_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:1852", "Spell-power risk multiplier in RSF_S_UNIQUE, branch 1."),
    danger_rsf_s_unique_power_multiplier_2 => (6, "multiplier", 0, 50, "borg_danger.rs:1855", "Spell-power risk multiplier in RSF_S_UNIQUE, branch 2."),
    danger_unknown_physical => (1000, "danger", 0, 5000, "borg_danger.rs:175", "Sets unknown physical, measured in danger, in the existing danger rule."),
    danger_knockout_max_sides => (3, "count", 0, 1000, "borg_danger.rs:207", "Sets knockout max sides, measured in count, in the existing danger rule."),
    danger_knockout_min_dice => (5, "count", 0, 1000, "borg_danger.rs:207", "Sets knockout min dice, measured in count, in the existing danger rule."),
    danger_gold_theft_low_level => (5, "level", 1, 51, "borg_danger.rs:258", "Sets gold theft low level, measured in level, in the existing danger rule."),
    danger_gold_theft_min_gold => (100, "gold", 0, 1000000, "borg_danger.rs:267", "Sets gold theft min gold, measured in gold, in the existing danger rule."),
    danger_gold_theft_max_gold => (100000, "gold", 0, 1000000, "borg_danger.rs:270", "Sets gold theft max gold, measured in gold, in the existing danger rule."),
    danger_food_theft_reserve => (5, "count", 0, 1000, "borg_danger.rs:298", "Sets food theft reserve, measured in count, in the existing danger rule."),
    danger_fuel_theft_reserve => (5, "count", 0, 1000, "borg_danger.rs:315", "Sets fuel theft reserve, measured in count, in the existing danger rule."),
    danger_str_drain_low_stat => (10, "stat", 3, 238, "borg_danger.rs:476", "Sets str drain low stat, measured in stat, in the existing danger rule."),
    danger_dex_drain_low_stat => (10, "stat", 3, 238, "borg_danger.rs:500", "Sets dex drain low stat, measured in stat, in the existing danger rule."),
    danger_con_drain_low_stat => (8, "stat", 3, 238, "borg_danger.rs:528", "Sets con drain low stat, measured in stat, in the existing danger rule."),
    danger_shatter_ac_cap => (150, "ac", 0, 1000, "borg_danger.rs:595", "Sets shatter ac cap, measured in ac, in the existing danger rule."),
    danger_shatter_ac_divisor => (250, "divisor", 1, 1000, "borg_danger.rs:595", "Sets shatter ac divisor, measured in divisor, in the existing danger rule."),
    danger_exp10_fear => (100, "danger", 0, 5000, "borg_danger.rs:608", "Sets exp10 fear, measured in danger, in the existing danger rule."),
    danger_exp_ignore_level => (50, "level", 1, 51, "borg_danger.rs:608", "Sets exp ignore level, measured in level, in the existing danger rule."),
    danger_exp20_fear => (150, "danger", 0, 5000, "borg_danger.rs:609", "Sets exp20 fear, measured in danger, in the existing danger rule."),
    danger_exp40_fear => (200, "danger", 0, 5000, "borg_danger.rs:610", "Sets exp40 fear, measured in danger, in the existing danger rule."),
    danger_exp80_fear => (250, "danger", 0, 5000, "borg_danger.rs:611", "Sets exp80 fear, measured in danger, in the existing danger rule."),
    danger_melee_hit_luck_percent => (150, "percent", 100, 300, "borg_danger.rs:657", "Sets melee hit luck percent, measured in percent, in the existing danger rule."),
    danger_melee_hit_ac_scale => (300, "coefficient", 0, 1000, "borg_danger.rs:657", "Sets melee hit ac scale, measured in coefficient, in the existing danger rule."),
    danger_melee_hit_ac_divisor => (4, "divisor", 1, 1000, "borg_danger.rs:657", "Sets melee hit ac divisor, measured in divisor, in the existing danger rule."),
    danger_melee_hit_power_scale => (3, "denominator multiplier", 1, 1000, "borg_danger.rs:657", "Positive multiplier inside the melee hit-chance denominator; zero would divide by zero."),
    danger_melee_hit_floor_percent => (5, "percent", 0, 100, "borg_danger.rs:663", "Sets melee hit floor percent, measured in percent, in the existing danger rule."),
    danger_summon_protected_divisor => (100, "divisor", 1, 1000, "borg_danger.rs:739", "Sets summon protected divisor, measured in divisor, in the existing danger rule."),
    danger_summon_corridor_divisor => (1000, "divisor", 1, 10000, "borg_danger.rs:742", "Sets summon corridor divisor, measured in divisor, in the existing danger rule."),
    danger_unknown_spell => (1000, "danger", 0, 5000, "borg_danger.rs:766", "Sets unknown spell, measured in danger, in the existing danger rule."),
    danger_brain_smash_save_weight => (2, "multiplier", 0, 50, "borg_danger.rs:1362", "Sets brain smash save weight, measured in multiplier, in the existing danger rule."),
    danger_cause4_save_bias => (120, "save_points", 0, 1000, "borg_danger.rs:1374", "Sets cause4 save bias, measured in save_points, in the existing danger rule."),
    danger_forget_low_sp => (15, "sp", 0, 10000, "borg_danger.rs:1661", "Sets forget low sp, measured in sp, in the existing danger rule."),
    danger_ignore_side_effects_depth => (75, "depth", 0, 127, "borg_danger.rs:1876", "Sets ignore side effects depth, measured in depth, in the existing danger rule."),
    danger_corridor_spell_tenths => (7, "tenths", 0, 30, "borg_danger.rs:1901", "Sets corridor spell tenths, measured in tenths, in the existing danger rule."),
    danger_worst_spell_average_tenths => (15, "tenths", 0, 30, "borg_danger.rs:1948", "Sets worst spell average tenths, measured in tenths, in the existing danger rule."),
    danger_worst_spell_hp_tenths => (8, "tenths", 0, 30, "borg_danger.rs:1948", "Sets worst spell hp tenths, measured in tenths, in the existing danger rule."),
    danger_sleep_level_margin => (15, "level", 1, 51, "borg_danger.rs:2043", "Sets sleep level margin, measured in level, in the existing danger rule."),
    danger_ghost_monster => (100, "danger", 0, 5000, "borg_danger.rs:2145", "Sets ghost monster, measured in danger, in the existing danger rule."),
    danger_monster_max_distance => (20, "grids", 1, 100, "borg_danger.rs:2138", "Sets monster max distance, measured in grids, in the existing danger rule."),
    danger_fast_player_threshold => (135, "speed", 110, 135, "borg_danger.rs:2143", "Sets fast player threshold, measured in speed, in the existing danger rule."),
    danger_fast_player_unique_speed => (120, "speed", 110, 189, "borg_danger.rs:2144", "Sets fast player unique speed, measured in speed, in the existing danger rule."),
    danger_fast_player_normal_speed => (125, "speed", 110, 189, "borg_danger.rs:2144", "Sets fast player normal speed, measured in speed, in the existing danger rule."),
    danger_low_hp_speed_limit => (20, "hp", 0, 10000, "borg_danger.rs:2180", "Sets low hp speed limit, measured in hp, in the existing danger rule."),
    danger_low_hp_monster_speed_bonus => (3, "speed", 0, 3, "borg_danger.rs:2181", "Sets low hp monster speed bonus, measured in speed, in the existing danger rule."),
    danger_minimum_monster_energy => (10, "energy_tenths", 1, 100, "borg_danger.rs:2235", "Sets minimum monster energy, measured in energy_tenths, in the existing danger rule."),
    danger_stuck_bounce_limit => (1200, "count", 0, 10000, "borg_danger.rs:2135", "Sets stuck bounce limit, measured in count, in the existing danger rule."),
    danger_stuck_level_turns => (25000, "turns", 0, 100000, "borg_danger.rs:2135", "Sets stuck level turns, measured in turns, in the existing danger rule."),
    danger_stuck_melee_divisor => (5, "divisor", 1, 1000, "borg_danger.rs:2136", "Sets stuck melee divisor, measured in divisor, in the existing danger rule."),
    danger_breeder_melee_max_level => (20, "level", 1, 51, "borg_danger.rs:2150", "Sets breeder melee max level, measured in level, in the existing danger rule."),
    danger_breeder_melee_extra_tenths => (15, "tenths", 0, 30, "borg_danger.rs:2152", "Sets breeder melee extra tenths, measured in tenths, in the existing danger rule."),
    danger_friends_melee_max_level => (20, "level", 1, 51, "borg_danger.rs:2158", "Sets friends melee max level, measured in level, in the existing danger rule."),
    danger_friends_melee_low_level => (15, "level", 1, 51, "borg_danger.rs:2159", "Sets friends melee low level, measured in level, in the existing danger rule."),
    danger_friends_melee_low_extra_tenths => (18, "tenths", 0, 30, "borg_danger.rs:2161", "Sets friends melee low extra tenths, measured in tenths, in the existing danger rule."),
    danger_friends_melee_extra_tenths => (13, "tenths", 0, 30, "borg_danger.rs:2164", "Sets friends melee extra tenths, measured in tenths, in the existing danger rule."),
    danger_sleeping_melee_alertness => (5, "sleep_rating", 0, 1000, "borg_danger.rs:2170", "Sets sleeping melee alertness, measured in sleep_rating, in the existing danger rule."),
    danger_sleeping_melee_min_level => (25, "level", 1, 51, "borg_danger.rs:2172", "Sets sleeping melee min level, measured in level, in the existing danger rule."),
    danger_sleeping_melee_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2173", "Sets sleeping melee divisor, measured in divisor, in the existing danger rule."),
    danger_sleep2_melee_weak_level => (20, "level", 1, 51, "borg_danger.rs:2183", "Sets sleep2 melee weak level, measured in level, in the existing danger rule."),
    danger_sleep2_melee_weak_hp_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2183", "Sets sleep2 melee weak hp divisor, measured in divisor, in the existing danger rule."),
    danger_sleep2_melee_divisor => (3, "divisor", 1, 1000, "borg_danger.rs:2186", "Sets sleep2 melee divisor, measured in divisor, in the existing danger rule."),
    danger_sleep_melee_weak_level => (20, "level", 1, 51, "borg_danger.rs:2192", "Sets sleep melee weak level, measured in level, in the existing danger rule."),
    danger_sleep_melee_weak_hp_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2192", "Sets sleep melee weak hp divisor, measured in divisor, in the existing danger rule."),
    danger_sleep_melee_distance_offset => (2, "count", 0, 1000, "borg_danger.rs:2195", "Sets sleep melee distance offset, measured in count, in the existing danger rule."),
    danger_confused_melee_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2204", "Sets confused melee divisor, measured in divisor, in the existing danger rule."),
    danger_stunned_melee_numerator => (10, "coefficient", 0, 1000, "borg_danger.rs:2207", "Sets stunned melee numerator, measured in coefficient, in the existing danger rule."),
    danger_stunned_melee_divisor => (13, "divisor", 1, 1000, "borg_danger.rs:2207", "Sets stunned melee divisor, measured in divisor, in the existing danger rule."),
    danger_confuse_spell_weak_level => (20, "level", 1, 51, "borg_danger.rs:2211", "Sets confuse spell weak level, measured in level, in the existing danger rule."),
    danger_confuse_spell_weak_hp_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2211", "Sets confuse spell weak hp divisor, measured in divisor, in the existing danger rule."),
    danger_confuse_spell_distance_offset => (2, "count", 0, 1000, "borg_danger.rs:2214", "Sets confuse spell distance offset, measured in count, in the existing danger rule."),
    danger_stuck_spell_divisor => (5, "divisor", 1, 1000, "borg_danger.rs:2401", "Sets stuck spell divisor, measured in divisor, in the existing danger rule."),
    danger_breeder_spell_max_level => (20, "level", 1, 51, "borg_danger.rs:2405", "Sets breeder spell max level, measured in level, in the existing danger rule."),
    danger_breeder_spell_extra_tenths => (12, "tenths", 0, 30, "borg_danger.rs:2406", "Sets breeder spell extra tenths, measured in tenths, in the existing danger rule."),
    danger_friends_spell_max_level => (20, "level", 1, 51, "borg_danger.rs:2410", "Sets friends spell max level, measured in level, in the existing danger rule."),
    danger_friends_spell_extra_tenths => (12, "tenths", 0, 30, "borg_danger.rs:2411", "Sets friends spell extra tenths, measured in tenths, in the existing danger rule."),
    danger_sleeping_spell_alertness => (5, "sleep_rating", 0, 1000, "borg_danger.rs:2416", "Sets sleeping spell alertness, measured in sleep_rating, in the existing danger rule."),
    danger_sleeping_spell_min_level => (25, "level", 1, 51, "borg_danger.rs:2418", "Sets sleeping spell min level, measured in level, in the existing danger rule."),
    danger_sleeping_spell_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2419", "Sets sleeping spell divisor, measured in divisor, in the existing danger rule."),
    danger_sleep2_spell_linear_level => (15, "level", 1, 51, "borg_danger.rs:2430", "Sets sleep2 spell linear level, measured in level, in the existing danger rule."),
    danger_sleep2_spell_level_subtract => (10, "levels", 0, 1000, "borg_danger.rs:2430", "Sets sleep2 spell level subtract, measured in levels, in the existing danger rule."),
    danger_sleep2_spell_level_divisor => (4, "divisor", 1, 1000, "borg_danger.rs:2430", "Sets sleep2 spell level divisor, measured in divisor, in the existing danger rule."),
    danger_sleep2_spell_level_multiplier => (3, "multiplier", 0, 50, "borg_danger.rs:2430", "Sets sleep2 spell level multiplier, measured in multiplier, in the existing danger rule."),
    danger_sleep2_spell_level_add => (10, "levels", 0, 1000, "borg_danger.rs:2430", "Sets sleep2 spell level add, measured in levels, in the existing danger rule."),
    danger_sleep2_spell_divisor => (3, "divisor", 1, 1000, "borg_danger.rs:2437", "Sets sleep2 spell divisor, measured in divisor, in the existing danger rule."),
    danger_sleep_spell_distance_offset => (2, "count", 0, 1000, "borg_danger.rs:2448", "Sets sleep spell distance offset, measured in count, in the existing danger rule."),
    danger_confused_spell_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:2452", "Sets confused spell divisor, measured in divisor, in the existing danger rule."),
    danger_stunned_spell_numerator => (10, "coefficient", 0, 1000, "borg_danger.rs:2456", "Sets stunned spell numerator, measured in coefficient, in the existing danger rule."),
    danger_stunned_spell_divisor => (13, "divisor", 1, 1000, "borg_danger.rs:2456", "Sets stunned spell divisor, measured in divisor, in the existing danger rule."),
    danger_confuse_spell_divisor => (6, "divisor", 1, 1000, "borg_danger.rs:2459", "Sets confuse spell divisor, measured in divisor, in the existing danger rule."),
    danger_frequency_low_percent => (11, "percent", 0, 100, "borg_danger.rs:2087", "Sets frequency low percent, measured in percent, in the existing danger rule."),
    danger_frequency_low_tenths => (4, "tenths", 0, 30, "borg_danger.rs:2088", "Sets frequency low tenths, measured in tenths, in the existing danger rule."),
    danger_frequency_mid_percent => (26, "percent", 0, 100, "borg_danger.rs:2089", "Sets frequency mid percent, measured in percent, in the existing danger rule."),
    danger_frequency_mid_tenths => (6, "tenths", 0, 30, "borg_danger.rs:2090", "Sets frequency mid tenths, measured in tenths, in the existing danger rule."),
    danger_frequency_high_percent => (51, "percent", 0, 100, "borg_danger.rs:2091", "Sets frequency high percent, measured in percent, in the existing danger rule."),
    danger_frequency_high_tenths => (8, "tenths", 0, 30, "borg_danger.rs:2092", "Sets frequency high tenths, measured in tenths, in the existing danger rule."),
    danger_maximum => (2000, "danger", 0, 5000, "borg_danger.rs:2487", "Sets maximum, measured in danger, in the existing danger rule."),
    danger_regional_fear_max_depth => (80, "depth", 0, 127, "borg_danger.rs:2666", "Sets regional fear max depth, measured in depth, in the existing danger rule."),
    danger_morgoth_regional_fear_cap => (300, "danger", 0, 5000, "borg_danger.rs:2671", "Sets morgoth regional fear cap, measured in danger, in the existing danger rule."),
    danger_monster_fear_bounce_limit => (200, "count", 0, 1000, "borg_danger.rs:2678", "Sets monster fear bounce limit, measured in count, in the existing danger rule."),
    danger_rsf_s_kin_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1685", "Implicit factor 1 at the cited protected branch (spell_power with no literal multiplier); spell-power risk multiplier for RSF_S_KIN."),
    danger_rsf_s_hi_demon_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1703", "Implicit factor 1 at the cited protected branch (spell_power with no literal multiplier); spell-power risk multiplier for RSF_S_HI_DEMON."),
    danger_rsf_s_hydra_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1759", "Implicit factor 1 at the cited protected branch (spell_power with no literal multiplier); spell-power risk multiplier for RSF_S_HYDRA."),
    danger_rsf_s_ainu_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1777", "Implicit factor 1 at the cited protected branch (spell_power with no literal multiplier); spell-power risk multiplier for RSF_S_AINU."),
    danger_rsf_s_demon_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1795", "Implicit factor 1 at the cited protected branch (spell_power with no literal multiplier); spell-power risk multiplier for RSF_S_DEMON."),
    danger_rsf_s_hi_undead_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1813", "Implicit factor 1 at the cited protected branch (spell_power with no literal multiplier); spell-power risk multiplier for RSF_S_HI_UNDEAD."),
    danger_rsf_s_unique_protected_power_multiplier => (1, "multiplier", 0, 50, "borg_danger.rs:1849", "Implicit factor 1 at the cited protected branch (spell_power with no literal multiplier); spell-power risk multiplier for RSF_S_UNIQUE."),
    escape_phase_adjacent_only => (0, "flag", 0, 1, "borg_escape.rs:489", "#539 (owner): 1 = no Phase Door unless an awake monster is next to the Borg or one in view has a ranged attack (\"do not phase door when a monster is still distant\"); 0 = off (C's behaviour)."),
    escape_phase_adjacent_rule => (0, "flag", 0, 1, "borg_escape.rs:902", "#584 (owner): 1 = at every Phase Door site (docs/PHASE-DOOR.md), if the Borg can Phase Door, no Phase Door unless an awake enemy is on an adjacent square and the fight is judged lost (escape_phase_lost_test). The rule only refuses the Phase Door: the escape levels go on with their other escapes, and if none acts the Borg attacks (its best attack by its own ranking; with no enemy adjacent only a ranged one) or, with an awake monster in view, boosts (Speed first, then Heroism, Berserk Strength, Bless; else borg_defend's resistances, Protection from Evil, shields and staves). Decided once per think; #539's escape_phase_adjacent_only check is skipped while it is on. 0 = each site's own check (C, #539)."),
    escape_teleport_block => (0, "flag", 0, 1, "borg_escape.rs:1717", "#829: 1 = a teleport curse stops every teleport of the player. The Borg learns it from \"Your X prevents you teleporting.\" and \"Teleportation forbidden!\" (the engine's own words, whatever the item's identification state) and keeps it while the watched equipment is unchanged or until \"The anti-teleportation curse is removed!\"; Phase Door (every site, inside phase_door), Teleportation and Teleport Level scrolls, the staff of Teleportation, the teleport activations and the teleporting spells are then refused, so the site goes on with its next escape. A no-teleport grid is not asked for Phase Door (the engine does not stop it). 0 = as before (BI_CRSNOTEL only, at the sites that ask borg_allow_teleport)."),
    escape_phase_lost_danger_percent => (100, "percent of HP", 0, 1000, "borg_escape.rs:1010", "#584: with escape_phase_adjacent_rule, an adjacent fight is judged lost (Phase Door allowed) when escape_phase_lost_test is 0: the danger where the Borg stands at or above this percent of the current HP; 100 = the Borg could die this turn."),
    escape_teleport_landing_either => (0, "flag", 0, 1, "borg_escape.rs:1578", "#724 (upstream Angband 95387b06): 1 = borg_caution_teleport checks the teleport landing with a teleport scroll or spell (BI_ATELEPORT) or a teleport staff or activation (BI_AESCAPE); 0 = C and borg-rs (both needed, so with only one the check never ran and the Destruction and Teleport Level defences never saw a safe landing)."),
    escape_phase_lost_mana => (1, "flag", 0, 1, "borg_escape.rs:1009", "#625 (owner): with escape_phase_adjacent_rule on and escape_phase_lost_test 1 or 2, 1 = the fight test weighs the chosen attack's true damage by its success chance (a spell's fail rate, a device's activation failure) and uses it only for the casts the current mana pays for, all of it (owner: the escape reserve is not set aside; a wand: its charges; a rod: one), then the best other attack that needs no mana, or melee; 0 = the chosen attack at full damage every turn (#612)."),
    escape_phase_lost_test => (1, "choice", 0, 2, "borg_escape.rs:1009", "#584 (owner): with escape_phase_adjacent_rule, how an adjacent fight is judged lost (Phase Door allowed): 0 = the danger where the Borg stands at or above escape_phase_lost_danger_percent of the HP; 1 = the fight compared: killing the adjacent awake monsters one after another (their HP left) takes more turns than the danger where it stands needs to kill the Borg; its damage a turn is the larger of its melee and the true damage of the attack it would choose now, of any kind (a Mage casts next to an enemy); 2 = as 1, and the mean danger of the sampled landing grids is below the danger here."),
    escape_fox_form => (0, "flag", 0, 1, "borg_escape.rs:524", "#551: 1 = a Druid about to take a flight step (GOAL_FLEE) at least escape_fox_form_min_steps from its goal casts Fox Form (double movement, Free Action) first, and keeps the shape while the flight lasts: an item or a spell answers the game's \"Change back and ...?\" with y (same command, no extra energy). 0 = C (never cast; any shape is reverted at once)."),
    escape_fox_form_fail => (25, "percent", 0, 100, "borg_escape.rs:562", "#551: with escape_fox_form, the highest Fox Form fail rate at which it is cast (Fox Form: level 1, 20% base fail)."),
    escape_fox_form_min_steps => (3, "steps of flow", 1, 250, "borg_escape.rs:659", "#551: with escape_fox_form, the fewest flow steps left to the flight's goal for which the cast (one turn) is worth it."),
    escape_fox_form_speed => (10, "speed", 0, 30, "borg_escape.rs:715", "#551: with escape_fox_form, the speed a fox's flight is worth: borg_danger while a fox (it only moves), and the stand decision's pursuer test while a fox (MOVES[1] halves a step's energy: +10 at normal speed)."),
    escape_fox_form_stand_speed => (10, "speed", 0, 30, "borg_escape.rs:717", "#551 (review of #594): with escape_fox_form, the speed the stand decision's pursuer test counts for a Druid that is not yet a fox but can cast Fox Form (it will at its first flight step): more speed, fewer pursuers, more flights. 0 = only a fox counts."),
    escape_fox_form_town => (0, "flag", 0, 1, "borg_escape.rs:531", "#620 (owner): 1 = a Druid at most escape_fox_form_town_max_level, about to take a step of a walk in town (to a shop, home, a staircase, or a flight; not a kill or a take) at least escape_fox_form_town_min_steps from its goal, casts Fox Form (double movement) if its fail rate is at most escape_fox_form_fail, keeps it while the walk lasts, and changes back at the shop door (the game shuts it on a fox) or before a blow. 0 = C (never in town)."),
    escape_fox_form_town_max_level => (20, "level", 1, 51, "borg_escape.rs:552", "#620: with escape_fox_form_town, the highest character level that casts Fox Form for a walk in town (the manual's CL1-20 escape, 14-class-playbooks.md:453)."),
    escape_fox_form_town_min_steps => (10, "steps of flow", 1, 250, "borg_escape.rs:566", "#620: with escape_fox_form_town, the fewest flow steps left to the walk's goal for which the cast (one turn, and its mana) is worth it."),
    magic_fox_form_study_rating => (0, "rating", 0, 255, "borg_magic.rs:120", "#694 (owner): \"make the druid learn this spell first\". Above 0, the rating Fox Form is studied by, in place of C's 5 (below the 50 a spell needs unless the Borg is bored: the Druid learns it once bored). 100 puts it above Detect Life and Stinking Cloud (95): learned first. 0 = C's rating."),
    escape_antisummon_reset_turns => (50, "turns", 0, 100000, "borg_escape.rs:87", "Sets antisummon reset turns, measured in turns, in the existing escape rule."),
    escape_recall_fail => (60, "percent", 0, 100, "borg_escape.rs:116", "Sets recall fail, measured in percent, in the existing escape rule."),
    escape_recall_depth_window => (5, "depth", 0, 127, "borg_escape.rs:112", "Sets recall depth window, measured in depth, in the existing escape rule."),
    escape_recall_reset_min_depth => (80, "depth", 0, 127, "borg_escape.rs:116", "Sets recall reset min depth, measured in depth, in the existing escape rule."),
    escape_recall_reset_unique_count => (5, "count", 0, 1000, "borg_escape.rs:128", "Sets recall reset unique count, measured in count, in the existing escape rule."),
    escape_surround_distance => (3, "grids", 1, 100, "borg_escape.rs:198", "Sets surround distance, measured in grids, in the existing escape rule."),
    escape_surround_single_exit_count => (1, "count", 1, 8, "borg_escape.rs:266", "Sets surround single exit count, measured in count, in the existing escape rule."),
    escape_surround_single_monster_count => (1, "count", 1, 8, "borg_escape.rs:266", "Sets surround single monster count, measured in count, in the existing escape rule."),
    escape_town_stair_value => (1000, "freedom_score", 0, 5000, "borg_escape.rs:333", "Sets town stair value, measured in freedom_score, in the existing escape rule."),
    escape_town_stair_close_distance => (4, "grids", 1, 100, "borg_escape.rs:336", "Sets town stair close distance, measured in grids, in the existing escape rule."),
    escape_town_stair_close_value => (2000, "freedom_score", 0, 5000, "borg_escape.rs:337", "Sets town stair close value, measured in freedom_score, in the existing escape rule."),
    escape_town_stair_distance_penalty => (500, "freedom_score", 0, 5000, "borg_escape.rs:337", "Sets town stair distance penalty, measured in freedom_score, in the existing escape rule."),
    escape_dungeon_stair_value => (1000, "freedom_score", 0, 5000, "borg_escape.rs:331", "Sets dungeon stair value, measured in freedom_score, in the existing escape rule."),
    escape_dungeon_stair_close_distance => (4, "grids", 1, 100, "borg_escape.rs:334", "Sets dungeon stair close distance, measured in grids, in the existing escape rule."),
    escape_dungeon_stair_close_value => (2000, "freedom_score", 0, 5000, "borg_escape.rs:335", "Sets dungeon stair close value, measured in freedom_score, in the existing escape rule."),
    escape_dungeon_stair_distance_penalty => (500, "freedom_score", 0, 5000, "borg_escape.rs:335", "Sets dungeon stair distance penalty, measured in freedom_score, in the existing escape rule."),
    escape_landing_samples => (100, "count", 1, 1000, "borg_escape.rs:379", "Sets landing samples, measured in count, in the existing escape rule."),
    escape_landing_trials => (100, "count", 1, 1000, "borg_escape.rs:393", "Sets landing trials, measured in count, in the existing escape rule."),
    escape_unknown_landing_max_hp => (30, "hp", 0, 10000, "borg_escape.rs:419", "Sets unknown landing max hp, measured in hp, in the existing escape rule."),
    escape_unknown_teleport_level_turns => (2000, "turns", 0, 100000, "borg_escape.rs:1574", "Sets unknown teleport level turns, measured in turns, in the existing escape rule."),
    escape_shadow_shift_min_hp => (12, "hp", 0, 10000, "borg_escape.rs:1825", "Sets shadow shift min hp, measured in hp, in the existing escape rule."),
    escape_dimension_door_range => (50, "grids", 1, 100, "borg_escape.rs:1834", "Sets dimension door range, measured in grids, in the existing escape rule."),
    escape_dimension_door_horizon => (2, "turns", 1, 10, "borg_escape.rs:1852", "Sets dimension door horizon, measured in turns, in the existing escape rule."),
    escape_near_unique_max_count => (8, "count", 1, 9, "borg_escape.rs:1925", "Sets near unique max count, measured in count, in the existing escape rule."),
    escape_normal_fail => (25, "percent", 0, 100, "borg_escape.rs:1952", "Sets normal fail, measured in percent, in the existing escape rule."),
    escape_healthy_hp_percent => (70, "percent", 0, 100, "borg_escape.rs:1955", "Sets healthy hp percent, measured in percent, in the existing escape rule."),
    escape_healthy_fail => (10, "percent", 0, 100, "borg_escape.rs:1956", "Sets healthy fail, measured in percent, in the existing escape rule."),
    escape_stunned_fail => (35, "percent", 0, 100, "borg_escape.rs:1961", "Sets stunned fail, measured in percent, in the existing escape rule."),
    escape_morgoth_hold_hp_tenths => (5, "tenths", 0, 100, "borg_escape.rs:1982", "Sets morgoth hold hp tenths, measured in tenths, in the existing escape rule."),
    escape_morgoth_hold_glyph_count => (3, "count", 0, 1000, "borg_escape.rs:1965", "Sets morgoth hold glyph count, measured in count, in the existing escape rule."),
    escape_risky_danger_extra_tenths => (3, "tenths", 0, 100, "borg_escape.rs:1999", "Sets risky danger extra tenths, measured in tenths, in the existing escape rule."),
    escape_emergency_danger_tenths => (45, "tenths", 0, 100, "borg_escape.rs:2026", "Sets emergency danger tenths, measured in tenths, in the existing escape rule."),
    escape_morgoth_danger_tenths => (40, "tenths", 0, 100, "borg_escape.rs:2027", "Sets morgoth danger tenths, measured in tenths, in the existing escape rule."),
    escape_morgoth_max_hp => (600, "hp", 0, 10000, "borg_escape.rs:2030", "Sets morgoth max hp, measured in hp, in the existing escape rule."),
    escape_sauron_danger_tenths => (30, "tenths", 0, 100, "borg_escape.rs:2031", "Sets sauron danger tenths, measured in tenths, in the existing escape rule."),
    escape_sauron_max_hp => (600, "hp", 0, 10000, "borg_escape.rs:2034", "Sets sauron max hp, measured in hp, in the existing escape rule."),
    escape_deep_unique_danger_tenths => (25, "tenths", 0, 100, "borg_escape.rs:2035", "Sets deep unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_deep_unique_depth => (95, "depth", 0, 127, "borg_escape.rs:2037", "Sets deep unique depth, measured in depth, in the existing escape rule."),
    escape_deep_unique_max_hp => (550, "hp", 0, 10000, "borg_escape.rs:2038", "Sets deep unique max hp, measured in hp, in the existing escape rule."),
    escape_unique_danger_tenths => (17, "tenths", 0, 100, "borg_escape.rs:1948", "Sets unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_ordinary_danger_tenths => (15, "tenths", 0, 100, "borg_escape.rs:1949", "Sets ordinary danger tenths, measured in tenths, in the existing escape rule."),
    escape_emergency_fail => (15, "percent", 0, 100, "borg_escape.rs:1951", "Sets emergency fail, measured in percent, in the existing escape rule."),
    escape_initial_fail_discount => (10, "percent", 0, 100, "borg_escape.rs:1955", "Sets initial fail discount, measured in percent, in the existing escape rule."),
    escape_emergency_fail_extra => (9, "percent", 0, 100, "borg_escape.rs:1964", "Sets emergency fail extra, measured in percent, in the existing escape rule."),
    escape_emergency_phase_tolerance => (75, "count", 0, 1000, "borg_escape.rs:1974", "Sets emergency phase tolerance, measured in count, in the existing escape rule."),
    escape_emergency_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:1974", "Sets emergency phase horizon, measured in turns, in the existing escape rule."),
    escape_last_ditch_max_level => (10, "level", 1, 51, "borg_escape.rs:1993", "Sets last ditch max level, measured in level, in the existing escape rule."),
    escape_last_ditch_hp_divisor => (10, "divisor", 1, 1000, "borg_escape.rs:1994", "Sets last ditch hp divisor, measured in divisor, in the existing escape rule."),
    escape_last_ditch_dimension_fail => (90, "percent", 0, 100, "borg_escape.rs:1996", "Sets last ditch dimension fail, measured in percent, in the existing escape rule."),
    escape_critical_phase_hp_divisor => (10, "divisor", 1, 1000, "borg_escape.rs:2008", "Sets critical phase hp divisor, measured in divisor, in the existing escape rule."),
    escape_last_ditch_phase_max_level => (10, "level", 1, 51, "borg_escape.rs:2022", "Sets last ditch phase max level, measured in level, in the existing escape rule."),
    escape_last_ditch_phase_hp_divisor => (10, "divisor", 1, 1000, "borg_escape.rs:2023", "Sets last ditch phase hp divisor, measured in divisor, in the existing escape rule."),
    escape_last_ditch_phase_fail => (15, "percent", 0, 100, "borg_escape.rs:2025", "Sets last ditch phase fail, measured in percent, in the existing escape rule."),
    escape_endgame_stay_danger_tenths => (25, "tenths", 0, 100, "borg_escape.rs:2040", "Sets endgame stay danger tenths, measured in tenths, in the existing escape rule."),
    escape_endgame_stay_unique_count => (3, "count", 1, 9, "borg_escape.rs:2042", "Sets endgame stay unique count, measured in count, in the existing escape rule."),
    escape_endgame_stay_min_depth => (97, "depth", 0, 127, "borg_escape.rs:2043", "Sets endgame stay min depth, measured in depth, in the existing escape rule."),
    escape_mage_danger_tenths => (3, "tenths", 0, 100, "borg_escape.rs:2052", "Sets mage danger tenths, measured in tenths, in the existing escape rule."),
    escape_mage_max_sp => (20, "sp", 0, 10000, "borg_escape.rs:2054", "Sets mage max sp, measured in sp, in the existing escape rule."),
    escape_mage_min_level => (45, "level", 1, 51, "borg_escape.rs:2055", "Sets mage min level, measured in level, in the existing escape rule."),
    escape_teleport_unique_danger_tenths => (13, "tenths", 0, 100, "borg_escape.rs:2056", "Sets teleport unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_teleport_ordinary_danger_tenths => (11, "tenths", 0, 100, "borg_escape.rs:2057", "Sets teleport ordinary danger tenths, measured in tenths, in the existing escape rule."),
    escape_teleport_phase_tolerance => (50, "count", 0, 1000, "borg_escape.rs:2083", "Sets teleport phase tolerance, measured in count, in the existing escape rule."),
    escape_teleport_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:2083", "Sets teleport phase horizon, measured in turns, in the existing escape rule."),
    escape_phase_antisummon_delay => (50, "turns", 0, 100000, "borg_escape.rs:2084", "Sets phase antisummon delay, measured in turns, in the existing escape rule."),
    escape_phase_unique_danger_tenths => (13, "tenths", 0, 100, "borg_escape.rs:2101", "Sets phase unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_phase_unique_min_count => (2, "count", 1, 9, "borg_escape.rs:2101", "Sets phase unique min count, measured in count, in the existing escape rule."),
    escape_phase_ordinary_danger_tenths => (10, "tenths", 0, 100, "borg_escape.rs:2102", "Sets phase ordinary danger tenths, measured in tenths, in the existing escape rule."),
    escape_afraid_warrior_danger_tenths => (10, "tenths", 0, 100, "borg_escape.rs:2103", "Sets afraid warrior danger tenths, measured in tenths, in the existing escape rule."),
    escape_phase_tolerance => (25, "count", 0, 1000, "borg_escape.rs:2159", "Sets phase tolerance, measured in count, in the existing escape rule."),
    escape_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:2159", "Sets phase horizon, measured in turns, in the existing escape rule."),
    escape_fallback_phase_tolerance => (75, "count", 0, 1000, "borg_escape.rs:2249", "Sets fallback phase tolerance, measured in count, in the existing escape rule."),
    escape_fallback_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:2249", "Sets fallback phase horizon, measured in turns, in the existing escape rule."),
    escape_failed_teleport_flee_max_level => (35, "level", 1, 51, "borg_escape.rs:1959", "Sets failed teleport flee max level, measured in level, in the existing escape rule."),
    escape_failed_teleport_leave_max_level => (35, "level", 1, 51, "borg_escape.rs:1971", "Sets failed teleport leave max level, measured in level, in the existing escape rule."),
    escape_compromised_danger_tenths => (8, "tenths", 0, 100, "borg_escape.rs:1983", "Sets compromised danger tenths, measured in tenths, in the existing escape rule."),
    escape_compromised_max_level => (35, "level", 1, 51, "borg_escape.rs:1984", "Sets compromised max level, measured in level, in the existing escape rule."),
    escape_compromised_hp_divisor => (3, "divisor", 1, 1000, "borg_escape.rs:1984", "Sets compromised hp divisor, measured in divisor, in the existing escape rule."),
    escape_compromised_unique_danger_tenths => (9, "tenths", 0, 100, "borg_escape.rs:1985", "Sets compromised unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_compromised_unique_max_level => (35, "level", 1, 51, "borg_escape.rs:1987", "Sets compromised unique max level, measured in level, in the existing escape rule."),
    escape_compromised_unique_hp_divisor => (3, "divisor", 1, 1000, "borg_escape.rs:1987", "Sets compromised unique hp divisor, measured in divisor, in the existing escape rule."),
    escape_compromised_ordinary_danger_tenths => (6, "tenths", 0, 100, "borg_escape.rs:1988", "Sets compromised ordinary danger tenths, measured in tenths, in the existing escape rule."),
    escape_compromised_ordinary_max_level => (20, "level", 1, 51, "borg_escape.rs:1988", "Sets compromised ordinary max level, measured in level, in the existing escape rule."),
    escape_compromised_general_danger_tenths => (6, "tenths", 0, 100, "borg_escape.rs:1989", "Sets compromised general danger tenths, measured in tenths, in the existing escape rule."),
    escape_compromised_general_max_level => (35, "level", 1, 51, "borg_escape.rs:1989", "Sets compromised general max level, measured in level, in the existing escape rule."),
    escape_compromised_phase_tolerance => (20, "count", 0, 1000, "borg_escape.rs:1993", "Sets compromised phase tolerance, measured in count, in the existing escape rule."),
    escape_compromised_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:1993", "Sets compromised phase horizon, measured in turns, in the existing escape rule."),
    escape_failed_compromised_flee_max_level => (25, "level", 1, 51, "borg_escape.rs:2032", "Sets failed compromised flee max level, measured in level, in the existing escape rule."),
    escape_weak_caster_max_level => (35, "level", 1, 51, "borg_escape.rs:2052", "Sets weak caster max level, measured in level, in the existing escape rule."),
    escape_weak_caster_phase_tolerance => (65, "count", 0, 1000, "borg_escape.rs:2053", "Sets weak caster phase tolerance, measured in count, in the existing escape rule."),
    escape_weak_caster_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:2053", "Sets weak caster phase horizon, measured in turns, in the existing escape rule."),
    escape_low_level_max_level => (10, "level", 1, 51, "borg_escape.rs:2085", "Sets low level max level, measured in level, in the existing escape rule."),
    escape_low_level_danger_tenths => (5, "tenths", 0, 100, "borg_escape.rs:2086", "Sets low level danger tenths, measured in tenths, in the existing escape rule."),
    escape_low_level_unique_danger_tenths => (7, "tenths", 0, 100, "borg_escape.rs:2086", "Sets low level unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_low_level_phase_tolerance => (20, "count", 0, 1000, "borg_escape.rs:2090", "Sets low level phase tolerance, measured in count, in the existing escape rule."),
    escape_low_level_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:2090", "Sets low level phase horizon, measured in turns, in the existing escape rule."),
    escape_low_caster_max_level => (8, "level", 1, 51, "borg_escape.rs:2345", "Sets low caster max level, measured in level, in the existing escape rule."),
    escape_low_caster_phase_tolerance => (65, "count", 0, 1000, "borg_escape.rs:2346", "Sets low caster phase tolerance, measured in count, in the existing escape rule."),
    escape_low_caster_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:2346", "Sets low caster phase horizon, measured in turns, in the existing escape rule."),
    escape_low_mana_danger_tenths => (6, "tenths", 0, 100, "borg_escape.rs:2367", "Sets low mana danger tenths, measured in tenths, in the existing escape rule."),
    escape_low_mana_unique_danger_tenths => (8, "tenths", 0, 100, "borg_escape.rs:2367", "Sets low mana unique danger tenths, measured in tenths, in the existing escape rule."),
    escape_low_mana_sp_divisor => (10, "divisor", 1, 1000, "borg_escape.rs:2368", "Sets low mana sp divisor, measured in divisor, in the existing escape rule."),
    escape_low_mana_min_maxsp => (100, "sp", 0, 10000, "borg_escape.rs:2368", "Sets low mana min maxsp, measured in sp, in the existing escape rule."),
    escape_low_mana_phase_tolerance => (20, "count", 0, 1000, "borg_escape.rs:2372", "Sets low mana phase tolerance, measured in count, in the existing escape rule."),
    escape_low_mana_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:2372", "Sets low mana phase horizon, measured in turns, in the existing escape rule."),
    escape_scoot_tolerance => (20, "count", 0, 1000, "borg_escape.rs:2412", "Sets scoot tolerance, measured in count, in the existing escape rule."),
    escape_scoot_horizon => (2, "turns", 1, 10, "borg_escape.rs:2412", "Sets scoot horizon, measured in turns, in the existing escape rule."),
    escape_twitch_limit => (50, "count", 0, 1000, "borg_escape.rs:2427", "Sets twitch limit, measured in count, in the existing escape rule."),
    escape_twitch_phase_tolerance => (20, "count", 0, 1000, "borg_escape.rs:2430", "Sets twitch phase tolerance, measured in count, in the existing escape rule."),
    escape_twitch_phase_horizon => (2, "turns", 1, 10, "borg_escape.rs:2430", "Sets twitch phase horizon, measured in turns, in the existing escape rule."),
    danger_default_hit_power => (0, "hit_power", 0, 1000, "borg_danger.rs:197", "Fallback hit-power estimate for blows without a specific legacy hit-power value."),
    danger_rsf_lash_damage => (100, "danger", 0, 5000, "borg_danger.rs:800", "Fixed danger estimate for a lash in reach."),
    danger_rsf_spit_damage => (100, "danger", 0, 5000, "borg_danger.rs:806", "Fixed danger estimate for spit in reach."),
    danger_pfe_melee_divisor => (2, "divisor", 1, 1000, "borg_danger.rs:212", "Divides melee damage plus fear when the legacy protection-from-evil approximation applies."),
    // borg-reborn-rs (#221): recovery source lines at 30ab9d66.
    recover_danger_horizon => (1, "turns", 1, 10, "borg_recover.rs:233", "Sets danger horizon, in turns, in the existing recovery rule."),
    recover_danger_divisor => (4, "divisor", 1, 1000, "borg_recover.rs:236", "Sets danger divisor, in divisor, in the existing recovery rule."),
    lightless_recovery_actions => (250, "actions", 1, 1000, "borg_lightless.rs:68", "#811: action budget per lightless episode on one dungeon level."),
    lightless_search_actions => (32, "actions", 0, 250, "borg_lightless.rs:69", "#811: action budget for walking to and probing map edges without a known return route. 0 disables probes."),
    lightless_door_attempts => (3, "attempts", 1, 10, "borg_lightless.rs:70", "#811: attempts from one approach to open a closed door during lightless recovery."),
    recover_hurt_hp_divisor => (2, "divisor", 1, 1000, "borg_recover.rs:229", "Sets hurt hp divisor, in divisor, in the existing recovery rule."),
    recover_hurt_chance_bonus => (10, "percent", 0, 100, "borg_recover.rs:230", "Sets hurt chance bonus, in percent, in the existing recovery rule."),
    recover_critical_hp_divisor => (4, "divisor", 1, 1000, "borg_recover.rs:234", "Sets critical hp divisor, in divisor, in the existing recovery rule."),
    recover_critical_chance_bonus => (10, "percent", 0, 100, "borg_recover.rs:235", "Sets critical chance bonus, in percent, in the existing recovery rule."),
    recover_cheap_stun_chance => (75, "percent", 0, 100, "borg_recover.rs:242", "Sets cheap stun chance, in percent, in the existing recovery rule."),
    recover_cheap_cut_chance => (75, "percent", 0, 100, "borg_recover.rs:278", "Sets cheap cut chance, in percent, in the existing recovery rule."),
    recover_cheap_poison_chance => (75, "percent", 0, 100, "borg_recover.rs:291", "Sets cheap poison chance, in percent, in the existing recovery rule."),
    recover_cheap_fear_chance => (75, "percent", 0, 100, "borg_recover.rs:304", "Sets cheap fear chance, in percent, in the existing recovery rule."),
    recover_cheap_hunger_chance => (75, "percent", 0, 100, "borg_recover.rs:317", "Sets cheap hunger chance, in percent, in the existing recovery rule."),
    recover_cheap_image_chance => (75, "percent", 0, 100, "borg_recover.rs:342", "Sets cheap image chance, in percent, in the existing recovery rule."),
    recover_cheap_heal_hp_divisor => (2, "divisor", 1, 1000, "borg_recover.rs:349", "Sets cheap heal hp divisor, in divisor, in the existing recovery rule."),
    recover_cheap_heal_chance => (75, "percent", 0, 100, "borg_recover.rs:350", "Sets cheap heal chance, in percent, in the existing recovery rule."),
    recover_cheap_heal_sp_divisor => (4, "divisor", 1, 1000, "borg_recover.rs:352", "Sets cheap heal sp divisor, in divisor, in the existing recovery rule."),
    recover_unholy_min_hp => (90, "hp", 0, 10000, "borg_recover.rs:237", "Sets unholy min hp, in hp, in the existing recovery rule."),
    recover_costly_stun_chance => (25, "percent", 0, 100, "borg_recover.rs:266", "Sets costly stun chance, in percent, in the existing recovery rule."),
    recover_costly_heavy_stun_chance => (95, "percent", 0, 100, "borg_recover.rs:279", "Sets costly heavy stun chance, in percent, in the existing recovery rule."),
    recover_costly_cut_chance => (25, "percent", 0, 100, "borg_recover.rs:292", "Sets costly cut chance, in percent, in the existing recovery rule."),
    recover_critical_ccw_hp => (10, "hp", 0, 10000, "borg_recover.rs:298", "Sets critical ccw hp, in hp, in the existing recovery rule."),
    recover_costly_poison_chance => (25, "percent", 0, 100, "borg_recover.rs:305", "Sets costly poison chance, in percent, in the existing recovery rule."),
    recover_costly_blind_chance => (25, "percent", 0, 100, "borg_recover.rs:321", "Sets costly blind chance, in percent, in the existing recovery rule."),
    recover_costly_confusion_chance => (25, "percent", 0, 100, "borg_recover.rs:336", "Sets costly confusion chance, in percent, in the existing recovery rule."),
    recover_costly_fear_chance => (25, "percent", 0, 100, "borg_recover.rs:350", "Sets costly fear chance, in percent, in the existing recovery rule."),
    recover_costly_hunger_chance => (25, "percent", 0, 100, "borg_recover.rs:362", "Sets costly hunger chance, in percent, in the existing recovery rule."),
    recover_costly_heal_hp_divisor => (2, "divisor", 1, 1000, "borg_recover.rs:370", "Sets costly heal hp divisor, in divisor, in the existing recovery rule."),
    recover_costly_heal_chance => (25, "percent", 0, 100, "borg_recover.rs:371", "Sets costly heal chance, in percent, in the existing recovery rule."),
    recover_rest_high_depth => (85, "depth", 0, 127, "borg_recover.rs:1322", "Sets rest high depth, in depth, in the existing recovery rule."),
    recover_rest_high_sp_tenths => (7, "tenths", 0, 10, "borg_recover.rs:1323", "Sets rest high sp tenths, in tenths, in the existing recovery rule."),
    recover_rest_arms_see_invis => (1, "flag", 0, 1, "borg_recover.rs:450", "#516: 1 = C: a rest stamps the see-invisible clock as recent, which arms the Detect Invisible defence after every rest; 0 = a rest leaves it alone (only an unseen attack arms it). The same clock gates the Light Beam defence (skipped while recent) and the removal of regional fear (kept while recent), so at 0 both can act after a rest where C did not. 187 reads on 18 games, 171 with no unseen monster on the level."),
    recover_rest_monster_turns => (0, "game turns", 0, 100000, "borg_recover.rs:56", "#615 rule a (owner): no rest (rod, HP/SP, mana) while an awake monster that can move was known within recover_rest_monster_distance in the last this many game turns on this level; a monster that comes into view would break the rest, and one that leaves the screen may come back. Kept when impaired (confused, blind, hallucinating, stunned, afraid); the clock clears when the monster that set it dies. 0 = off (C)."),
    recover_rest_monster_distance => (20, "grids", 1, 100, "borg_recover.rs:57", "#615 rule a: with recover_rest_monster_turns, how near an awake monster must be (the engine's distance)."),
    recover_rest_reach_stairs => (0, "steps of flow", 0, 250, "borg_recover.rs:143", "#614: a Borg on a flight to a staircase (GOAL_FLEE) at most this many flow steps away does not rest (rod, HP or mana) until it stands on the staircase; potions and cures are not rests; a farther staircase is a long walk, not a commitment. 0 = C (the low-level mana rest stopped a flight 11 steps short of the stairs)."),
    recover_rest_reach_stairs_memory => (10, "borg time", 0, 1000, "borg_recover.rs:164", "#614 (review of #616): with recover_rest_reach_stairs, how long after its last flight step, on the same level, a flight that is still on (fleeing, leaving or a scary level) keeps the Borg committed to its staircase when the flow goal was wiped (a new monster wipes it for a think)."),
    recover_rest_sp_tenths => (6, "tenths", 0, 10, "borg_recover.rs:1325", "Sets rest sp tenths, in tenths, in the existing recovery rule."),
    light_first_cast_require_dark => (0, "flag", 0, 1, "borg_light.rs:284", "#1119: the radius-one room-lighting check requires an unglowing casting square or observed unglowing floor within the spell's light sphere. 0 restores the first-cast behavior before #1119. Both values keep the confirmed-cast recast check."),
    light_hallway_repeat_guard => (0, "flag", 0, 1, "borg_light.rs:1166", "#995: require a set defensive Light Beam stamp and suppress repeated beams on unchanged terrain. 0 = legacy timing and beam selection for recorded starts."),
    light_detect_dark_entry => (0, "flag", 0, 1, "borg_light.rs:486", "#796: permit Detect Monsters or Read Minds below the usual level gate when entering an undetected dark frontier (one attempt per panel per level). 0 preserves current play."),
    light_detect_dark_entry_reserve_sp => (2, "spell_points", 0, 255, "borg_light.rs:499", "#796: minimum SP the dark-entry detection leaves, raised to the cheapest known suitable response and both applicable mana recovery floors."),
    light_detect_evil_spell_min_level => (0, "level", 0, 51, "borg_light.rs:593", "#685: below this character level the panel check casts no Detect Evil, Detect Monsters, Read Minds or Seek Battle (items and rods still detect). 0 = C. Per class with light_detect_evil_spell_min_level@<class>."),
    recover_mana_rest_max_level => (40, "level", 1, 51, "borg_recover.rs:1350", "Sets mana rest max level, in level, in the existing recovery rule."),
    recover_mana_rest_deep_depth => (85, "depth", 0, 127, "borg_recover.rs:1351", "Sets mana rest deep depth, in depth, in the existing recovery rule."),
    recover_mana_rest_sp_tenths => (8, "tenths", 0, 10, "borg_recover.rs:1352", "Sets mana rest sp tenths, in tenths, in the existing recovery rule."),
    recover_mana_rest_danger_divisor => (10, "divisor", 1, 1000, "borg_recover.rs:463", "Sets mana rest danger divisor, in divisor, in the existing recovery rule."),
    // #1211: the recorded Priest's corridor was priced at danger 20-25 (remembered orc archers within max_range), above the mana rest's own gate (17) but inside the general recovery gate (42) and the defense's "some danger" window — the Borg that cures and preps at that danger could never rest for the one resource it was missing.
    recover_mana_rest_full_hp => (0, "flag", 0, 1, "borg_recover.rs:638", "#1211: 1 = at full HP the Resting-to-gain-Mana danger gate is the general recovery gate (recover_danger_divisor) instead of the stricter recover_mana_rest_danger_divisor: only mana is missing, the top recovery veto already passed at that danger, and the rest ends at the first monster seen or blow. 0 retains C's gate."),
    recover_mana_rest_min_food => (2, "count", 0, 1000, "borg_recover.rs:469", "Sets mana rest min food, in count, in the existing recovery rule."),
    recover_rod_rest_turns => (100, "turns", 2, 9999, "borg_recover.rs:404", "Number of rest turns queued while waiting for a healing/recall rod to recharge."),
    // borg-reborn-rs (#240): attack source at 5d9abb8f.
    attack_melee_min_hit_tenths => (8, "tenths", 0, 16, "borg_fight_attack.rs:659", "Attack melee min hit tenths: existing policy tenths."),
    attack_melee_bonus_min_level => (15, "level", 0, 51, "borg_fight_attack.rs:649", "Attack melee bonus min level: existing policy level."),
    attack_melee_hit_bonus => (10, "percent", 0, 100, "borg_fight_attack.rs:650", "Attack melee hit bonus: existing policy percent."),
    attack_mage_melee_min_sp => (1, "mana", 0, 1000, "borg_fight_attack.rs:655", "Attack mage melee min sp: existing policy mana."),
    attack_mage_hit_penalty => (10, "percent", 0, 100, "borg_fight_attack.rs:657", "Attack mage hit penalty: existing policy percent."),
    attack_melee_overkill_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:669", "Attack melee overkill multiplier: existing policy multiplier."),
    attack_mage_melee_max_level => (40, "level", 0, 51, "borg_fight_attack.rs:677", "Attack mage melee max level: existing policy level."),
    attack_mage_melee_damage_tenths => (8, "tenths", 0, 16, "borg_fight_attack.rs:680", "Attack mage melee damage tenths: existing policy tenths."),
    attack_melee_unique_bonus => (5, "multiplier", 0, 10, "borg_fight_attack.rs:716", "Attack melee unique bonus: existing policy multiplier."),
    attack_melee_town_unique_numerator => (2, "multiplier", 0, 10, "borg_fight_attack.rs:734", "Attack melee town unique numerator: existing policy multiplier."),
    attack_melee_town_unique_divisor => (3, "divisor", 1, 1000, "borg_fight_attack.rs:734", "Attack melee town unique divisor: existing policy divisor."),
    attack_town_unique_min_level => (5, "level", 0, 51, "borg_fight_attack.rs:740", "Attack town unique min level: existing policy level."),
    attack_town_dangerous_on_sight => (0, "flag", 0, 1, "borg_fight_attack.rs:548", "#619 (owner): 1 = in town a dangerous town monster (attack_town_dangerous_min_damage; the veteran, the mercenary, the squint-eyed rogue, the urchin) is killed on sight once the town is not scary (the scary list's level per monster and class decides that): it is the preferred target (attack_town_dangerous_bonus_percent) with the best attack borg_attack has, and the hunt walks to it below CL20; 0 = C."),
    attack_town_dangerous_min_damage => (6, "damage", 1, 1000, "borg_fight_attack.rs:532", "#619: a town monster is dangerous when a blow steals (gold, an item) or hurts for at least this much (dice x sides): 6 takes the veteran (2d6), the mercenary (1d10) and the rogue (1d6), not the merchant (1d3) or the cats and dogs (1d1)."),
    attack_town_dangerous_bonus_percent => (300, "percent", 100, 10000, "borg_fight_attack.rs:553", "#619: a dangerous town monster's attack value is multiplied by this / 100, so it is chosen before the harmless ones."),
    attack_town_dangerous_caster_walk => (0, "flag", 0, 1, "borg_fight_attack.rs:569", "#637 (owner): with attack_town_dangerous_on_sight, 1 = in town a Mage or Necromancer below borg_flow_kill's caster level (CL25 in town; the town hunt gate stops at CL20, this does not) walks to a dangerous town monster too, with at least half its mana (its town hunt is otherwise breeders only, and no town race breeds). 0 = C."),
    attack_town_dangerous_wand => (0, "flag", 0, 1, "borg_fight_attack.rs:578", "#637 (owner): with attack_town_dangerous_on_sight, 1 = in town a wand (not Wonder) may be aimed when a dangerous town monster is known, and only those are worth its charge. 0 = C (no wands in town: charges are too spendy)."),
    attack_town_dangerous_bonus_whole => (0, "flag", 0, 1, "borg_fight_attack.rs:561", "#637 (owner): with attack_town_dangerous_on_sight, 1 = attack_town_dangerous_bonus_percent also multiplies the danger term of an attack's value (melee and bolts), so a dangerous town monster is the target for certain, not only when its damage term wins. 0 = the damage term only (#619)."),
    attack_melee_breeder_numerator => (3, "multiplier", 0, 10, "borg_fight_attack.rs:619", "Attack melee breeder numerator: existing policy multiplier."),
    attack_melee_breeder_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:619", "Attack melee breeder divisor: existing policy divisor."),
    attack_melee_summoner_numerator => (3, "multiplier", 0, 10, "borg_fight_attack.rs:633", "Attack melee summoner numerator: existing policy multiplier."),
    attack_melee_summoner_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:633", "Attack melee summoner divisor: existing policy divisor."),
    attack_melee_questor_bonus => (5, "multiplier", 0, 10, "borg_fight_attack.rs:653", "Attack melee questor bonus: existing policy multiplier."),
    attack_melee_sleep_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:804", "Attack melee sleep danger horizon: existing policy turns."),
    attack_melee_sleep_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:806", "Attack melee sleep danger multiplier: existing policy multiplier."),
    attack_melee_danger_horizon => (2, "turns", 1, 10, "borg_fight_attack.rs:802", "Attack melee danger horizon: existing policy turns."),
    attack_melee_partial_min_level => (15, "level", 0, 51, "borg_fight_attack.rs:810", "Attack melee partial min level: existing policy level."),
    attack_melee_partial_danger_divisor => (10, "divisor", 1, 1000, "borg_fight_attack.rs:811", "Attack melee partial danger divisor: existing policy divisor."),
    attack_status_low_level_cutoff => (13, "level", 0, 51, "borg_fight_attack.rs:1042", "Attack status low level cutoff: existing policy level."),
    attack_status_low_level_limit => (10, "level", 0, 51, "borg_fight_attack.rs:1042", "Attack status low level limit: existing policy level."),
    attack_status_level_offset => (10, "level", 0, 51, "borg_fight_attack.rs:1042", "Attack status level offset: existing policy level."),
    attack_status_level_divisor => (4, "divisor", 1, 1000, "borg_fight_attack.rs:1042", "Attack status level divisor: existing policy divisor."),
    attack_status_level_numerator => (3, "multiplier", 0, 10, "borg_fight_attack.rs:1042", "Attack status level numerator: existing policy multiplier."),
    attack_status_level_base => (10, "level", 0, 51, "borg_fight_attack.rs:1042", "Attack status level base: existing policy level."),
    attack_status_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:1073", "Attack status danger horizon: existing policy turns."),
    attack_fear_bonus_max_level => (10, "level", 0, 51, "borg_fight_attack.rs:1076", "Attack fear bonus max level: existing policy level."),
    attack_fear_danger_bonus => (20, "danger", 0, 5000, "borg_fight_attack.rs:1077", "Attack fear danger bonus: existing policy danger."),
    attack_arrow_point_blank_divisor => (5, "divisor", 1, 1000, "borg_fight_attack.rs:3972", "The default divisor of a shot or throw at an adjacent non-unique (#485: overridden per class by attack_arrow_point_blank_divisor_<class>)."),
    attack_throw_point_blank_divisor => (0, "divisor (0 = inherit arrow)", 0, 1000, "borg_fight_attack.rs:1125", "#870: divisor for adjacent non-unique throws rated with attack_throw_as_melee. 0 (compiled) inherits the resolved attack_arrow_point_blank_divisor, preserving existing class/profile behavior; 1 removes this policy penalty. Arrows and the focused planner are unchanged."),
    attack_arrow_min_hit_tenths => (8, "tenths", 0, 16, "borg_fight_attack.rs:1140", "Attack arrow min hit tenths: existing policy tenths."),
    attack_mana_unique_min_potions => (3, "count", 0, 255, "borg_fight_attack.rs:1148", "Attack mana unique min potions: existing policy count."),
    attack_mana_unique_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:1150", "Attack mana unique multiplier: existing policy multiplier."),
    attack_nether_point_blank_divisor => (5, "divisor", 1, 1000, "borg_fight_attack.rs:1102", "Attack nether point blank divisor: existing policy divisor."),
    attack_teleport_unique_injury => (60, "percent", 0, 100, "borg_fight_attack.rs:1342", "Attack teleport unique injury: existing policy percent."),
    attack_teleport_unique_danger_tenths => (13, "tenths", 0, 26, "borg_fight_attack.rs:1348", "Attack teleport unique danger tenths: existing policy tenths."),
    attack_teleport_unique_max_depth => (98, "depth", 0, 127, "borg_fight_attack.rs:1348", "Attack teleport unique max depth: existing policy depth."),
    attack_teleport_unique_min_count => (2, "count", 0, 255, "borg_fight_attack.rs:1351", "Attack teleport unique min count: existing policy count."),
    attack_teleport_unique_max_count => (8, "count", 0, 9, "borg_fight_attack.rs:1351", "Attack teleport unique max count: existing policy count."),
    attack_teleport_mage_unique_distance => (2, "grids", 1, 20, "borg_fight_attack.rs:1354", "Attack teleport mage unique distance: existing policy grids."),
    attack_teleport_vault_min_walls => (2, "count", 0, 255, "borg_fight_attack.rs:1390", "Attack teleport vault min walls: existing policy count."),
    attack_teleport_glyph_value => (300, "score", 0, 100000, "borg_fight_attack.rs:1452", "Attack teleport glyph value: existing policy score."),
    attack_teleport_morgoth_value => (100, "score", 0, 100000, "borg_fight_attack.rs:1465", "Attack teleport morgoth value: existing policy score."),
    attack_teleport_low_sp => (35, "mana", 0, 1000, "borg_fight_attack.rs:1481", "Attack teleport low sp: existing policy mana."),
    attack_teleport_low_sp_value => (150, "score", 0, 100000, "borg_fight_attack.rs:1484", "Attack teleport low sp value: existing policy score."),
    attack_turn_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:1107", "Attack turn danger horizon: existing policy turns."),
    attack_status_slow_speed_margin => (5, "speed", 0, 100, "borg_fight_attack.rs:1115", "Attack status slow speed margin: existing policy speed."),
    attack_fear_danger_horizon => (2, "turns", 1, 10, "borg_fight_attack.rs:1177", "Attack fear danger horizon: existing policy turns."),
    attack_fear_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:1179", "Attack fear danger multiplier: existing policy multiplier."),
    attack_crush_monster_level_margin => (5, "level", 0, 51, "borg_fight_attack.rs:1192", "Attack crush monster level margin: existing policy level."),
    attack_sleep_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:1215", "Attack sleep danger horizon: existing policy turns."),
    attack_missile_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:1253", "Attack missile danger horizon: existing policy turns."),
    attack_missile_danger_tenths => (2, "tenths", 0, 10, "borg_fight_attack.rs:1253", "Attack missile danger tenths: existing policy tenths."),
    attack_missile_level_margin => (5, "level", 0, 51, "borg_fight_attack.rs:1255", "Attack missile level margin: existing policy level."),
    attack_missile_weak_max_level => (20, "level", 0, 51, "borg_fight_attack.rs:1261", "Attack missile weak max level: existing policy level."),
    attack_ranged_overkill_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:1275", "Attack ranged overkill multiplier: existing policy multiplier."),
    attack_ranged_unique_multiplier => (3, "multiplier", 0, 10, "borg_fight_attack.rs:1282", "Attack ranged unique multiplier: existing policy multiplier."),
    attack_ranged_town_unique_numerator => (2, "multiplier", 0, 10, "borg_fight_attack.rs:1290", "Attack ranged town unique numerator: existing policy multiplier."),
    attack_ranged_town_unique_divisor => (3, "divisor", 1, 1000, "borg_fight_attack.rs:1290", "Attack ranged town unique divisor: existing policy divisor."),
    attack_ranged_breeder_numerator => (3, "multiplier", 0, 10, "borg_fight_attack.rs:1300", "Attack ranged breeder numerator: existing policy multiplier."),
    attack_ranged_breeder_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:1300", "Attack ranged breeder divisor: existing policy divisor."),
    attack_ranged_summoner_numerator => (3, "multiplier", 0, 10, "borg_fight_attack.rs:1307", "Attack ranged summoner numerator: existing policy multiplier."),
    attack_ranged_summoner_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:1307", "Attack ranged summoner divisor: existing policy divisor."),
    attack_ranged_questor_bonus => (9, "multiplier", 0, 18, "borg_fight_attack.rs:1315", "Attack ranged questor bonus: existing policy multiplier."),
    attack_bolt_max_target_age => (2, "turns", 1, 10, "borg_fight_attack.rs:1803", "Attack bolt max target age: existing policy turns."),
    attack_bolt_unknown_min_walls => (2, "count", 0, 255, "borg_fight_attack.rs:1846", "Attack bolt unknown min walls: existing policy count."),
    attack_bolt_unknown_min_grids => (1, "count", 0, 255, "borg_fight_attack.rs:1846", "Attack bolt unknown min grids: existing policy count."),
    attack_bolt_sleep_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:1876", "Attack bolt sleep danger horizon: existing policy turns."),
    attack_bolt_sleep_danger_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:1879", "Attack bolt sleep danger divisor: existing policy divisor."),
    attack_bolt_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:1794", "Attack bolt danger horizon: existing policy turns."),
    attack_bolt_kill_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:1831", "Attack bolt kill multiplier: existing policy multiplier."),
    attack_destroy_unknown_boots_value => (20, "score", 0, 100000, "borg_fight_attack.rs:1940", "Attack destroy unknown boots value: existing policy score."),
    attack_destroy_unknown_ring_value => (20, "score", 0, 100000, "borg_fight_attack.rs:1946", "Attack destroy unknown ring value: existing policy score."),
    attack_destroy_speed_ring_value => (2000, "score", 0, 100000, "borg_fight_attack.rs:1949", "Attack destroy speed ring value: existing policy score."),
    attack_destroy_valuable_potion_value => (2000, "score", 0, 100000, "borg_fight_attack.rs:1950", "Attack destroy valuable potion value: existing policy score."),
    attack_destroy_unknown_potion_value => (20, "score", 0, 100000, "borg_fight_attack.rs:1953", "Attack destroy unknown potion value: existing policy score."),
    attack_unmapped_fear_divisor => (20, "divisor", 1, 1000, "borg_fight_attack.rs:2021", "Attack unmapped fear divisor: existing policy divisor."),
    attack_ego_ammo_multiplier => (99, "multiplier", 0, 198, "borg_fight_attack.rs:2711", "Attack ego ammo multiplier: existing policy multiplier."),
    attack_throw_max_target_age => (10, "turns", 1, 10, "borg_fight_attack.rs:2841", "Attack throw max target age: existing policy turns."),
    attack_throw_fast_monster_margin => (5, "speed", 0, 100, "borg_fight_attack.rs:2851", "Attack throw fast monster margin: existing policy speed."),
    kill_flow_mode => (0, "mode", 0, 3, "borg_flow_kill.rs:2002", "#691: the hunt (borg_flow_kill, walking to a monster to fight it) removed in steps. 0 = C. 1 = only the targeted hunts: breeders when a hunt for them applies (#503, #558), dangerous town monsters (#619, #637). 2 = no out-of-view call (\"chase old monsters\"). 3 = never hunt; borg_attack still fights what is in reach."),
    attack_throw_min_item_value => (100, "gold", 0, 100000, "borg_fight_attack.rs:3120", "Attack throw min item value: existing policy gold."),
    attack_throw_low_damage => (5, "damage", 0, 5000, "borg_fight_attack.rs:3120", "Attack throw low damage: existing policy damage."),
    attack_confused_by_direction => (0, "flag", 0, 1, "borg_fight_attack.rs:304", "#867 (owner): 1 = a confused Borg (not blind, not hallucinating) may throw, shoot, aim a wand or rod and use a staff at an adjacent monster, besides melee: the engine refuses only spells and scrolls while confused (player_can_cast). An aimed attack (melee included) is rated at the aim chance of player_confuse_dir, 11/32 (a direction key kept 25%, else one of 8 at random), and sent by direction key, never at a target (direction 5 is always redrawn); a staff needs no aim and keeps its full value. The attack chooser then picks the best of them as usual: a weak-armed caster throws oil, a fighter swings; a blow's damage for the fight-or-flee tests stays C's (full). Next to an immobile monster (not a unique) whose blow confuses, while an attack is open, the confusion cure waits and the attack goes first (borg_caution.rs cure_held_for_attack). 0 = C (no item attack while confused)."),
    attack_throw_surplus => (0, "flag", 0, 1, "borg_fight_attack.rs:2969", "#1200: consider known low-value spare throwing weapons dominated by the equipped weapon. Both throw families use the engine-based rating when enabled. 0 keeps the current throw candidates."),
    attack_throw_surplus_max_value => (100, "gold", 0, 100000, "borg_fight_attack.rs:2990", "#1200: maximum value of a spare weapon considered for throwing."),
    attack_throw_as_melee => (0, "flag", 0, 1, "borg_fight_attack.rs:3082", "#568: 1 = a throw is rated on melee's footing: the engine's throw damage (a flask of oil 22, not its 1d4), the engine's hit chance with the throwing skill, no launcher, and melee's danger term (its horizon, full damage, the partial-kill division). 0 = C (the dice average, the bow skill and launcher, the bolt's one-turn danger)."),
    attack_reserve_mage_sp => (6, "mana", 0, 1000, "borg_fight_attack.rs:3312", "Attack reserve mage sp: existing policy mana."),
    attack_reserve_ranger_sp => (22, "mana", 0, 1000, "borg_fight_attack.rs:3313", "Attack reserve ranger sp: existing policy mana."),
    attack_reserve_rogue_sp => (20, "mana", 0, 1000, "borg_fight_attack.rs:3314", "Attack reserve rogue sp: existing policy mana."),
    attack_reserve_priest_sp => (8, "mana", 0, 1000, "borg_fight_attack.rs:3315", "Attack reserve priest sp: existing policy mana."),
    attack_reserve_paladin_sp => (20, "mana", 0, 1000, "borg_fight_attack.rs:3316", "Attack reserve paladin sp: existing policy mana."),
    attack_reserve_necromancer_sp => (10, "mana", 0, 1000, "borg_fight_attack.rs:3317", "Attack reserve necromancer sp: existing policy mana."),
    attack_spell_paranoia_max_level => (2, "level", 0, 51, "borg_fight_attack.rs:3343", "Attack spell paranoia max level: existing policy level."),
    attack_spell_paranoia_chance => (1, "percent", 0, 100, "borg_fight_attack.rs:3344", "Attack spell paranoia chance: existing policy percent."),
    attack_spell_unique_fail => (40, "percent", 0, 100, "borg_fight_attack.rs:3355", "Attack spell unique fail: existing policy percent."),
    attack_spell_fail => (25, "percent", 0, 100, "borg_fight_attack.rs:3355", "Attack spell fail: existing policy percent."),
    attack_primary_teleport_fail => (15, "percent", 0, 100, "borg_fight_attack.rs:3371", "Attack primary teleport fail: existing policy percent."),
    attack_primary_max_level => (30, "level", 0, 51, "borg_fight_attack.rs:3371", "Attack primary max level: existing policy level."),
    attack_spell_low_max_sp => (50, "mana", 0, 1000, "borg_fight_attack.rs:3384", "Attack spell low max sp: existing policy mana."),
    attack_spell_reserve_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:3389", "Attack spell reserve divisor: existing policy divisor."),
    attack_spell_reserve_penalty => (3, "multiplier", 0, 10, "borg_fight_attack.rs:3390", "Attack spell reserve penalty: existing policy multiplier."),
    attack_spell_deep_reserve_divisor => (3, "divisor", 1, 1000, "borg_fight_attack.rs:3394", "Attack spell deep reserve divisor: existing policy divisor."),
    attack_spell_deep_reserve_penalty => (5, "multiplier", 0, 10, "borg_fight_attack.rs:3395", "Attack spell deep reserve penalty: existing policy multiplier."),
    attack_escape_reserve_min_max_sp => (30, "mana", 0, 1000, "borg_fight_attack.rs:3401", "Attack escape reserve min max sp: existing policy mana."),
    attack_escape_reserve_penalty => (750, "multiplier", 0, 1500, "borg_fight_attack.rs:3402", "Attack escape reserve penalty: existing policy multiplier."),
    attack_reserve_spell_max_level => (15, "level", 0, 51, "borg_fight_attack.rs:3532", "Attack reserve spell max level: existing policy level."),
    attack_reserve_spell_fail => (25, "percent", 0, 100, "borg_fight_attack.rs:3547", "Attack reserve spell fail: existing policy percent."),
    attack_reserve_spell_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:3567", "Attack reserve spell danger horizon: existing policy turns."),
    attack_reserve_spell_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:3571", "Attack reserve spell danger multiplier: existing policy multiplier."),
    attack_reserve_spell_near_distance => (7, "grids", 1, 20, "borg_fight_attack.rs:3599", "Attack reserve spell near distance: existing policy grids."),
    attack_reserve_spell_overexert => (1, "flag", 0, 1, "borg_fight_attack.rs:3558", "#980: 1 = the emergency attack spell (borg_attack_aux_spell_bolt_reserve, Magic Missile) is cast beyond the current mana, as in the C Borg; the engine then always faints the Borg (paralysis for 1d(5 x missing SP + 1) turns, past free action; player-spell.c:540-553) and adds 5% fail per missing SP. 0 = never cast it beyond the current SP. 1 retains today's play."),
    attack_reserve_spell_damage_margin => (4, "damage", 0, 5000, "borg_fight_attack.rs:3607", "Attack reserve spell damage margin: existing policy damage."),
    attack_dispel_paranoia_chance => (2, "percent", 0, 100, "borg_fight_attack.rs:3718", "Attack dispel paranoia chance: existing policy percent."),
    attack_dispel_fail => (25, "percent", 0, 100, "borg_fight_attack.rs:3723", "Attack dispel fail: existing policy percent."),
    attack_dispel_reserve_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:3736", "Attack dispel reserve divisor: existing policy divisor."),
    attack_dispel_reserve_penalty => (3, "multiplier", 0, 10, "borg_fight_attack.rs:3737", "Attack dispel reserve penalty: existing policy multiplier."),
    attack_dispel_deep_reserve_divisor => (3, "divisor", 1, 1000, "borg_fight_attack.rs:3741", "Attack dispel deep reserve divisor: existing policy divisor."),
    attack_dispel_deep_reserve_penalty => (5, "multiplier", 0, 10, "borg_fight_attack.rs:3742", "Attack dispel deep reserve penalty: existing policy multiplier."),
    attack_dispel_escape_min_max_sp => (30, "mana", 0, 1000, "borg_fight_attack.rs:3747", "Attack dispel escape min max sp: existing policy mana."),
    attack_dispel_escape_penalty => (750, "multiplier", 0, 1500, "borg_fight_attack.rs:3748", "Attack dispel escape penalty: existing policy multiplier."),
    attack_wand_paranoia_chance => (2, "percent", 0, 100, "borg_fight_attack.rs:3783", "Attack wand paranoia chance: existing policy percent."),
    attack_wand_charge_penalty => (50, "score", 0, 100000, "borg_fight_attack.rs:3796", "Attack wand charge penalty: existing policy score."),
    attack_rod_paranoia_chance => (2, "percent", 0, 100, "borg_fight_attack.rs:3821", "Attack rod paranoia chance: existing policy percent."),
    attack_rod_max_failure => (500, "permille", 0, 1000, "borg_fight_attack.rs:3827", "Attack rod max failure: existing policy permille."),
    attack_unknown_device_paranoia_chance => (2, "percent", 0, 100, "borg_fight_attack.rs:3892", "Attack unknown device paranoia chance: existing policy percent."),
    attack_unknown_wand_max_failure => (500, "permille", 0, 1000, "borg_fight_attack.rs:3911", "Attack unknown wand max failure: existing policy permille."),
    attack_unknown_wand_penalty_min_level => (5, "level", 0, 51, "borg_fight_attack.rs:3923", "Attack unknown wand penalty min level: existing policy level."),
    attack_unknown_wand_penalty => (5, "score", 0, 100000, "borg_fight_attack.rs:3924", "Attack unknown wand penalty: existing policy score."),
    attack_unknown_wand_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:3935", "Attack unknown wand danger horizon: existing policy turns."),
    attack_unknown_wand_danger_tenths => (7, "tenths", 0, 14, "borg_fight_attack.rs:3935", "Attack unknown wand danger tenths: existing policy tenths."),
    attack_unknown_wand_emergency_value => (999, "score", 0, 100000, "borg_fight_attack.rs:3937", "Attack unknown wand emergency value: existing policy score."),
    attack_activation_paranoia_chance => (2, "percent", 0, 100, "borg_fight_attack.rs:3992", "Attack activation paranoia chance: existing policy percent."),
    attack_ring_paranoia_chance => (2, "percent", 0, 100, "borg_fight_attack.rs:4037", "Attack ring paranoia chance: existing policy percent."),
    attack_dragon_paranoia_chance => (2, "percent", 0, 100, "borg_fight_attack.rs:4077", "Attack dragon paranoia chance: existing policy percent."),
    attack_leap_min_target_score => (10, "score", 0, 100000, "borg_fight_attack.rs:4203", "Attack leap min target score: existing policy score."),
    attack_leap_max_distance => (4, "grids", 1, 4, "borg_fight_attack.rs:4215", "Attack leap max distance: existing policy grids."),
    attack_vampire_max_distance => (20, "grids", 1, 20, "borg_fight_attack.rs:4455", "Attack vampire max distance: existing policy grids."),
    attack_vampire_sleep_danger_horizon => (1, "turns", 1, 10, "borg_fight_attack.rs:4523", "Attack vampire sleep danger horizon: existing policy turns."),
    attack_vampire_sleep_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:4525", "Attack vampire sleep danger multiplier: existing policy multiplier."),
    attack_vampire_danger_horizon => (2, "turns", 1, 10, "borg_fight_attack.rs:4531", "Attack vampire danger horizon: existing policy turns."),
    attack_vampire_partial_min_level => (15, "level", 0, 51, "borg_fight_attack.rs:4534", "Attack vampire partial min level: existing policy level."),
    attack_vampire_partial_danger_divisor => (10, "divisor", 1, 1000, "borg_fight_attack.rs:4535", "Attack vampire partial danger divisor: existing policy divisor."),
    attack_flag_danger_horizon => (4, "turns", 1, 10, "borg_fight_attack.rs:4564", "Attack flag danger horizon: existing policy turns."),
    attack_crush_hp_margin => (10, "hp", 0, 10000, "borg_fight_attack.rs:4582", "Attack crush hp margin: existing policy hp."),
    attack_crush_hp_level_multiplier => (4, "multiplier", 0, 10, "borg_fight_attack.rs:4582", "Attack crush hp level multiplier: existing policy multiplier."),
    attack_crush_post_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_attack.rs:4594", "Attack crush post danger multiplier: existing policy multiplier."),
    attack_crush_min_post_hp => (50, "hp", 0, 10000, "borg_fight_attack.rs:4594", "Attack crush min post hp: existing policy hp."),
    attack_crush_reserve_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:4604", "Attack crush reserve divisor: existing policy divisor."),
    attack_crush_reserve_penalty => (10, "multiplier", 0, 20, "borg_fight_attack.rs:4605", "Attack crush reserve penalty: existing policy multiplier."),
    attack_trance_reserve_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:4650", "Attack trance reserve divisor: existing policy divisor."),
    attack_trance_reserve_penalty => (10, "multiplier", 0, 20, "borg_fight_attack.rs:4651", "Attack trance reserve penalty: existing policy multiplier."),
    attack_holy_word_min_loss => (300, "hp", 0, 10000, "borg_fight_attack.rs:4875", "Attack holy word min loss: existing policy hp."),
    attack_curse_min_hp => (120, "hp", 0, 10000, "borg_fight_attack.rs:4988", "Attack curse min hp: existing policy hp."),
    attack_holy_staff_hp_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:4961", "Attack holy staff hp divisor: existing policy divisor."),
    attack_holy_staff_hurt_value => (500, "score", 0, 100000, "borg_fight_attack.rs:4961", "Attack holy staff hurt value: existing policy score."),
    attack_holy_staff_value => (120, "score", 0, 100000, "borg_fight_attack.rs:4961", "Attack holy staff value: existing policy score."),
    attack_holy_activation_hp_divisor => (2, "divisor", 1, 1000, "borg_fight_attack.rs:4846", "Attack holy activation hp divisor: existing policy divisor."),
    attack_holy_activation_hurt_value => (500, "score", 0, 100000, "borg_fight_attack.rs:4846", "Attack holy activation hurt value: existing policy score."),
    attack_holy_activation_value => (120, "score", 0, 100000, "borg_fight_attack.rs:4846", "Attack holy activation value: existing policy score."),
    attack_max_target_age => (2, "turns", 1, 10, "borg_fight_attack.rs:582", "Attack max target age: existing policy turns."),
    attack_town_mage_max_level => (10, "level", 0, 51, "borg_fight_attack.rs:5433", "Attack town mage max level: existing policy level."),
    attack_fast_monster_distance => (2, "grids", 1, 20, "borg_fight_attack.rs:5449", "Attack fast monster distance: existing policy grids."),
    attack_scary_breeder_max_depth => (5, "depth", 0, 127, "borg_fight_attack.rs:5458", "Attack scary breeder max depth: existing policy depth."),
    attack_scary_max_level_time => (2000, "turns", 1, 2000, "borg_fight_attack.rs:5460", "Attack scary max level time: existing policy turns."),
    attack_scary_max_town_time => (3000, "turns", 1, 3000, "borg_fight_attack.rs:5460", "Attack scary max town time: existing policy turns."),
    attack_scary_depth_multiplier => (4, "multiplier", 0, 10, "borg_fight_attack.rs:5469", "Attack scary depth multiplier: existing policy multiplier."),
    attack_scary_min_level => (10, "level", 0, 51, "borg_fight_attack.rs:5469", "Attack scary min level: existing policy level."),
    attack_speed_fight_danger_tenths => (0, "tenths", 0, 1000, "borg_fight_attack.rs:6380", "Danger from the monsters that can move, in tenths of avoidance, above which a carried Potion of Speed is drunk before a fight the Borg has committed to (#469); 0 = off."),

    // borg-reborn-rs (#241): defend source at 5d9abb8f.
    defend_bless_fail => (25, "percent", 0, 100, "borg_fight_defend.rs:318", "Defend bless fail: existing policy percent."),
    defend_bless_max_target_age => (5, "turns", 1, 10, "borg_fight_defend.rs:332", "Defend bless max target age: existing policy turns."),
    defend_bless_no_rest_duration => (11000, "duration", 0, 32767, "borg_fight_defend.rs:357", "Defend bless no rest duration: existing policy duration."),
    defend_speed_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:420", "Defend speed fail base: existing policy percent."),
    defend_speed_fail_mid_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:420", "Defend speed fail mid adjustment: existing policy percent."),
    defend_speed_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:420", "Defend speed fail low adjustment: existing policy percent."),
    defend_grim_purpose_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:562", "Defend grim purpose fail base: existing policy percent."),
    defend_grim_purpose_fail_mid_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:562", "Defend grim purpose fail mid adjustment: existing policy percent."),
    defend_grim_purpose_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:562", "Defend grim purpose fail low adjustment: existing policy percent."),
    defend_grim_purpose_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:554", "Defend grim purpose improvement divisor: existing policy divisor."),
    defend_grim_purpose_score_bonus => (2, "score", 0, 100000, "borg_fight_defend.rs:557", "Defend grim purpose score bonus: existing policy score."),
    defend_grim_purpose_no_rest_duration => (13000, "duration", 0, 32767, "borg_fight_defend.rs:565", "Defend grim purpose no rest duration: existing policy duration."),
    defend_resist_fecap_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:630", "Defend resist fecap improvement divisor: existing policy divisor."),
    defend_resist_fecap_score_bonus => (2, "score", 0, 100000, "borg_fight_defend.rs:633", "Defend resist fecap score bonus: existing policy score."),
    defend_resist_fecap_no_rest_duration => (21000, "duration", 0, 32767, "borg_fight_defend.rs:643", "Defend resist fecap no rest duration: existing policy duration."),
    defend_resist_f_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:695", "Defend resist f fail base: existing policy percent."),
    defend_resist_f_fail_mid_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:695", "Defend resist f fail mid adjustment: existing policy percent."),
    defend_resist_f_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:695", "Defend resist f fail low adjustment: existing policy percent."),
    defend_resist_f_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:698", "Defend resist f improvement divisor: existing policy divisor."),
    defend_resist_f_no_rest_duration => (21000, "duration", 0, 32767, "borg_fight_defend.rs:721", "Defend resist f no rest duration: existing policy duration."),
    defend_resist_c_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:771", "Defend resist c fail base: existing policy percent."),
    defend_resist_c_fail_mid_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:771", "Defend resist c fail mid adjustment: existing policy percent."),
    defend_resist_c_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:771", "Defend resist c fail low adjustment: existing policy percent."),
    defend_resist_c_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:775", "Defend resist c improvement divisor: existing policy divisor."),
    defend_resist_c_no_rest_duration => (21000, "duration", 0, 32767, "borg_fight_defend.rs:798", "Defend resist c no rest duration: existing policy duration."),
    defend_resist_a_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:848", "Defend resist a fail base: existing policy percent."),
    defend_resist_a_fail_mid_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:848", "Defend resist a fail mid adjustment: existing policy percent."),
    defend_resist_a_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:848", "Defend resist a fail low adjustment: existing policy percent."),
    defend_resist_a_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:844", "Defend resist a improvement divisor: existing policy divisor."),
    defend_resist_a_no_rest_duration => (21000, "duration", 0, 32767, "borg_fight_defend.rs:871", "Defend resist a no rest duration: existing policy duration."),
    defend_resist_e_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:920", "Defend resist e fail base: existing policy percent."),
    defend_resist_e_fail_mid_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:920", "Defend resist e fail mid adjustment: existing policy percent."),
    defend_resist_e_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:920", "Defend resist e fail low adjustment: existing policy percent."),
    defend_resist_e_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:917", "Defend resist e improvement divisor: existing policy divisor."),
    defend_resist_e_no_rest_duration => (21000, "duration", 0, 32767, "borg_fight_defend.rs:945", "Defend resist e no rest duration: existing policy duration."),
    defend_resist_p_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:994", "Defend resist p fail base: existing policy percent."),
    defend_resist_p_fail_mid_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:994", "Defend resist p fail mid adjustment: existing policy percent."),
    defend_resist_p_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:994", "Defend resist p fail low adjustment: existing policy percent."),
    defend_resist_p_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:990", "Defend resist p improvement divisor: existing policy divisor."),
    defend_resist_p_no_rest_duration => (21000, "duration", 0, 32767, "borg_fight_defend.rs:1006", "Defend resist p no rest duration: existing policy duration."),
    defend_prot_evil_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:1059", "Defend prot evil fail base: existing policy percent."),
    defend_prot_evil_fail_mid_adjustment => (5, "percent", 0, 100, "borg_fight_defend.rs:1059", "Defend prot evil fail mid adjustment: existing policy percent."),
    defend_prot_evil_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:1059", "Defend prot evil fail low adjustment: existing policy percent."),
    defend_prot_evil_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:1069", "Defend prot evil improvement divisor: existing policy divisor."),
    defend_shield_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:1126", "Defend shield improvement divisor: existing policy divisor."),
    defend_shield_no_rest_duration => (2000, "duration", 0, 32767, "borg_fight_defend.rs:1139", "Defend shield no rest duration: existing policy duration."),
    defend_tele_away_fail => (50, "percent", 0, 100, "borg_fight_defend.rs:1179", "Defend tele away fail: existing policy percent."),
    defend_tele_away_max_target_age => (2, "turns", 1, 10, "borg_fight_defend.rs:1235", "Defend tele away max target age: existing policy turns."),
    defend_hero_fail => (15, "percent", 0, 100, "borg_fight_defend.rs:1361", "Defend hero fail: existing policy percent."),
    defend_hero_no_rest_duration => (10000, "duration", 0, 32767, "borg_fight_defend.rs:1374", "Defend hero no rest duration: existing policy duration."),
    defend_regen_fail => (15, "percent", 0, 100, "borg_fight_defend.rs:1410", "Defend regen fail: existing policy percent."),
    defend_regen_no_rest_duration => (10000, "duration", 0, 32767, "borg_fight_defend.rs:1416", "Defend regen no rest duration: existing policy duration."),
    // #1211: the recorded Priest (full HP, SP 0-2/47, danger 20-25 from remembered orc archers) cast every regenerated SP back into Heroism while its recovery walk paced for a grid it was never allowed to rest on.
    defend_prep_mana_starved => (0, "flag", 0, 1, "borg_fight_defend.rs:220", "#1211: 1 = the some-danger prep spells (Hero, Berserk Strength, Rapid Regeneration — Bless and Smite Evil already require a monster at hand) are not cast while at full HP with SP at or below MAXSP/4, the level that arms the recovery walk: that mana is what the walk is for. The potion and activation fallbacks are untouched (nothing behind them). 0 retains C's casts."),
    defend_berserk_fail => (15, "percent", 0, 100, "borg_fight_defend.rs:1454", "Defend berserk fail: existing policy percent."),
    defend_near_evil_max_target_age => (2, "turns", 1, 10, "borg_fight_defend.rs:1516", "Defend near evil max target age: existing policy turns."),
    defend_smite_evil_fail => (15, "percent", 0, 100, "borg_fight_defend.rs:1549", "Defend smite evil fail: existing policy percent."),
    defend_glyph_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:1620", "Defend glyph fail base: existing policy percent."),
    defend_glyph_fail_mid_adjustment => (5, "percent", 0, 100, "borg_fight_defend.rs:1620", "Defend glyph fail mid adjustment: existing policy percent."),
    defend_glyph_fail_low_adjustment => (20, "percent", 0, 100, "borg_fight_defend.rs:1620", "Defend glyph fail low adjustment: existing policy percent."),
    defend_glyph_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:1622", "Defend glyph improvement divisor: existing policy divisor."),
    defend_create_door_fail_base => (30, "percent", 0, 100, "borg_fight_defend.rs:1700", "Defend create door fail base: existing policy percent."),
    defend_create_door_fail_mid_adjustment => (5, "percent", 0, 100, "borg_fight_defend.rs:1700", "Defend create door fail mid adjustment: existing policy percent."),
    defend_create_door_fail_low_adjustment => (20, "percent", 0, 100, "borg_fight_defend.rs:1700", "Defend create door fail low adjustment: existing policy percent."),
    defend_create_door_improvement_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:1725", "Defend create door improvement divisor: existing policy divisor."),
    defend_genocide_fail_base => (25, "percent", 0, 100, "borg_fight_defend.rs:1929", "Defend genocide fail base: existing policy percent."),
    defend_genocide_fail_mid_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:1929", "Defend genocide fail mid adjustment: existing policy percent."),
    defend_genocide_fail_low_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:1929", "Defend genocide fail low adjustment: existing policy percent."),
    defend_earthquake_improvement_divisor => (5, "divisor", 1, 1000, "borg_fight_defend.rs:2359", "Defend earthquake improvement divisor: existing policy divisor."),
    defend_banishment_fail => (15, "percent", 0, 100, "borg_fight_defend.rs:2590", "Defend banishment fail: existing policy percent."),
    defend_inviso_fail => (25, "percent", 0, 100, "borg_fight_defend.rs:2756", "Defend inviso fail: existing policy percent."),
    defend_inviso_no_rest_duration => (18000, "duration", 0, 32767, "borg_fight_defend.rs:2835", "Defend inviso no rest duration: existing policy duration."),
    defend_inviso_potion_no_rest_duration => (16000, "duration", 0, 32767, "borg_fight_defend.rs:2841", "Defend inviso potion no rest duration: existing policy duration."),
    defend_tele_away_morgoth_fail => (40, "percent", 0, 100, "borg_fight_defend.rs:3172", "Defend tele away morgoth fail: existing policy percent."),
    defend_tele_away_morgoth_max_target_age => (2, "turns", 1, 10, "borg_fight_defend.rs:3221", "Defend tele away morgoth max target age: existing policy turns."),
    defend_banishment_morgoth_fail => (50, "percent", 0, 100, "borg_fight_defend.rs:3323", "Defend banishment morgoth fail: existing policy percent."),
    defend_banishment_morgoth_max_target_age => (2, "turns", 1, 10, "borg_fight_defend.rs:3362", "Defend banishment morgoth max target age: existing policy turns."),
    defend_light_morgoth_fail => (50, "percent", 0, 100, "borg_fight_defend.rs:3444", "Defend light morgoth fail: existing policy percent."),
    defend_light_morgoth_max_target_age => (2, "turns", 1, 10, "borg_fight_defend.rs:3489", "Defend light morgoth max target age: existing policy turns."),
    defend_scaled_fail_high_adjustment => (19, "percent", 0, 100, "borg_fight_defend.rs:170", "Defend scaled fail high adjustment: existing policy percent."),
    defend_scaled_fail_mid_numerator => (2, "multiplier", 0, 10, "borg_fight_defend.rs:171", "Defend scaled fail mid numerator: existing policy multiplier."),
    defend_scaled_fail_mid_divisor => (3, "divisor", 1, 1000, "borg_fight_defend.rs:171", "Defend scaled fail mid divisor: existing policy divisor."),
    defend_scaled_fail_low_divisor => (3, "divisor", 1, 1000, "borg_fight_defend.rs:173", "Defend scaled fail low divisor: existing policy divisor."),
    defend_limit_unique_numerator => (2, "multiplier", 0, 10, "borg_fight_defend.rs:183", "Defend limit unique numerator: existing policy multiplier."),
    defend_limit_unique_divisor => (3, "divisor", 1, 1000, "borg_fight_defend.rs:183", "Defend limit unique divisor: existing policy divisor."),
    defend_limit_divisor => (2, "divisor", 1, 1000, "borg_fight_defend.rs:185", "Defend limit divisor: existing policy divisor."),
    defend_some_danger_divisor => (10, "divisor", 1, 1000, "borg_fight_defend.rs:199", "Defend some danger divisor: existing policy divisor."),
    defend_some_danger_unique_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:199", "Defend some danger unique tenths: existing policy tenths."),
    defend_some_danger_tenths => (5, "tenths", 0, 10, "borg_fight_defend.rs:199", "Defend some danger tenths: existing policy tenths."),
    defend_danger_horizon => (1, "turns", 1, 10, "borg_fight_defend.rs:249", "Defend danger horizon: existing policy turns."),
    defend_bless_near_distance => (3, "grids", 1, 20, "borg_fight_defend.rs:337", "Defend bless near distance: existing policy grids."),
    defend_bless_danger_divisor => (12, "divisor", 1, 1000, "borg_fight_defend.rs:347", "Defend bless danger divisor: existing policy divisor."),
    defend_bless_max_level => (15, "level", 0, 51, "borg_fight_defend.rs:347", "Defend bless max level: existing policy level."),
    defend_bless_max_danger_divisor => (2, "divisor", 1, 1000, "borg_fight_defend.rs:347", "Defend bless max danger divisor: existing policy divisor."),
    defend_speed_morgoth_tenths => (3, "tenths", 0, 10, "borg_fight_defend.rs:443", "Defend speed morgoth tenths: existing policy tenths."),
    defend_speed_breeder_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:448", "Defend speed breeder tenths: existing policy tenths."),
    defend_speed_questor_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:452", "Defend speed questor tenths: existing policy tenths."),
    defend_speed_sauron_tenths => (6, "tenths", 0, 12, "borg_fight_defend.rs:456", "Defend speed sauron tenths: existing policy tenths."),
    defend_speed_unique_tenths => (6, "tenths", 0, 12, "borg_fight_defend.rs:468", "Defend speed unique tenths: existing policy tenths."),
    defend_speed_morgoth_fight_tenths => (5, "tenths", 0, 10, "borg_fight_defend.rs:473", "Defend speed morgoth fight tenths: existing policy tenths."),
    defend_speed_no_unique_min_depth => (97, "depth", 0, 127, "borg_fight_defend.rs:477", "Defend speed no unique min depth: existing policy depth."),
    defend_speed_good_unique_numerator => (2, "multiplier", 0, 10, "borg_fight_defend.rs:485", "Defend speed good unique numerator: existing policy multiplier."),
    defend_speed_good_unique_divisor => (3, "divisor", 1, 1000, "borg_fight_defend.rs:485", "Defend speed good unique divisor: existing policy divisor."),
    defend_speed_good_divisor => (2, "divisor", 1, 1000, "borg_fight_defend.rs:485", "Defend speed good divisor: existing policy divisor."),
    defend_speed_good_min_danger_divisor => (5, "divisor", 1, 1000, "borg_fight_defend.rs:485", "Defend speed good min danger divisor: existing policy divisor."),
    defend_speed_unique_numerator => (2, "multiplier", 0, 10, "borg_fight_defend.rs:486", "Defend speed unique numerator: existing policy multiplier."),
    defend_speed_unique_divisor => (3, "divisor", 1, 1000, "borg_fight_defend.rs:486", "Defend speed unique divisor: existing policy divisor."),
    defend_speed_divisor => (3, "divisor", 1, 1000, "borg_fight_defend.rs:486", "Defend speed divisor: existing policy divisor."),
    defend_speed_min_danger_divisor => (7, "divisor", 1, 1000, "borg_fight_defend.rs:486", "Defend speed min danger divisor: existing policy divisor."),
    defend_speed_no_rest_per_level => (1000, "turns", 1, 2000, "borg_fight_defend.rs:496", "Defend speed no rest per level: existing policy turns."),
    defend_prayer_unique_tenths => (8, "tenths", 0, 16, "borg_fight_defend.rs:619", "Defend prayer unique tenths: existing policy tenths."),
    defend_prayer_bonus_min_level => (45, "level", 0, 51, "borg_fight_defend.rs:624", "Defend prayer bonus min level: existing policy level."),
    defend_prayer_level_tenths => (8, "tenths", 0, 16, "borg_fight_defend.rs:625", "Defend prayer level tenths: existing policy tenths."),
    defend_resist_fire_unique_tenths => (8, "tenths", 0, 16, "borg_fight_defend.rs:693", "Defend resist fire unique tenths: existing policy tenths."),
    defend_resist_cold_unique_tenths => (8, "tenths", 0, 16, "borg_fight_defend.rs:770", "Defend resist cold unique tenths: existing policy tenths."),
    defend_resist_all_no_rest_per_level => (1000, "turns", 1, 2000, "borg_fight_defend.rs:1085", "Defend resist all no rest per level: existing policy turns."),
    defend_protection_evil_unique_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:1148", "Defend protection evil unique tenths: existing policy tenths."),
    defend_shield_saving_danger_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:1190", "Defend shield saving danger tenths: existing policy tenths."),
    defend_shield_saving_min_sp => (30, "mana", 0, 1000, "borg_fight_defend.rs:1190", "Defend shield saving min sp: existing policy mana."),
    defend_shield_low_danger_tenths => (5, "tenths", 0, 10, "borg_fight_defend.rs:1193", "Defend shield low danger tenths: existing policy tenths."),
    defend_shield_min_danger_tenths => (4, "tenths", 0, 10, "borg_fight_defend.rs:1198", "Defend shield min danger tenths: existing policy tenths."),
    defend_shield_high_danger_multiplier => (3, "multiplier", 0, 10, "borg_fight_defend.rs:1203", "Defend shield high danger multiplier: existing policy multiplier."),
    defend_shield_high_fail_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:1204", "Defend shield high fail adjustment: existing policy percent."),
    defend_shield_mid_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_defend.rs:1178", "Defend shield mid danger multiplier: existing policy multiplier."),
    defend_shield_mid_fail_adjustment => (5, "percent", 0, 100, "borg_fight_defend.rs:1180", "Defend shield mid fail adjustment: existing policy percent."),
    defend_shield_remnant_danger_divisor => (2, "divisor", 1, 1000, "borg_fight_defend.rs:1291", "Defend shield remnant danger divisor: existing policy divisor."),
    defend_berserk_min_max_hp => (100, "hp", 0, 10000, "borg_fight_defend.rs:1423", "Defend berserk min max hp: existing policy hp."),
    defend_hero_value => (5, "score", 0, 100000, "borg_fight_defend.rs:1486", "Defend hero value: existing policy score."),
    defend_regeneration_near_distance => (3, "grids", 1, 20, "borg_fight_defend.rs:1534", "Defend regeneration near distance: existing policy grids."),
    defend_regeneration_value => (5, "score", 0, 100000, "borg_fight_defend.rs:1570", "Defend regeneration value: existing policy score."),
    defend_create_doors_max_bad_grids => (6, "count", 0, 8, "borg_fight_defend.rs:1713", "Defend create doors max bad grids: existing policy count."),
    defend_mass_banish_fail => (40, "percent", 0, 100, "borg_fight_defend.rs:1788", "Defend mass banish fail: existing policy percent."),
    defend_mass_banish_min_danger_tenths => (12, "tenths", 0, 24, "borg_fight_defend.rs:1797", "Defend mass banish min danger tenths: existing policy tenths."),
    defend_mass_banish_distance => (20, "grids", 1, 20, "borg_fight_defend.rs:1789", "Defend mass banish distance: existing policy grids."),
    defend_mass_banish_self_harm_tenths => (12, "tenths", 0, 24, "borg_fight_defend.rs:1815", "Defend mass banish self harm tenths: existing policy tenths."),
    defend_mass_banish_morgoth_min_count => (8, "count", 0, 255, "borg_fight_defend.rs:1823", "Defend mass banish morgoth min count: existing policy count."),
    defend_mass_banish_morgoth_tenths => (6, "tenths", 0, 12, "borg_fight_defend.rs:1824", "Defend mass banish morgoth tenths: existing policy tenths."),
    defend_banish_min_hp => (75, "hp", 0, 10000, "borg_fight_defend.rs:1933", "Defend banish min hp: existing policy hp."),
    defend_banish_threat_min_hp => (375, "hp", 0, 10000, "borg_fight_defend.rs:2045", "Defend banish threat min hp: existing policy hp."),
    defend_banish_threat_hp_multiplier => (3, "multiplier", 0, 10, "borg_fight_defend.rs:2050", "Defend banish threat hp multiplier: existing policy multiplier."),
    defend_banish_threat_self_harm_tenths => (12, "tenths", 0, 24, "borg_fight_defend.rs:2056", "Defend banish threat self harm tenths: existing policy tenths."),
    defend_banish_threat_min_count => (75, "count", 0, 255, "borg_fight_defend.rs:2062", "Defend banish threat min count: existing policy count."),
    defend_banish_threat_danger_divisor => (5, "divisor", 1, 1000, "borg_fight_defend.rs:2067", "Defend banish threat danger divisor: existing policy divisor."),
    defend_banish_danger_self_harm_tenths => (12, "tenths", 0, 24, "borg_fight_defend.rs:2094", "Defend banish danger self harm tenths: existing policy tenths."),
    defend_banish_min_danger_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:2102", "Defend banish min danger tenths: existing policy tenths."),
    defend_banish_deep_depth => (75, "depth", 0, 127, "borg_fight_defend.rs:2102", "Defend banish deep depth: existing policy depth."),
    defend_banish_deep_danger_tenths => (6, "tenths", 0, 12, "borg_fight_defend.rs:2102", "Defend banish deep danger tenths: existing policy tenths."),
    defend_banish_remnant_divisor => (2, "divisor", 1, 1000, "borg_fight_defend.rs:2107", "Defend banish remnant divisor: existing policy divisor."),
    defend_banish_nasties_min_hp_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:2243", "Defend banish nasties min hp tenths: existing policy tenths."),
    defend_banish_nasties_min_hp => (250, "hp", 0, 10000, "borg_fight_defend.rs:2243", "Defend banish nasties min hp: existing policy hp."),
    defend_banish_nasties_min_depth => (25, "depth", 0, 127, "borg_fight_defend.rs:2243", "Defend banish nasties min depth: existing policy depth."),
    defend_banish_nasties_danger_divisor => (4, "divisor", 1, 1000, "borg_fight_defend.rs:2253", "Defend banish nasties danger divisor: existing policy divisor."),
    defend_banish_nasties_fail => (35, "percent", 0, 100, "borg_fight_defend.rs:2257", "Defend banish nasties fail: existing policy percent."),
    defend_banish_nasties_value => (10, "score", 0, 100000, "borg_fight_defend.rs:2250", "Defend banish nasties value: existing policy score."),
    defend_quake_fail => (35, "percent", 0, 100, "borg_fight_defend.rs:2356", "Defend quake fail: existing policy percent."),
    defend_quake_min_danger_tenths => (6, "tenths", 0, 12, "borg_fight_defend.rs:2332", "Defend quake min danger tenths: existing policy tenths."),
    defend_quake_min_monster_distance => (2, "grids", 1, 20, "borg_fight_defend.rs:2343", "Defend quake min monster distance: existing policy grids."),
    defend_quake_high_threat_count => (4, "count", 0, 255, "borg_fight_defend.rs:2352", "Defend quake high threat count: existing policy count."),
    defend_quake_high_danger_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:2352", "Defend quake high danger tenths: existing policy tenths."),
    defend_quake_high_reduction_divisor => (3, "divisor", 1, 1000, "borg_fight_defend.rs:2353", "Defend quake high reduction divisor: existing policy divisor."),
    defend_quake_mid_threat_count => (3, "count", 0, 255, "borg_fight_defend.rs:2355", "Defend quake mid threat count: existing policy count."),
    defend_quake_mid_danger_tenths => (7, "tenths", 0, 14, "borg_fight_defend.rs:2355", "Defend quake mid danger tenths: existing policy tenths."),
    defend_quake_mid_reduction_tenths => (6, "tenths", 0, 12, "borg_fight_defend.rs:2356", "Defend quake mid reduction tenths: existing policy tenths."),
    defend_destruction_value => (500, "score", 0, 100000, "borg_fight_defend.rs:2421", "Defend destruction value: existing policy score."),
    defend_destruction_min_danger_tenths => (8, "tenths", 0, 16, "borg_fight_defend.rs:2406", "Defend destruction min danger tenths: existing policy tenths."),
    defend_destruction_low_hp_min_depth => (90, "depth", 0, 127, "borg_fight_defend.rs:2406", "Defend destruction low hp min depth: existing policy depth."),
    defend_destruction_deep_low_hp => (300, "hp", 0, 10000, "borg_fight_defend.rs:2406", "Defend destruction deep low hp: existing policy hp."),
    defend_destruction_unique_max_count => (4, "count", 0, 9, "borg_fight_defend.rs:2427", "Defend destruction unique max count: existing policy count."),
    defend_destruction_teleport_min_hp => (275, "hp", 0, 10000, "borg_fight_defend.rs:2428", "Defend destruction teleport min hp: existing policy hp."),
    defend_destruction_teleport_chance => (75, "percent", 0, 100, "borg_fight_defend.rs:2429", "Defend destruction teleport chance: existing policy percent."),
    defend_destruction_teleport_horizon => (2, "turns", 1, 10, "borg_fight_defend.rs:2429", "Defend destruction teleport horizon: existing policy turns."),
    defend_destruction_teleport_min_escapes => (2, "count", 0, 255, "borg_fight_defend.rs:2435", "Defend destruction teleport min escapes: existing policy count."),
    defend_destruction_fail => (55, "percent", 0, 100, "borg_fight_defend.rs:2440", "Defend destruction fail: existing policy percent."),
    defend_destruction_emergency_danger_multiplier => (4, "multiplier", 0, 10, "borg_fight_defend.rs:2448", "Defend destruction emergency danger multiplier: existing policy multiplier."),
    defend_destruction_emergency_hp => (150, "hp", 0, 10000, "borg_fight_defend.rs:2448", "Defend destruction emergency hp: existing policy hp."),
    defend_destruction_few_unique_count => (2, "count", 0, 9, "borg_fight_defend.rs:2465", "Defend destruction few unique count: existing policy count."),
    defend_destruction_few_unique_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_defend.rs:2465", "Defend destruction few unique danger multiplier: existing policy multiplier."),
    defend_teleport_level_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_defend.rs:2524", "Defend teleport level danger multiplier: existing policy multiplier."),
    defend_teleport_level_safe_chance => (65, "percent", 0, 100, "borg_fight_defend.rs:2546", "Defend teleport level safe chance: existing policy percent."),
    defend_teleport_level_safe_horizon => (2, "turns", 1, 10, "borg_fight_defend.rs:2546", "Defend teleport level safe horizon: existing policy turns."),
    defend_teleport_level_min_escapes => (2, "count", 0, 255, "borg_fight_defend.rs:2519", "Defend teleport level min escapes: existing policy count."),
    defend_teleport_level_fail => (55, "percent", 0, 100, "borg_fight_defend.rs:2524", "Defend teleport level fail: existing policy percent."),
    defend_banish_evil_min_danger_divisor => (10, "divisor", 1, 1000, "borg_fight_defend.rs:2591", "Defend banish evil min danger divisor: existing policy divisor."),
    defend_banish_evil_high_danger_multiplier => (4, "multiplier", 0, 10, "borg_fight_defend.rs:2596", "Defend banish evil high danger multiplier: existing policy multiplier."),
    defend_banish_evil_fail_adjustment => (10, "percent", 0, 100, "borg_fight_defend.rs:2597", "Defend banish evil fail adjustment: existing policy percent."),
    defend_banish_evil_artifact_min_hp => (100, "hp", 0, 10000, "borg_fight_defend.rs:2605", "Defend banish evil artifact min hp: existing policy hp."),
    defend_banish_evil_unique_injury => (60, "percent", 0, 100, "borg_fight_defend.rs:2649", "Defend banish evil unique injury: existing policy percent."),
    defend_banish_evil_sauron_hp => (250, "hp", 0, 10000, "borg_fight_defend.rs:2703", "Defend banish evil sauron hp: existing policy hp."),
    defend_banish_evil_morgoth_hp => (350, "hp", 0, 10000, "borg_fight_defend.rs:2706", "Defend banish evil morgoth hp: existing policy hp."),
    defend_see_invis_max_age => (45, "turns", 1, 90, "borg_fight_defend.rs:2763", "Defend see invis max age: existing policy turns."),
    defend_see_invis_max_danger_multiplier => (2, "multiplier", 0, 10, "borg_fight_defend.rs:2768", "Defend see invis max danger multiplier: existing policy multiplier."),
    defend_see_invis_value => (10, "score", 0, 100000, "borg_fight_defend.rs:2755", "Defend see invis value: existing policy score."),
    defend_see_invis_short_duration => (3000, "duration", 0, 32767, "borg_fight_defend.rs:2830", "Defend see invis short duration: existing policy duration."),
    defend_see_invis_spell_duration => (18000, "duration", 0, 32767, "borg_fight_defend.rs:2834", "Defend see invis spell duration: existing policy duration."),
    defend_see_invis_potion_duration => (30000, "duration", 0, 32767, "borg_fight_defend.rs:2840", "Defend see invis potion duration: existing policy duration."),
    defend_light_beam_max_invis_age => (48, "turns", 1, 96, "borg_fight_defend.rs:2885", "Defend light beam max invis age: existing policy turns."),
    defend_light_beam_danger_numerator => (3, "multiplier", 0, 10, "borg_fight_defend.rs:2940", "Defend light beam danger numerator: existing policy multiplier."),
    defend_light_beam_danger_divisor => (4, "divisor", 1, 1000, "borg_fight_defend.rs:2940", "Defend light beam danger divisor: existing policy divisor."),
    defend_light_beam_value => (10, "score", 0, 100000, "borg_fight_defend.rs:2881", "Defend light beam value: existing policy score."),
    defend_panel_shift_min_depth => (70, "depth", 0, 127, "borg_fight_defend.rs:2987", "Defend panel shift min depth: existing policy depth."),
    defend_panel_shift_edge_distance => (5, "grids", 1, 20, "borg_fight_defend.rs:3010", "Defend panel shift edge distance: existing policy grids."),
    defend_panel_shift_max_age => (10, "turns", 1, 20, "borg_fight_defend.rs:3052", "Defend panel shift max age: existing policy turns."),
    defend_panel_shift_morgoth_max_age => (10, "turns", 1, 20, "borg_fight_defend.rs:3053", "Defend panel shift morgoth max age: existing policy turns."),
    defend_panel_shift_antisummon_min_age => (50, "turns", 1, 100, "borg_fight_defend.rs:3095", "Defend panel shift antisummon min age: existing policy turns."),
    defend_panel_shift_teleport_fail => (30, "percent", 0, 100, "borg_fight_defend.rs:3108", "Defend panel shift teleport fail: existing policy percent."),
    defend_panel_shift_value => (200, "score", 0, 100000, "borg_fight_defend.rs:3152", "Defend panel shift value: existing policy score."),
    defend_teleport_morgoth_min_glyphs => (10, "count", 0, 255, "borg_fight_defend.rs:3191", "Defend teleport morgoth min glyphs: existing policy count."),
    defend_teleport_morgoth_value_cap => (1000, "score", 0, 100000, "borg_fight_defend.rs:3199", "Defend teleport morgoth value cap: existing policy score."),
    defend_teleport_distance => (7, "grids", 1, 20, "borg_fight_defend.rs:3384", "Defend teleport distance: existing policy grids."),
    defend_teleport_min_count => (7, "count", 0, 255, "borg_fight_defend.rs:3322", "Defend teleport min count: existing policy count."),
    defend_teleport_simulated_value => (1500, "score", 0, 100000, "borg_fight_defend.rs:3328", "Defend teleport simulated value: existing policy score."),
    defend_teleport_value => (1000, "score", 0, 100000, "borg_fight_defend.rs:3358", "Defend teleport value: existing policy score."),
    defend_smite_max_distance => (5, "grids", 1, 20, "borg_fight_defend.rs:3501", "Defend smite max distance: existing policy grids."),
    defend_smite_simulated_value => (500, "score", 0, 100000, "borg_fight_defend.rs:3452", "Defend smite simulated value: existing policy score."),
    defend_smite_value => (200, "score", 0, 100000, "borg_fight_defend.rs:3469", "Defend smite value: existing policy score."),
    defend_refresh_resistance_horizon => (2, "turns", 1, 10, "borg_fight_defend.rs:3603", "Defend refresh resistance horizon: existing policy turns."),
    defend_refresh_resistance_duration => (25000, "duration", 0, 32767, "borg_fight_defend.rs:3593", "Defend refresh resistance duration: existing policy duration."),

    // borg-reborn-rs (#242): item_use source at 5d9abb8f.
    item_use_ccw_recent_window => (4, "turns", 1, 10, "borg_item_use.rs:139", "Item_Use ccw recent window: existing policy turns."),
    item_use_ccw_recent_skip_chance => (75, "percent", 0, 100, "borg_item_use.rs:139", "Item_Use ccw recent skip chance: existing policy percent."),
    item_use_ccw_min_stock => (2, "count", 0, 255, "borg_item_use.rs:144", "Item_Use ccw min stock: existing policy count."),
    item_use_food_heal_critical_hp => (4, "hp", 0, 10000, "borg_item_use.rs:379", "Item_Use food heal critical hp: existing policy hp."),
    item_use_rod_equip_max_failure => (500, "permille", 0, 1000, "borg_item_use.rs:107", "Item_Use rod equip max failure: existing policy permille."),
    item_use_rod_use_max_failure => (500, "permille", 0, 1000, "borg_item_use.rs:108", "Item_Use rod use max failure: existing policy permille."),
    item_use_staff_use_max_failure => (500, "permille", 0, 1000, "borg_item_use.rs:483", "Item_Use staff use max failure: existing policy permille."),
    item_use_staff_equip_max_failure => (500, "permille", 0, 1000, "borg_item_use.rs:111", "Item_Use staff equip max failure: existing policy permille."),
    item_use_staff_teleport_max_failure => (650, "permille", 0, 1000, "borg_item_use.rs:624", "Item_Use staff teleport max failure: existing policy permille."),
    item_use_ring_equip_max_failure => (500, "permille", 0, 1000, "borg_item_use.rs:725", "Item_Use ring equip max failure: existing policy permille."),
    item_use_dragon_equip_max_failure => (500, "permille", 0, 1000, "borg_item_use.rs:847", "Item_Use dragon equip max failure: existing policy permille."),
    item_use_unholy_min_hp => (90, "hp", 0, 10000, "borg_item_use.rs:1009", "Item_Use unholy min hp: existing policy hp."),
    item_use_recharge_wand_max_charges => (1, "count", 0, 255, "borg_item_use.rs:1201", "Item_Use recharge wand max charges: existing policy count."),
    item_use_recharge_staff_max_charges => (1, "count", 0, 255, "borg_item_use.rs:1218", "Item_Use recharge staff max charges: existing policy count."),
    item_use_recharge_stacked_min_charges => (1, "count", 0, 255, "borg_item_use.rs:1230", "Item_Use recharge stacked min charges: existing policy count."),
    item_use_recharge_low_charges => (2, "count", 0, 255, "borg_item_use.rs:1210", "Item_Use recharge low charges: existing policy count."),
    item_use_recharge_fail => (96, "percent", 0, 100, "borg_item_use.rs:1211", "Item_Use recharge fail: existing policy percent."),
    item_use_teleport_staff_min_charges => (3, "count", 0, 255, "borg_item_use.rs:1223", "Item_Use teleport staff min charges: existing policy count."),
    item_use_recharge_min_total_stock => (4, "count", 0, 255, "borg_item_use.rs:1230", "Item_Use recharge min total stock: existing policy count."),

    // borg-reborn-rs (#243): power source at 5d9abb8f.
    power_activation_tele_other_mage_base => (500, "score", 0, 100000, "borg_power.rs:111", "Power activation tele other mage base: existing policy score."),
    power_activation_tele_other_base => (500, "score", 0, 100000, "borg_power.rs:111", "Power activation tele other base: existing policy score."),
    power_activation_tele_other_nonmage_bonus => (500, "score", 0, 100000, "borg_power.rs:111", "Power activation tele other nonmage bonus: existing policy score."),
    power_activation_remove_fear_caster_base => (500, "score", 0, 100000, "borg_power.rs:111", "Power activation remove fear caster base: existing policy score."),
    power_activation_remove_fear_base => (500, "score", 0, 100000, "borg_power.rs:113", "Power activation remove fear base: existing policy score."),
    power_activation_remove_fear_noncaster_bonus => (200, "score", 0, 100000, "borg_power.rs:113", "Power activation remove fear noncaster bonus: existing policy score."),
    power_activation_resist_all_base => (500, "score", 0, 100000, "borg_power.rs:115", "Power activation resist all base: existing policy score."),
    power_activation_resist_all_extra => (150, "score", 0, 100000, "borg_power.rs:115", "Power activation resist all extra: existing policy score."),
    power_activation_resist_all_nonmage_bonus => (25000, "score", 0, 100000, "borg_power.rs:115", "Power activation resist all nonmage bonus: existing policy score."),
    power_activation_sleep_base => (500, "score", 0, 100000, "borg_power.rs:116", "Power activation sleep base: existing policy score."),
    power_activation_sleep_class_bonus => (200, "score", 0, 100000, "borg_power.rs:118", "Power activation sleep class bonus: existing policy score."),
    power_activation_recharge_base => (500, "score", 0, 100000, "borg_power.rs:122", "Power activation recharge base: existing policy score."),
    power_activation_recharge_class_bonus => (100, "score", 0, 100000, "borg_power.rs:124", "Power activation recharge class bonus: existing policy score."),
    power_activation_rage_bless_resist_base => (500, "score", 0, 100000, "borg_power.rs:128", "Power activation rage bless resist base: existing policy score."),
    power_activation_rage_bless_resist_extra => (150, "score", 0, 100000, "borg_power.rs:128", "Power activation rage bless resist extra: existing policy score."),
    power_activation_rage_bless_resist_nonmage_bonus => (25000, "score", 0, 100000, "borg_power.rs:128", "Power activation rage bless resist nonmage bonus: existing policy score."),
    power_activation_dispel_evil_base => (500, "score", 0, 100000, "borg_power.rs:129", "Power activation dispel evil base: existing policy score."),
    power_activation_illumination_base => (500, "score", 0, 1000, "borg_power.rs:132", "Power activation illumination base: existing policy score."),
    power_activation_mapping_base => (550, "score", 0, 1100, "borg_power.rs:133", "Power activation mapping base: existing policy score."),
    power_activation_clairvoyance_base => (600, "score", 0, 1200, "borg_power.rs:134", "Power activation clairvoyance base: existing policy score."),
    power_activation_fire_bolt_base => (500, "score", 0, 1000, "borg_power.rs:135", "Power activation fire bolt base: existing policy score."),
    power_activation_cold_bolt_base => (500, "score", 0, 1000, "borg_power.rs:136", "Power activation cold bolt base: existing policy score."),
    power_activation_elec_bolt_base => (500, "score", 0, 1000, "borg_power.rs:137", "Power activation elec bolt base: existing policy score."),
    power_activation_acid_bolt_base => (500, "score", 0, 1000, "borg_power.rs:138", "Power activation acid bolt base: existing policy score."),
    power_activation_mana_bolt_base => (500, "score", 0, 1000, "borg_power.rs:139", "Power activation mana bolt base: existing policy score."),
    power_activation_stinking_cloud_base => (500, "score", 0, 1000, "borg_power.rs:140", "Power activation stinking cloud base: existing policy score."),
    power_activation_cold_ball50_base => (500, "score", 0, 1000, "borg_power.rs:141", "Power activation cold ball50 base: existing policy score."),
    power_activation_cold_ball100_base => (500, "score", 0, 1000, "borg_power.rs:142", "Power activation cold ball100 base: existing policy score."),
    power_activation_fire_ball72_base => (500, "score", 0, 1000, "borg_power.rs:143", "Power activation fire ball72 base: existing policy score."),
    power_activation_cold_bolt2_base => (500, "score", 0, 1000, "borg_power.rs:144", "Power activation cold bolt2 base: existing policy score."),
    power_activation_fire_ball_base => (500, "score", 0, 1000, "borg_power.rs:145", "Power activation fire ball base: existing policy score."),
    power_activation_confuse2_base => (0, "score", 0, 100, "borg_power.rs:147", "Power activation confuse2 base: existing policy score."),
    power_activation_haste_base => (0, "score", 0, 100, "borg_power.rs:148", "Power activation haste base: existing policy score."),
    power_activation_haste1_base => (0, "score", 0, 100, "borg_power.rs:149", "Power activation haste1 base: existing policy score."),
    power_activation_haste2_base => (0, "score", 0, 100, "borg_power.rs:150", "Power activation haste2 base: existing policy score."),
    power_activation_detect_objects_base => (10, "score", 0, 100, "borg_power.rs:151", "Power activation detect objects base: existing policy score."),
    power_activation_probing_base => (0, "score", 0, 100, "borg_power.rs:152", "Power activation probing base: existing policy score."),
    power_activation_stone_to_mud_base => (0, "score", 0, 100, "borg_power.rs:153", "Power activation stone to mud base: existing policy score."),
    power_activation_drain_life1_base => (500, "score", 0, 1000, "borg_power.rs:155", "Power activation drain life1 base: existing policy score."),
    power_activation_drain_life2_base => (500, "score", 0, 1000, "borg_power.rs:156", "Power activation drain life2 base: existing policy score."),
    power_activation_berserker_base => (500, "score", 0, 1000, "borg_power.rs:157", "Power activation berserker base: existing policy score."),
    power_activation_cure_light_base => (0, "score", 0, 100, "borg_power.rs:158", "Power activation cure light base: existing policy score."),
    power_activation_cure_serious_base => (0, "score", 0, 100, "borg_power.rs:159", "Power activation cure serious base: existing policy score."),
    power_activation_cure_critical_base => (0, "score", 0, 100, "borg_power.rs:160", "Power activation cure critical base: existing policy score."),
    power_activation_cure_full2_base => (0, "score", 0, 100, "borg_power.rs:161", "Power activation cure full2 base: existing policy score."),
    power_activation_loskill_base => (500, "score", 0, 1000, "borg_power.rs:162", "Power activation loskill base: existing policy score."),
    power_activation_recall_base => (0, "score", 0, 100, "borg_power.rs:163", "Power activation recall base: existing policy score."),
    power_activation_arrow_base => (500, "score", 0, 1000, "borg_power.rs:164", "Power activation arrow base: existing policy score."),
    power_activation_tele_phase_base => (500, "score", 0, 1000, "borg_power.rs:166", "Power activation tele phase base: existing policy score."),
    power_activation_detect_all_base => (0, "score", 0, 100, "borg_power.rs:167", "Power activation detect all base: existing policy score."),
    power_activation_cure_full_base => (0, "score", 0, 100, "borg_power.rs:168", "Power activation cure full base: existing policy score."),
    power_activation_heal1_base => (0, "score", 0, 100, "borg_power.rs:169", "Power activation heal1 base: existing policy score."),
    power_activation_heal2_base => (0, "score", 0, 100, "borg_power.rs:170", "Power activation heal2 base: existing policy score."),
    power_activation_heal3_base => (0, "score", 0, 100, "borg_power.rs:171", "Power activation heal3 base: existing policy score."),
    power_activation_cure_nonorlybig_base => (0, "score", 0, 100, "borg_power.rs:172", "Power activation cure nonorlybig base: existing policy score."),
    power_activation_protevil_base => (0, "score", 0, 100, "borg_power.rs:173", "Power activation protevil base: existing policy score."),
    power_activation_destroy_doors_base => (50, "score", 0, 100, "borg_power.rs:174", "Power activation destroy doors base: existing policy score."),
    power_activation_banishment_base => (1000, "score", 0, 2000, "borg_power.rs:175", "Power activation banishment base: existing policy score."),
    power_activation_tele_long_base => (300, "score", 0, 600, "borg_power.rs:179", "Power activation tele long base: existing policy score."),
    power_activation_missile_base => (500, "score", 0, 1000, "borg_power.rs:180", "Power activation missile base: existing policy score."),
    power_activation_cure_temp_base => (500, "score", 0, 1000, "borg_power.rs:181", "Power activation cure temp base: existing policy score."),
    power_activation_starlight2_base => (100, "score", 0, 200, "borg_power.rs:182", "Power activation starlight2 base: existing policy score."),
    power_activation_bizarre_base => (999999, "score", 0, 1999998, "borg_power.rs:183", "Power activation bizarre base: existing policy score."),
    power_activation_star_ball_base => (500, "score", 0, 1000, "borg_power.rs:184", "Power activation star ball base: existing policy score."),
    power_activation_polymorph_base => (0, "score", 0, 100, "borg_power.rs:186", "Power activation polymorph base: existing policy score."),
    power_activation_starlight_base => (100, "score", 0, 200, "borg_power.rs:187", "Power activation starlight base: existing policy score."),
    power_activation_light_base => (0, "score", 0, 100, "borg_power.rs:188", "Power activation light base: existing policy score."),
    power_activation_firebrand_base => (500, "score", 0, 1000, "borg_power.rs:189", "Power activation firebrand base: existing policy score."),
    power_activation_restore_life_base => (0, "score", 0, 100, "borg_power.rs:190", "Power activation restore life base: existing policy score."),
    power_activation_restore_exp_base => (0, "score", 0, 100, "borg_power.rs:191", "Power activation restore exp base: existing policy score."),
    power_activation_restore_st_lev_base => (0, "score", 0, 100, "borg_power.rs:192", "Power activation restore st lev base: existing policy score."),
    power_activation_enlightenment_base => (500, "score", 0, 1000, "borg_power.rs:193", "Power activation enlightenment base: existing policy score."),
    power_activation_hero_base => (500, "score", 0, 1000, "borg_power.rs:194", "Power activation hero base: existing policy score."),
    power_activation_shero_base => (500, "score", 0, 1000, "borg_power.rs:195", "Power activation shero base: existing policy score."),
    power_activation_cure_paranoia_base => (100, "score", 0, 200, "borg_power.rs:196", "Power activation cure paranoia base: existing policy score."),
    power_activation_cure_mind_base => (1000, "score", 0, 2000, "borg_power.rs:197", "Power activation cure mind base: existing policy score."),
    power_activation_cure_body_base => (1000, "score", 0, 2000, "borg_power.rs:198", "Power activation cure body base: existing policy score."),
    power_activation_mon_slow_base => (500, "score", 0, 1000, "borg_power.rs:199", "Power activation mon slow base: existing policy score."),
    power_activation_mon_confuse_base => (500, "score", 0, 1000, "borg_power.rs:200", "Power activation mon confuse base: existing policy score."),
    power_activation_sleep_all_base => (500, "score", 0, 1000, "borg_power.rs:201", "Power activation sleep all base: existing policy score."),
    power_activation_mon_scare_base => (500, "score", 0, 1000, "borg_power.rs:202", "Power activation mon scare base: existing policy score."),
    power_activation_light_line_base => (50, "score", 0, 100, "borg_power.rs:203", "Power activation light line base: existing policy score."),
    power_activation_disable_traps_base => (50, "score", 0, 100, "borg_power.rs:204", "Power activation disable traps base: existing policy score."),
    power_activation_drain_life3_base => (500, "score", 0, 1000, "borg_power.rs:205", "Power activation drain life3 base: existing policy score."),
    power_activation_drain_life4_base => (500, "score", 0, 1000, "borg_power.rs:206", "Power activation drain life4 base: existing policy score."),
    power_activation_elec_ball_base => (500, "score", 0, 1000, "borg_power.rs:207", "Power activation elec ball base: existing policy score."),
    power_activation_elec_ball2_base => (500, "score", 0, 1000, "borg_power.rs:208", "Power activation elec ball2 base: existing policy score."),
    power_activation_acid_bolt2_base => (500, "score", 0, 1000, "borg_power.rs:209", "Power activation acid bolt2 base: existing policy score."),
    power_activation_acid_bolt3_base => (500, "score", 0, 1000, "borg_power.rs:210", "Power activation acid bolt3 base: existing policy score."),
    power_activation_acid_ball_base => (500, "score", 0, 1000, "borg_power.rs:211", "Power activation acid ball base: existing policy score."),
    power_activation_cold_ball160_base => (500, "score", 0, 1000, "borg_power.rs:212", "Power activation cold ball160 base: existing policy score."),
    power_activation_cold_ball2_base => (500, "score", 0, 1000, "borg_power.rs:213", "Power activation cold ball2 base: existing policy score."),
    power_activation_fire_ball2_base => (500, "score", 0, 1000, "borg_power.rs:214", "Power activation fire ball2 base: existing policy score."),
    power_activation_fire_ball200_base => (500, "score", 0, 1000, "borg_power.rs:215", "Power activation fire ball200 base: existing policy score."),
    power_activation_fire_bolt2_base => (500, "score", 0, 1000, "borg_power.rs:216", "Power activation fire bolt2 base: existing policy score."),
    power_activation_fire_bolt3_base => (500, "score", 0, 1000, "borg_power.rs:217", "Power activation fire bolt3 base: existing policy score."),
    power_activation_dispel_evil60_base => (500, "score", 0, 1000, "borg_power.rs:218", "Power activation dispel evil60 base: existing policy score."),
    power_activation_dispel_undead_base => (500, "score", 0, 1000, "borg_power.rs:219", "Power activation dispel undead base: existing policy score."),
    power_activation_dispel_all_base => (500, "score", 0, 1000, "borg_power.rs:220", "Power activation dispel all base: existing policy score."),
    power_activation_deep_descent_base => (0, "score", 0, 100, "borg_power.rs:221", "Power activation deep descent base: existing policy score."),
    power_activation_earthquakes_base => (500, "score", 0, 1000, "borg_power.rs:222", "Power activation earthquakes base: existing policy score."),
    power_activation_destruction2_base => (500, "score", 0, 1000, "borg_power.rs:223", "Power activation destruction2 base: existing policy score."),
    power_activation_losslow_base => (50, "score", 0, 100, "borg_power.rs:224", "Power activation losslow base: existing policy score."),
    power_activation_lossleep_base => (100, "score", 0, 200, "borg_power.rs:225", "Power activation lossleep base: existing policy score."),
    power_activation_losconf_base => (100, "score", 0, 200, "borg_power.rs:226", "Power activation losconf base: existing policy score."),
    power_activation_satisfy_base => (50, "score", 0, 100, "borg_power.rs:227", "Power activation satisfy base: existing policy score."),
    power_activation_blessing_base => (50, "score", 0, 100, "borg_power.rs:228", "Power activation blessing base: existing policy score."),
    power_activation_blessing2_base => (50, "score", 0, 100, "borg_power.rs:229", "Power activation blessing2 base: existing policy score."),
    power_activation_blessing3_base => (50, "score", 0, 100, "borg_power.rs:230", "Power activation blessing3 base: existing policy score."),
    power_activation_glyph_base => (0, "score", 0, 100, "borg_power.rs:231", "Power activation glyph base: existing policy score."),
    power_activation_tele_level_base => (5000, "score", 0, 10000, "borg_power.rs:232", "Power activation tele level base: existing policy score."),
    power_activation_confusing_base => (0, "score", 0, 100, "borg_power.rs:233", "Power activation confusing base: existing policy score."),
    power_activation_enchant_tohit_base => (0, "score", 0, 100, "borg_power.rs:234", "Power activation enchant tohit base: existing policy score."),
    power_activation_enchant_todam_base => (0, "score", 0, 100, "borg_power.rs:235", "Power activation enchant todam base: existing policy score."),
    power_activation_enchant_weapon_base => (0, "score", 0, 100, "borg_power.rs:236", "Power activation enchant weapon base: existing policy score."),
    power_activation_enchant_armor_base => (0, "score", 0, 100, "borg_power.rs:237", "Power activation enchant armor base: existing policy score."),
    power_activation_enchant_armor2_base => (0, "score", 0, 100, "borg_power.rs:238", "Power activation enchant armor2 base: existing policy score."),
    power_activation_remove_curse_base => (9000, "score", 0, 18000, "borg_power.rs:239", "Power activation remove curse base: existing policy score."),
    power_activation_remove_curse2_base => (10000, "score", 0, 20000, "borg_power.rs:240", "Power activation remove curse2 base: existing policy score."),
    power_activation_detect_treasure_base => (0, "score", 0, 100, "borg_power.rs:241", "Power activation detect treasure base: existing policy score."),
    power_activation_detect_invis_base => (50, "score", 0, 100, "borg_power.rs:242", "Power activation detect invis base: existing policy score."),
    power_activation_detect_evil_base => (50, "score", 0, 100, "borg_power.rs:243", "Power activation detect evil base: existing policy score."),
    power_activation_restore_mana_base => (5000, "score", 0, 10000, "borg_power.rs:244", "Power activation restore mana base: existing policy score."),
    power_activation_brawn_base => (0, "score", 0, 100, "borg_power.rs:245", "Power activation brawn base: existing policy score."),
    power_activation_intellect_base => (0, "score", 0, 100, "borg_power.rs:246", "Power activation intellect base: existing policy score."),
    power_activation_contemplation_base => (0, "score", 0, 100, "borg_power.rs:247", "Power activation contemplation base: existing policy score."),
    power_activation_toughness_base => (0, "score", 0, 100, "borg_power.rs:248", "Power activation toughness base: existing policy score."),
    power_activation_nimbleness_base => (0, "score", 0, 100, "borg_power.rs:249", "Power activation nimbleness base: existing policy score."),
    power_activation_restore_str_base => (50, "score", 0, 100, "borg_power.rs:250", "Power activation restore str base: existing policy score."),
    power_activation_restore_int_base => (50, "score", 0, 100, "borg_power.rs:251", "Power activation restore int base: existing policy score."),
    power_activation_restore_wis_base => (50, "score", 0, 100, "borg_power.rs:252", "Power activation restore wis base: existing policy score."),
    power_activation_restore_dex_base => (50, "score", 0, 100, "borg_power.rs:253", "Power activation restore dex base: existing policy score."),
    power_activation_restore_con_base => (50, "score", 0, 100, "borg_power.rs:254", "Power activation restore con base: existing policy score."),
    power_activation_restore_all_base => (150, "score", 0, 300, "borg_power.rs:255", "Power activation restore all base: existing policy score."),
    power_activation_tmd_free_act_base => (0, "score", 0, 100, "borg_power.rs:256", "Power activation tmd free act base: existing policy score."),
    power_activation_tmd_infra_base => (0, "score", 0, 100, "borg_power.rs:257", "Power activation tmd infra base: existing policy score."),
    power_activation_tmd_sinvis_base => (50, "score", 0, 100, "borg_power.rs:258", "Power activation tmd sinvis base: existing policy score."),
    power_activation_tmd_esp_base => (50, "score", 0, 100, "borg_power.rs:259", "Power activation tmd esp base: existing policy score."),
    power_activation_resist_acid_base => (100, "score", 0, 200, "borg_power.rs:260", "Power activation resist acid base: existing policy score."),
    power_activation_resist_elec_base => (100, "score", 0, 200, "borg_power.rs:261", "Power activation resist elec base: existing policy score."),
    power_activation_resist_fire_base => (100, "score", 0, 200, "borg_power.rs:262", "Power activation resist fire base: existing policy score."),
    power_activation_resist_cold_base => (100, "score", 0, 200, "borg_power.rs:263", "Power activation resist cold base: existing policy score."),
    power_activation_resist_pois_base => (150, "score", 0, 300, "borg_power.rs:264", "Power activation resist pois base: existing policy score."),
    power_activation_cure_confusion_base => (50, "score", 0, 100, "borg_power.rs:265", "Power activation cure confusion base: existing policy score."),
    power_activation_wonder_base => (300, "score", 0, 600, "borg_power.rs:266", "Power activation wonder base: existing policy score."),
    power_activation_wand_breath_base => (0, "score", 0, 100, "borg_power.rs:267", "Power activation wand breath base: existing policy score."),
    power_activation_staff_magi_base => (0, "score", 0, 100, "borg_power.rs:268", "Power activation staff magi base: existing policy score."),
    power_activation_staff_holy_base => (1000, "score", 0, 2000, "borg_power.rs:269", "Power activation staff holy base: existing policy score."),
    power_activation_drink_breath_base => (0, "score", 0, 100, "borg_power.rs:270", "Power activation drink breath base: existing policy score."),
    power_activation_food_waybread_base => (50, "score", 0, 100, "borg_power.rs:271", "Power activation food waybread base: existing policy score."),
    power_activation_shroom_emergency_base => (0, "score", 0, 100, "borg_power.rs:272", "Power activation shroom emergency base: existing policy score."),
    power_activation_shroom_terror_base => (0, "score", 0, 100, "borg_power.rs:273", "Power activation shroom terror base: existing policy score."),
    power_activation_shroom_stone_base => (50, "score", 0, 100, "borg_power.rs:274", "Power activation shroom stone base: existing policy score."),
    power_activation_shroom_debility_base => (0, "score", 0, 100, "borg_power.rs:275", "Power activation shroom debility base: existing policy score."),
    power_activation_shroom_sprinting_base => (0, "score", 0, 100, "borg_power.rs:276", "Power activation shroom sprinting base: existing policy score."),
    power_activation_shroom_purging_base => (50, "score", 0, 100, "borg_power.rs:277", "Power activation shroom purging base: existing policy score."),
    power_activation_ring_acid_base => (10000, "score", 0, 20000, "borg_power.rs:278", "Power activation ring acid base: existing policy score."),
    power_activation_ring_flames_base => (25000, "score", 0, 50000, "borg_power.rs:279", "Power activation ring flames base: existing policy score."),
    power_activation_ring_ice_base => (15000, "score", 0, 30000, "borg_power.rs:280", "Power activation ring ice base: existing policy score."),
    power_activation_ring_lightning_base => (10000, "score", 0, 20000, "borg_power.rs:281", "Power activation ring lightning base: existing policy score."),
    power_activation_dragon_blue_base => (1100, "score", 0, 2200, "borg_power.rs:282", "Power activation dragon blue base: existing policy score."),
    power_activation_dragon_green_base => (2750, "score", 0, 5500, "borg_power.rs:283", "Power activation dragon green base: existing policy score."),
    power_activation_dragon_red_base => (1100, "score", 0, 2200, "borg_power.rs:284", "Power activation dragon red base: existing policy score."),
    power_activation_dragon_multihued_base => (3250, "score", 0, 6500, "borg_power.rs:285", "Power activation dragon multihued base: existing policy score."),
    power_activation_dragon_gold_base => (5150, "score", 0, 10300, "borg_power.rs:286", "Power activation dragon gold base: existing policy score."),
    power_activation_dragon_chaos_base => (5150, "score", 0, 10300, "borg_power.rs:287", "Power activation dragon chaos base: existing policy score."),
    power_activation_dragon_law_base => (5150, "score", 0, 10300, "borg_power.rs:288", "Power activation dragon law base: existing policy score."),
    power_activation_dragon_balance_base => (5150, "score", 0, 10300, "borg_power.rs:289", "Power activation dragon balance base: existing policy score."),
    power_activation_dragon_shining_base => (5150, "score", 0, 10300, "borg_power.rs:290", "Power activation dragon shining base: existing policy score."),
    power_activation_dragon_power_base => (5150, "score", 0, 10300, "borg_power.rs:291", "Power activation dragon power base: existing policy score."),
    power_equipment_unidentified_weapon_bonus => (1000000, "score", 0, 2000000, "borg_power.rs:321", "Power equipment unidentified weapon bonus: existing policy score."),
    power_equipment_melee_dice_weight => (20, "score", 0, 100, "borg_power.rs:327", "Power equipment melee dice weight: existing policy score."),
    power_equipment_blows_weight => (1, "score", 0, 100, "borg_power.rs:330", "Power equipment blows weight: existing policy score."),
    power_equipment_tohit_weight => (100, "score", 0, 200, "borg_power.rs:333", "Power equipment tohit weight: existing policy score."),
    power_equipment_todam_weight => (30, "score", 0, 100, "borg_power.rs:336", "Power equipment todam weight: existing policy score."),
    power_equipment_tohit_weight_2 => (15, "score", 0, 100, "borg_power.rs:340", "Power equipment tohit weight 2: existing policy score."),
    power_equipment_maxdepth_threshold => (75, "depth", 0, 127, "borg_power.rs:344", "Power equipment maxdepth threshold: existing policy depth."),
    power_equipment_tohit_weight_3 => (15, "score", 0, 100, "borg_power.rs:345", "Power equipment tohit weight 3: existing policy score."),
    power_equipment_blows_weight_2 => (20, "score", 0, 100, "borg_power.rs:347", "Power equipment blows weight 2: existing policy score."),
    power_equipment_blows_weight_3 => (2, "score", 0, 100, "borg_power.rs:347", "Power equipment blows weight 3: existing policy score."),
    power_equipment_ws_animal_weight => (2, "score", 0, 100, "borg_power.rs:354", "Power equipment ws animal weight: existing policy score."),
    power_equipment_ws_evil_weight => (7, "score", 0, 100, "borg_power.rs:357", "Power equipment ws evil weight: existing policy score."),
    power_equipment_wk_undead_weight => (5, "score", 0, 100, "borg_power.rs:369", "Power equipment wk undead weight: existing policy score."),
    power_equipment_wk_demon_weight => (3, "score", 0, 100, "borg_power.rs:372", "Power equipment wk demon weight: existing policy score."),
    power_equipment_wk_dragon_weight => (6, "score", 0, 100, "borg_power.rs:375", "Power equipment wk dragon weight: existing policy score."),
    power_equipment_ws_giant_weight => (4, "score", 0, 100, "borg_power.rs:378", "Power equipment ws giant weight: existing policy score."),
    power_equipment_wb_acid_weight => (4, "score", 0, 100, "borg_power.rs:381", "Power equipment wb acid weight: existing policy score."),
    power_equipment_wb_elec_weight => (5, "score", 0, 100, "borg_power.rs:384", "Power equipment wb elec weight: existing policy score."),
    power_equipment_wb_fire_weight => (3, "score", 0, 100, "borg_power.rs:387", "Power equipment wb fire weight: existing policy score."),
    power_equipment_wb_cold_weight => (3, "score", 0, 100, "borg_power.rs:390", "Power equipment wb cold weight: existing policy score."),
    power_equipment_ws_orc_weight => (1, "score", 0, 100, "borg_power.rs:394", "Power equipment ws orc weight: existing policy score."),
    power_equipment_ws_troll_weight => (2, "score", 0, 100, "borg_power.rs:397", "Power equipment ws troll weight: existing policy score."),
    power_equipment_ws_evil_weight_2 => (1, "score", 0, 100, "borg_power.rs:401", "Power equipment ws evil weight 2: existing policy score."),
    power_equipment_ws_evil_weight_3 => (1, "score", 0, 100, "borg_power.rs:404", "Power equipment ws evil weight 3: existing policy score."),
    power_equipment_wk_undead_weight_2 => (5, "score", 0, 100, "borg_power.rs:416", "Power equipment wk undead weight 2: existing policy score."),
    power_equipment_wk_demon_weight_2 => (5, "score", 0, 100, "borg_power.rs:419", "Power equipment wk demon weight 2: existing policy score."),
    power_equipment_wk_dragon_weight_2 => (5, "score", 0, 100, "borg_power.rs:422", "Power equipment wk dragon weight 2: existing policy score."),
    power_equipment_w_impact_weight => (50, "score", 0, 100, "borg_power.rs:431", "Power equipment w impact weight: existing policy score."),
    power_equipment_heavywepon_weight => (500000, "score", 0, 1000000, "borg_power.rs:436", "Power equipment heavywepon weight: existing policy score."),
    power_equipment_clevel_threshold => (10, "level", 0, 51, "borg_power.rs:440", "Power equipment clevel threshold: existing policy level."),
    power_equipment_blows_weight_4 => (45000, "score", 0, 90000, "borg_power.rs:441", "Power equipment blows weight 4: existing policy score."),
    power_equipment_unidentified_bow_bonus => (6000000, "score", 0, 12000000, "borg_power.rs:454", "Power equipment unidentified bow bonus: existing policy score."),
    power_equipment_reward_damage_weight => (11, "score", 0, 100, "borg_power.rs:467", "Power equipment reward damage weight: existing policy score."),
    power_equipment_reward_damage_weight_2 => (9, "score", 0, 100, "borg_power.rs:469", "Power equipment reward damage weight 2: existing policy score."),
    power_equipment_clevel_threshold_2 => (15, "level", 0, 51, "borg_power.rs:473", "Power equipment clevel threshold 2: existing policy level."),
    power_equipment_clevel_weight => (200, "score", 0, 400, "borg_power.rs:474", "Power equipment clevel weight: existing policy score."),
    power_equipment_str_threshold => (9, "stat", 0, 238, "borg_power.rs:479", "Power equipment str threshold: existing policy stat."),
    power_equipment_str_weight => (5000, "score", 0, 10000, "borg_power.rs:480", "Power equipment str weight: existing policy score."),
    power_equipment_clevel_threshold_3 => (1, "level", 0, 51, "borg_power.rs:484", "Power equipment clevel threshold 3: existing policy level."),
    power_equipment_str_threshold_2 => (9, "stat", 0, 238, "borg_power.rs:484", "Power equipment str threshold 2: existing policy stat."),
    power_equipment_str_weight_2 => (8000, "score", 0, 16000, "borg_power.rs:485", "Power equipment str weight 2: existing policy score."),
    power_equipment_tohit_weight_4 => (100, "score", 0, 200, "borg_power.rs:489", "Power equipment tohit weight 4: existing policy score."),
    power_equipment_tohit_weight_5 => (25, "score", 0, 100, "borg_power.rs:493", "Power equipment tohit weight 5: existing policy score."),
    power_equipment_ammo_tval_weight => (30000, "score", 0, 60000, "borg_power.rs:498", "Power equipment ammo tval weight: existing policy score."),
    power_equipment_it_is_hard_to_hold_a_heavy_weapon_weight => (500000, "score", 0, 1000000, "borg_power.rs:503", "Power equipment it is hard to hold a heavy weapon weight: existing policy score."),
    power_equipment_light_threshold => (3, "count", 0, 255, "borg_power.rs:512", "Power equipment light threshold: existing policy count."),
    power_equipment_light_weight => (10000, "score", 0, 20000, "borg_power.rs:513", "Power equipment light weight: existing policy score."),
    power_equipment_light_threshold_2 => (3, "count", 0, 255, "borg_power.rs:514", "Power equipment light threshold 2: existing policy count."),
    power_equipment_light_weight_2 => (30000, "score", 0, 60000, "borg_power.rs:515", "Power equipment light weight 2: existing policy score."),
    power_equipment_light_weight_3 => (1000, "score", 0, 2000, "borg_power.rs:515", "Power equipment light weight 3: existing policy score."),
    power_equipment_mod_moves_weight => (3000, "score", 0, 6000, "borg_power.rs:518", "Power equipment mod moves weight: existing policy score."),
    power_equipment_dam_red_weight => (10000, "score", 0, 20000, "borg_power.rs:519", "Power equipment dam red weight: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold => (150, "speed", 0, 199, "borg_power.rs:528", "Power equipment reward higher for 10 than 50 speed dec threshold: existing policy speed."),
    power_equipment_speed_tier_base => (120, "speed", 0, 199, "borg_power.rs:529", "Power equipment speed tier base: existing policy speed."),
    power_equipment_speed_tier_slope => (1500, "score", 0, 3000, "borg_power.rs:529", "Power equipment speed tier slope: existing policy score."),
    power_equipment_speed_tier_offset => (185000, "score", 0, 370000, "borg_power.rs:529", "Power equipment speed tier offset: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_2 => (145, "speed", 0, 199, "borg_power.rs:531", "Power equipment reward higher for 10 than 50 speed dec threshold 2: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_3 => (149, "speed", 0, 199, "borg_power.rs:531", "Power equipment reward higher for 10 than 50 speed dec threshold 3: existing policy speed."),
    power_equipment_speed_tier_base_2 => (120, "speed", 0, 199, "borg_power.rs:532", "Power equipment speed tier base 2: existing policy speed."),
    power_equipment_speed_tier_slope_2 => (1500, "score", 0, 3000, "borg_power.rs:532", "Power equipment speed tier slope 2: existing policy score."),
    power_equipment_speed_tier_offset_2 => (180000, "score", 0, 360000, "borg_power.rs:532", "Power equipment speed tier offset 2: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_4 => (140, "speed", 0, 199, "borg_power.rs:534", "Power equipment reward higher for 10 than 50 speed dec threshold 4: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_5 => (144, "speed", 0, 199, "borg_power.rs:534", "Power equipment reward higher for 10 than 50 speed dec threshold 5: existing policy speed."),
    power_equipment_speed_tier_base_3 => (120, "speed", 0, 199, "borg_power.rs:535", "Power equipment speed tier base 3: existing policy speed."),
    power_equipment_speed_tier_slope_3 => (1500, "score", 0, 3000, "borg_power.rs:535", "Power equipment speed tier slope 3: existing policy score."),
    power_equipment_speed_tier_offset_3 => (175000, "score", 0, 350000, "borg_power.rs:535", "Power equipment speed tier offset 3: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_6 => (135, "speed", 0, 199, "borg_power.rs:537", "Power equipment reward higher for 10 than 50 speed dec threshold 6: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_7 => (139, "speed", 0, 199, "borg_power.rs:537", "Power equipment reward higher for 10 than 50 speed dec threshold 7: existing policy speed."),
    power_equipment_speed_tier_base_4 => (120, "speed", 0, 199, "borg_power.rs:538", "Power equipment speed tier base 4: existing policy speed."),
    power_equipment_speed_tier_slope_4 => (1500, "score", 0, 3000, "borg_power.rs:538", "Power equipment speed tier slope 4: existing policy score."),
    power_equipment_speed_tier_offset_4 => (175000, "score", 0, 350000, "borg_power.rs:538", "Power equipment speed tier offset 4: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_8 => (130, "speed", 0, 199, "borg_power.rs:540", "Power equipment reward higher for 10 than 50 speed dec threshold 8: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_9 => (134, "speed", 0, 199, "borg_power.rs:540", "Power equipment reward higher for 10 than 50 speed dec threshold 9: existing policy speed."),
    power_equipment_speed_tier_base_5 => (120, "speed", 0, 199, "borg_power.rs:541", "Power equipment speed tier base 5: existing policy speed."),
    power_equipment_speed_tier_slope_5 => (1500, "score", 0, 3000, "borg_power.rs:541", "Power equipment speed tier slope 5: existing policy score."),
    power_equipment_speed_tier_offset_5 => (160000, "score", 0, 320000, "borg_power.rs:541", "Power equipment speed tier offset 5: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_10 => (125, "speed", 0, 199, "borg_power.rs:543", "Power equipment reward higher for 10 than 50 speed dec threshold 10: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_11 => (129, "speed", 0, 199, "borg_power.rs:543", "Power equipment reward higher for 10 than 50 speed dec threshold 11: existing policy speed."),
    power_equipment_speed_tier_base_6 => (110, "speed", 0, 199, "borg_power.rs:544", "Power equipment speed tier base 6: existing policy speed."),
    power_equipment_speed_tier_slope_6 => (1500, "score", 0, 3000, "borg_power.rs:544", "Power equipment speed tier slope 6: existing policy score."),
    power_equipment_speed_tier_offset_6 => (135000, "score", 0, 270000, "borg_power.rs:544", "Power equipment speed tier offset 6: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_12 => (120, "speed", 0, 199, "borg_power.rs:546", "Power equipment reward higher for 10 than 50 speed dec threshold 12: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_13 => (124, "speed", 0, 199, "borg_power.rs:546", "Power equipment reward higher for 10 than 50 speed dec threshold 13: existing policy speed."),
    power_equipment_speed_tier_base_7 => (110, "speed", 0, 199, "borg_power.rs:547", "Power equipment speed tier base 7: existing policy speed."),
    power_equipment_speed_tier_slope_7 => (1500, "score", 0, 3000, "borg_power.rs:547", "Power equipment speed tier slope 7: existing policy score."),
    power_equipment_speed_tier_offset_7 => (110000, "score", 0, 220000, "borg_power.rs:547", "Power equipment speed tier offset 7: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_14 => (115, "speed", 0, 199, "borg_power.rs:549", "Power equipment reward higher for 10 than 50 speed dec threshold 14: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_15 => (119, "speed", 0, 199, "borg_power.rs:549", "Power equipment reward higher for 10 than 50 speed dec threshold 15: existing policy speed."),
    power_equipment_speed_tier_base_8 => (110, "speed", 0, 199, "borg_power.rs:550", "Power equipment speed tier base 8: existing policy speed."),
    power_equipment_speed_tier_slope_8 => (1500, "score", 0, 3000, "borg_power.rs:550", "Power equipment speed tier slope 8: existing policy score."),
    power_equipment_speed_tier_offset_8 => (85000, "score", 0, 170000, "borg_power.rs:550", "Power equipment speed tier offset 8: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_16 => (110, "speed", 0, 199, "borg_power.rs:552", "Power equipment reward higher for 10 than 50 speed dec threshold 16: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_17 => (114, "speed", 0, 199, "borg_power.rs:552", "Power equipment reward higher for 10 than 50 speed dec threshold 17: existing policy speed."),
    power_equipment_speed_tier_base_9 => (110, "speed", 0, 199, "borg_power.rs:553", "Power equipment speed tier base 9: existing policy speed."),
    power_equipment_speed_tier_slope_9 => (1500, "score", 0, 3000, "borg_power.rs:553", "Power equipment speed tier slope 9: existing policy score."),
    power_equipment_speed_tier_offset_9 => (65000, "score", 0, 130000, "borg_power.rs:553", "Power equipment speed tier offset 9: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_18 => (110, "speed", 0, 199, "borg_power.rs:555", "Power equipment reward higher for 10 than 50 speed dec threshold 18: existing policy speed."),
    power_equipment_speed_tier_base_10 => (110, "speed", 0, 199, "borg_power.rs:556", "Power equipment speed tier base 10: existing policy speed."),
    power_equipment_speed_tier_slope_10 => (2500, "score", 0, 5000, "borg_power.rs:556", "Power equipment speed tier slope 10: existing policy score."),
    power_equipment_speed_tier_offset_10 => (0, "score", 0, 100, "borg_power.rs:556", "Power equipment speed tier offset 10: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_19 => (140, "speed", 0, 199, "borg_power.rs:559", "Power equipment reward higher for 10 than 50 speed dec threshold 19: existing policy speed."),
    power_equipment_speed_tier_base_11 => (120, "speed", 0, 199, "borg_power.rs:560", "Power equipment speed tier base 11: existing policy speed."),
    power_equipment_speed_tier_slope_11 => (1000, "score", 0, 2000, "borg_power.rs:560", "Power equipment speed tier slope 11: existing policy score."),
    power_equipment_speed_tier_offset_11 => (175000, "score", 0, 350000, "borg_power.rs:560", "Power equipment speed tier offset 11: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_20 => (135, "speed", 0, 199, "borg_power.rs:562", "Power equipment reward higher for 10 than 50 speed dec threshold 20: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_21 => (139, "speed", 0, 199, "borg_power.rs:562", "Power equipment reward higher for 10 than 50 speed dec threshold 21: existing policy speed."),
    power_equipment_speed_tier_base_12 => (120, "speed", 0, 199, "borg_power.rs:563", "Power equipment speed tier base 12: existing policy speed."),
    power_equipment_speed_tier_slope_12 => (1000, "score", 0, 2000, "borg_power.rs:563", "Power equipment speed tier slope 12: existing policy score."),
    power_equipment_speed_tier_offset_12 => (165000, "score", 0, 330000, "borg_power.rs:563", "Power equipment speed tier offset 12: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_22 => (130, "speed", 0, 199, "borg_power.rs:565", "Power equipment reward higher for 10 than 50 speed dec threshold 22: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_23 => (134, "speed", 0, 199, "borg_power.rs:565", "Power equipment reward higher for 10 than 50 speed dec threshold 23: existing policy speed."),
    power_equipment_speed_tier_base_13 => (120, "speed", 0, 199, "borg_power.rs:566", "Power equipment speed tier base 13: existing policy speed."),
    power_equipment_speed_tier_slope_13 => (1000, "score", 0, 2000, "borg_power.rs:566", "Power equipment speed tier slope 13: existing policy score."),
    power_equipment_speed_tier_offset_13 => (150000, "score", 0, 300000, "borg_power.rs:566", "Power equipment speed tier offset 13: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_24 => (125, "speed", 0, 199, "borg_power.rs:568", "Power equipment reward higher for 10 than 50 speed dec threshold 24: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_25 => (129, "speed", 0, 199, "borg_power.rs:568", "Power equipment reward higher for 10 than 50 speed dec threshold 25: existing policy speed."),
    power_equipment_speed_tier_base_14 => (110, "speed", 0, 199, "borg_power.rs:569", "Power equipment speed tier base 14: existing policy speed."),
    power_equipment_speed_tier_slope_14 => (1000, "score", 0, 2000, "borg_power.rs:569", "Power equipment speed tier slope 14: existing policy score."),
    power_equipment_speed_tier_offset_14 => (125000, "score", 0, 250000, "borg_power.rs:569", "Power equipment speed tier offset 14: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_26 => (120, "speed", 0, 199, "borg_power.rs:571", "Power equipment reward higher for 10 than 50 speed dec threshold 26: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_27 => (124, "speed", 0, 199, "borg_power.rs:571", "Power equipment reward higher for 10 than 50 speed dec threshold 27: existing policy speed."),
    power_equipment_speed_tier_base_15 => (110, "speed", 0, 199, "borg_power.rs:572", "Power equipment speed tier base 15: existing policy speed."),
    power_equipment_speed_tier_slope_15 => (1000, "score", 0, 2000, "borg_power.rs:572", "Power equipment speed tier slope 15: existing policy score."),
    power_equipment_speed_tier_offset_15 => (100000, "score", 0, 200000, "borg_power.rs:572", "Power equipment speed tier offset 15: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_28 => (115, "speed", 0, 199, "borg_power.rs:574", "Power equipment reward higher for 10 than 50 speed dec threshold 28: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_29 => (119, "speed", 0, 199, "borg_power.rs:574", "Power equipment reward higher for 10 than 50 speed dec threshold 29: existing policy speed."),
    power_equipment_speed_tier_base_16 => (110, "speed", 0, 199, "borg_power.rs:575", "Power equipment speed tier base 16: existing policy speed."),
    power_equipment_speed_tier_slope_16 => (1000, "score", 0, 2000, "borg_power.rs:575", "Power equipment speed tier slope 16: existing policy score."),
    power_equipment_speed_tier_offset_16 => (75000, "score", 0, 150000, "borg_power.rs:575", "Power equipment speed tier offset 16: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_30 => (110, "speed", 0, 199, "borg_power.rs:577", "Power equipment reward higher for 10 than 50 speed dec threshold 30: existing policy speed."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_31 => (114, "speed", 0, 199, "borg_power.rs:577", "Power equipment reward higher for 10 than 50 speed dec threshold 31: existing policy speed."),
    power_equipment_speed_tier_base_17 => (110, "speed", 0, 199, "borg_power.rs:578", "Power equipment speed tier base 17: existing policy speed."),
    power_equipment_speed_tier_slope_17 => (1000, "score", 0, 2000, "borg_power.rs:578", "Power equipment speed tier slope 17: existing policy score."),
    power_equipment_speed_tier_offset_17 => (55000, "score", 0, 110000, "borg_power.rs:578", "Power equipment speed tier offset 17: existing policy score."),
    power_equipment_reward_higher_for_10_than_50_speed_dec_threshold_32 => (110, "speed", 0, 199, "borg_power.rs:580", "Power equipment reward higher for 10 than 50 speed dec threshold 32: existing policy speed."),
    power_equipment_speed_tier_base_18 => (110, "speed", 0, 199, "borg_power.rs:581", "Power equipment speed tier base 18: existing policy speed."),
    power_equipment_speed_tier_slope_18 => (2500, "score", 0, 5000, "borg_power.rs:581", "Power equipment speed tier slope 18: existing policy score."),
    power_equipment_speed_tier_offset_18 => (0, "score", 0, 100, "borg_power.rs:581", "Power equipment speed tier offset 18: existing policy score."),
    power_equipment_str_index_weight => (100, "score", 0, 200, "borg_power.rs:586", "Power equipment str index weight: existing policy score."),
    power_equipment_str_index_weight_2 => (500, "score", 0, 1000, "borg_power.rs:591", "Power equipment str index weight 2: existing policy score."),
    power_equipment_other_fail_rates_threshold => (1, "count", 0, 255, "borg_power.rs:609", "Power equipment other fail rates threshold: existing policy count."),
    power_equipment_other_fail_rates_weight => (30000, "score", 0, 60000, "borg_power.rs:610", "Power equipment other fail rates weight: existing policy score."),
    power_equipment_dex_index_weight => (120, "score", 0, 240, "borg_power.rs:618", "Power equipment dex index weight: existing policy score."),
    power_equipment_con_index_weight => (250, "score", 0, 500, "borg_power.rs:624", "Power equipment con index weight: existing policy score."),
    power_equipment_hp_adj_threshold => (800, "hp", 0, 10000, "borg_power.rs:629", "Power equipment hp adj threshold: existing policy hp."),
    power_equipment_hp_adj_weight => (450, "score", 0, 900, "borg_power.rs:630", "Power equipment hp adj weight: existing policy score."),
    power_equipment_hp_adj_weight_2 => (800, "score", 0, 1600, "borg_power.rs:632", "Power equipment hp adj weight 2: existing policy score."),
    power_equipment_hp_adj_weight_3 => (100, "score", 0, 200, "borg_power.rs:632", "Power equipment hp adj weight 3: existing policy score."),
    power_equipment_hp_adj_weight_4 => (350, "score", 0, 700, "borg_power.rs:632", "Power equipment hp adj weight 4: existing policy score."),
    power_equipment_hp_adj_weight_5 => (500, "score", 0, 1000, "borg_power.rs:632", "Power equipment hp adj weight 5: existing policy score."),
    power_equipment_con_index_weight_2 => (150, "score", 0, 300, "borg_power.rs:636", "Power equipment con index weight 2: existing policy score."),
    power_equipment_hp_adj_threshold_2 => (500, "hp", 0, 10000, "borg_power.rs:641", "Power equipment hp adj threshold 2: existing policy hp."),
    power_equipment_hp_adj_weight_6 => (350, "score", 0, 700, "borg_power.rs:642", "Power equipment hp adj weight 6: existing policy score."),
    power_equipment_hp_adj_weight_7 => (500, "score", 0, 1000, "borg_power.rs:644", "Power equipment hp adj weight 7: existing policy score."),
    power_equipment_hp_adj_weight_8 => (100, "score", 0, 200, "borg_power.rs:644", "Power equipment hp adj weight 8: existing policy score."),
    power_equipment_hp_adj_weight_9 => (350, "score", 0, 700, "borg_power.rs:644", "Power equipment hp adj weight 9: existing policy score."),
    power_equipment_hp_adj_weight_10 => (500, "score", 0, 1000, "borg_power.rs:644", "Power equipment hp adj weight 10: existing policy score."),
    power_equipment_disp_weight => (2, "score", 0, 100, "borg_power.rs:660", "Power equipment disp weight: existing policy score."),
    power_equipment_dism_weight => (2, "score", 0, 100, "borg_power.rs:661", "Power equipment dism weight: existing policy score."),
    power_equipment_dev_weight => (25, "score", 0, 100, "borg_power.rs:662", "Power equipment dev weight: existing policy score."),
    power_equipment_sav_weight => (25, "score", 0, 100, "borg_power.rs:663", "Power equipment sav weight: existing policy score."),
    power_equipment_sav_threshold => (99, "count", 0, 255, "borg_power.rs:665", "Power equipment sav threshold: existing policy count."),
    power_equipment_sav_weight_2 => (10000, "score", 0, 20000, "borg_power.rs:666", "Power equipment sav weight 2: existing policy score."),
    power_equipment_stl_weight => (2, "score", 0, 100, "borg_power.rs:668", "Power equipment stl weight: existing policy score."),
    power_equipment_thn_weight => (5, "score", 0, 100, "borg_power.rs:670", "Power equipment thn weight: existing policy score."),
    power_equipment_thb_weight => (35, "score", 0, 100, "borg_power.rs:671", "Power equipment thb weight: existing policy score."),
    power_equipment_tht_weight => (2, "score", 0, 100, "borg_power.rs:672", "Power equipment tht weight: existing policy score."),
    power_equipment_dig_weight => (2, "score", 0, 100, "borg_power.rs:673", "Power equipment dig weight: existing policy score."),
    power_equipment_sdig_weight => (750, "score", 0, 1500, "borg_power.rs:679", "Power equipment sdig weight: existing policy score."),
    power_equipment_ishungry_weight => (7500, "score", 0, 15000, "borg_power.rs:682", "Power equipment ishungry weight: existing policy score."),
    power_equipment_isweak_weight => (7500, "score", 0, 15000, "borg_power.rs:685", "Power equipment isweak weight: existing policy score."),
    power_equipment_maxdepth_threshold_2 => (20, "depth", 0, 127, "borg_power.rs:689", "Power equipment maxdepth threshold 2: existing policy depth."),
    power_equipment_feath_weight => (500, "score", 0, 1000, "borg_power.rs:691", "Power equipment feath weight: existing policy score."),
    power_equipment_feath_weight_2 => (50, "score", 0, 100, "borg_power.rs:694", "Power equipment feath weight 2: existing policy score."),
    power_equipment_light_weight_4 => (2000, "score", 0, 4000, "borg_power.rs:698", "Power equipment light weight 4: existing policy score."),
    power_equipment_sinv_weight => (500, "score", 0, 1000, "borg_power.rs:702", "Power equipment sinv weight: existing policy score."),
    power_equipment_sinv_weight_2 => (5000, "score", 0, 10000, "borg_power.rs:706", "Power equipment sinv weight 2: existing policy score."),
    power_equipment_fract_weight => (10000, "score", 0, 20000, "borg_power.rs:710", "Power equipment fract weight: existing policy score."),
    power_equipment_hlife_weight => (2000, "score", 0, 4000, "borg_power.rs:716", "Power equipment hlife weight: existing policy score."),
    power_equipment_hlife_weight_2 => (200, "score", 0, 400, "borg_power.rs:719", "Power equipment hlife weight 2: existing policy score."),
    power_equipment_reg_weight => (2000, "score", 0, 4000, "borg_power.rs:722", "Power equipment reg weight: existing policy score."),
    power_equipment_esp_weight => (80000, "score", 0, 160000, "borg_power.rs:725", "Power equipment esp weight: existing policy score."),
    power_equipment_icold_weight => (65000, "score", 0, 130000, "borg_power.rs:730", "Power equipment icold weight: existing policy score."),
    power_equipment_ielec_weight => (40000, "score", 0, 80000, "borg_power.rs:733", "Power equipment ielec weight: existing policy score."),
    power_equipment_ifire_weight => (80000, "score", 0, 160000, "borg_power.rs:736", "Power equipment ifire weight: existing policy score."),
    power_equipment_iacid_weight => (50000, "score", 0, 100000, "borg_power.rs:739", "Power equipment iacid weight: existing policy score."),
    power_equipment_rcold_weight => (3000, "score", 0, 6000, "borg_power.rs:744", "Power equipment rcold weight: existing policy score."),
    power_equipment_relec_weight => (4000, "score", 0, 8000, "borg_power.rs:747", "Power equipment relec weight: existing policy score."),
    power_equipment_racid_weight => (6000, "score", 0, 12000, "borg_power.rs:750", "Power equipment racid weight: existing policy score."),
    power_equipment_rfire_weight => (8000, "score", 0, 16000, "borg_power.rs:753", "Power equipment rfire weight: existing policy score."),
    power_equipment_rcold_weight_2 => (10000, "score", 0, 20000, "borg_power.rs:757", "Power equipment rcold weight 2: existing policy score."),
    power_equipment_rpois_weight => (20000, "score", 0, 40000, "borg_power.rs:760", "Power equipment rpois weight: existing policy score."),
    power_equipment_rsnd_weight => (3500, "score", 0, 7000, "borg_power.rs:763", "Power equipment rsnd weight: existing policy score."),
    power_equipment_rlite_weight => (800, "score", 0, 1600, "borg_power.rs:766", "Power equipment rlite weight: existing policy score."),
    power_equipment_rdark_weight => (800, "score", 0, 1600, "borg_power.rs:769", "Power equipment rdark weight: existing policy score."),
    power_equipment_rkaos_weight => (5000, "score", 0, 10000, "borg_power.rs:772", "Power equipment rkaos weight: existing policy score."),
    power_equipment_rconf_weight => (80000, "score", 0, 160000, "borg_power.rs:777", "Power equipment rconf weight: existing policy score."),
    power_equipment_rconf_weight_2 => (2000, "score", 0, 4000, "borg_power.rs:782", "Power equipment rconf weight 2: existing policy score."),
    power_equipment_rdis_weight => (5000, "score", 0, 10000, "borg_power.rs:785", "Power equipment rdis weight: existing policy score."),
    power_equipment_rshrd_weight => (100, "score", 0, 200, "borg_power.rs:788", "Power equipment rshrd weight: existing policy score."),
    power_equipment_rnxus_weight => (100, "score", 0, 200, "borg_power.rs:791", "Power equipment rnxus weight: existing policy score."),
    power_equipment_rblind_weight => (5000, "score", 0, 10000, "borg_power.rs:794", "Power equipment rblind weight: existing policy score."),
    power_equipment_rnthr_weight => (5500, "score", 0, 11000, "borg_power.rs:797", "Power equipment rnthr weight: existing policy score."),
    power_equipment_rfear_weight => (2000, "score", 0, 4000, "borg_power.rs:800", "Power equipment rfear weight: existing policy score."),
    power_equipment_sstr_weight => (50, "score", 0, 100, "borg_power.rs:805", "Power equipment sstr weight: existing policy score."),
    power_equipment_sint_weight => (50, "score", 0, 100, "borg_power.rs:808", "Power equipment sint weight: existing policy score."),
    power_equipment_swis_weight => (50, "score", 0, 100, "borg_power.rs:811", "Power equipment swis weight: existing policy score."),
    power_equipment_scon_weight => (50, "score", 0, 100, "borg_power.rs:814", "Power equipment scon weight: existing policy score."),
    power_equipment_sdex_weight => (50, "score", 0, 100, "borg_power.rs:817", "Power equipment sdex weight: existing policy score."),
    power_equipment_sstr_weight_2 => (1000, "score", 0, 2000, "borg_power.rs:826", "Power equipment sstr weight 2: existing policy score."),
    power_equipment_esp_threshold => (10, "depth", 0, 127, "borg_power.rs:835", "Power equipment esp threshold: existing policy depth."),
    power_equipment_esp_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:836", "Power equipment esp weight 2: existing policy score."),
    power_equipment_fract_threshold => (20, "depth", 0, 127, "borg_power.rs:840", "Power equipment fract threshold: existing policy depth."),
    power_equipment_fract_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:841", "Power equipment fract weight 2: existing policy score."),
    power_equipment_rfire_threshold => (25, "depth", 0, 127, "borg_power.rs:845", "Power equipment rfire threshold: existing policy depth."),
    power_equipment_rfire_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:846", "Power equipment rfire weight 2: existing policy score."),
    power_equipment_rpois_threshold => (40, "depth", 0, 127, "borg_power.rs:850", "Power equipment rpois threshold: existing policy depth."),
    power_equipment_rpois_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:851", "Power equipment rpois weight 2: existing policy score."),
    power_equipment_relec_threshold => (40, "depth", 0, 127, "borg_power.rs:853", "Power equipment relec threshold: existing policy depth."),
    power_equipment_relec_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:854", "Power equipment relec weight 2: existing policy score."),
    power_equipment_racid_threshold => (40, "depth", 0, 127, "borg_power.rs:856", "Power equipment racid threshold: existing policy depth."),
    power_equipment_racid_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:857", "Power equipment racid weight 2: existing policy score."),
    power_equipment_rcold_threshold => (40, "depth", 0, 127, "borg_power.rs:859", "Power equipment rcold threshold: existing policy depth."),
    power_equipment_rcold_weight_3 => (100000, "score", 0, 200000, "borg_power.rs:860", "Power equipment rcold weight 3: existing policy score."),
    power_equipment_maxclevel_weight => (100000, "score", 0, 200000, "borg_power.rs:865", "Power equipment maxclevel weight: existing policy score."),
    power_equipment_speed_threshold => (115, "speed", 0, 199, "borg_power.rs:867", "Power equipment speed threshold: existing policy speed."),
    power_equipment_speed_threshold_2 => (46, "speed", 0, 199, "borg_power.rs:867", "Power equipment speed threshold 2: existing policy speed."),
    power_equipment_speed_weight => (100000, "score", 0, 200000, "borg_power.rs:868", "Power equipment speed weight: existing policy score."),
    power_equipment_rconf_threshold => (46, "depth", 0, 127, "borg_power.rs:870", "Power equipment rconf threshold: existing policy depth."),
    power_equipment_rconf_weight_3 => (100000, "score", 0, 200000, "borg_power.rs:871", "Power equipment rconf weight 3: existing policy score."),
    power_equipment_rnthr_threshold => (50, "depth", 0, 127, "borg_power.rs:875", "Power equipment rnthr threshold: existing policy depth."),
    power_equipment_rnthr_weight_2 => (55000, "score", 0, 110000, "borg_power.rs:876", "Power equipment rnthr weight 2: existing policy score."),
    power_equipment_rsnd_threshold => (50, "depth", 0, 127, "borg_power.rs:880", "Power equipment rsnd threshold: existing policy depth."),
    power_equipment_rsnd_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:881", "Power equipment rsnd weight 2: existing policy score."),
    power_equipment_rblind_threshold => (55, "depth", 0, 127, "borg_power.rs:885", "Power equipment rblind threshold: existing policy depth."),
    power_equipment_rblind_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:886", "Power equipment rblind weight 2: existing policy score."),
    power_equipment_esp_threshold_2 => (55, "depth", 0, 127, "borg_power.rs:888", "Power equipment esp threshold 2: existing policy depth."),
    power_equipment_esp_weight_3 => (100000, "score", 0, 200000, "borg_power.rs:889", "Power equipment esp weight 3: existing policy score."),
    power_equipment_rnthr_threshold_2 => (60, "depth", 0, 127, "borg_power.rs:891", "Power equipment rnthr threshold 2: existing policy depth."),
    power_equipment_rnthr_weight_3 => (55000, "score", 0, 110000, "borg_power.rs:892", "Power equipment rnthr weight 3: existing policy score."),
    power_equipment_rkaos_threshold => (60, "depth", 0, 127, "borg_power.rs:896", "Power equipment rkaos threshold: existing policy depth."),
    power_equipment_rkaos_weight_2 => (104000, "score", 0, 208000, "borg_power.rs:897", "Power equipment rkaos weight 2: existing policy score."),
    power_equipment_rdis_threshold => (60, "depth", 0, 127, "borg_power.rs:899", "Power equipment rdis threshold: existing policy depth."),
    power_equipment_rdis_weight_2 => (90000, "score", 0, 180000, "borg_power.rs:900", "Power equipment rdis weight 2: existing policy score."),
    power_equipment_speed_threshold_3 => (120, "speed", 0, 199, "borg_power.rs:902", "Power equipment speed threshold 3: existing policy speed."),
    power_equipment_speed_threshold_4 => (60, "speed", 0, 199, "borg_power.rs:902", "Power equipment speed threshold 4: existing policy speed."),
    power_equipment_speed_weight_2 => (100000, "score", 0, 200000, "borg_power.rs:903", "Power equipment speed weight 2: existing policy score."),
    power_equipment_speed_threshold_5 => (130, "speed", 0, 199, "borg_power.rs:907", "Power equipment speed threshold 5: existing policy speed."),
    power_equipment_speed_threshold_6 => (80, "speed", 0, 199, "borg_power.rs:907", "Power equipment speed threshold 6: existing policy speed."),
    power_equipment_speed_weight_3 => (100000, "score", 0, 200000, "borg_power.rs:908", "Power equipment speed weight 3: existing policy score."),
    power_equipment_rnthr_threshold_3 => (80, "depth", 0, 127, "borg_power.rs:915", "Power equipment rnthr threshold 3: existing policy depth."),
    power_equipment_rnthr_weight_4 => (15000, "score", 0, 30000, "borg_power.rs:916", "Power equipment rnthr weight 4: existing policy score."),
    power_equipment_rdark_threshold => (80, "depth", 0, 127, "borg_power.rs:918", "Power equipment rdark threshold: existing policy depth."),
    power_equipment_rdark_weight_2 => (25000, "score", 0, 50000, "borg_power.rs:919", "Power equipment rdark weight 2: existing policy score."),
    power_equipment_speed_threshold_7 => (140, "speed", 0, 199, "borg_power.rs:921", "Power equipment speed threshold 7: existing policy speed."),
    power_equipment_speed_threshold_8 => (80, "speed", 0, 199, "borg_power.rs:921", "Power equipment speed threshold 8: existing policy speed."),
    power_equipment_class_weight => (100000, "score", 0, 200000, "borg_power.rs:922", "Power equipment class weight: existing policy score."),
    power_equipment_reward_armor_threshold => (15, "armor", 0, 500, "borg_power.rs:930", "Power equipment reward armor threshold: existing policy armor."),
    power_equipment_reward_armor_weight => (2500, "score", 0, 5000, "borg_power.rs:931", "Power equipment reward armor weight: existing policy score."),
    power_equipment_reward_armor_threshold_2 => (15, "armor", 0, 500, "borg_power.rs:933", "Power equipment reward armor threshold 2: existing policy armor."),
    power_equipment_reward_armor_threshold_3 => (75, "armor", 0, 500, "borg_power.rs:933", "Power equipment reward armor threshold 3: existing policy armor."),
    power_equipment_reward_armor_weight_2 => (2000, "score", 0, 4000, "borg_power.rs:934", "Power equipment reward armor weight 2: existing policy score."),
    power_equipment_reward_armor_weight_3 => (28250, "score", 0, 56500, "borg_power.rs:934", "Power equipment reward armor weight 3: existing policy score."),
    power_equipment_reward_armor_threshold_4 => (75, "armor", 0, 500, "borg_power.rs:936", "Power equipment reward armor threshold 4: existing policy armor."),
    power_equipment_reward_armor_weight_4 => (1500, "score", 0, 3000, "borg_power.rs:937", "Power equipment reward armor weight 4: existing policy score."),
    power_equipment_reward_armor_weight_5 => (73750, "score", 0, 147500, "borg_power.rs:937", "Power equipment reward armor weight 5: existing policy score."),
    power_equipment_reward_armor_threshold_5 => (15, "armor", 0, 500, "borg_power.rs:940", "Power equipment reward armor threshold 5: existing policy armor."),
    power_equipment_reward_armor_weight_6 => (2000, "score", 0, 4000, "borg_power.rs:941", "Power equipment reward armor weight 6: existing policy score."),
    power_equipment_reward_armor_threshold_6 => (15, "armor", 0, 500, "borg_power.rs:943", "Power equipment reward armor threshold 6: existing policy armor."),
    power_equipment_reward_armor_threshold_7 => (75, "armor", 0, 500, "borg_power.rs:943", "Power equipment reward armor threshold 7: existing policy armor."),
    power_equipment_reward_armor_weight_7 => (500, "score", 0, 1000, "borg_power.rs:944", "Power equipment reward armor weight 7: existing policy score."),
    power_equipment_reward_armor_weight_8 => (28350, "score", 0, 56700, "borg_power.rs:944", "Power equipment reward armor weight 8: existing policy score."),
    power_equipment_reward_armor_threshold_8 => (75, "armor", 0, 500, "borg_power.rs:946", "Power equipment reward armor threshold 8: existing policy armor."),
    power_equipment_reward_armor_weight_9 => (100, "score", 0, 200, "borg_power.rs:947", "Power equipment reward armor weight 9: existing policy score."),
    power_equipment_reward_armor_weight_10 => (73750, "score", 0, 147500, "borg_power.rs:947", "Power equipment reward armor weight 10: existing policy score."),
    power_equipment_crsagrv_weight => (800000, "score", 0, 1600000, "borg_power.rs:955", "Power equipment crsagrv weight: existing policy score."),
    power_equipment_crshpimp_weight => (35000, "score", 0, 70000, "borg_power.rs:958", "Power equipment crshpimp weight: existing policy score."),
    power_equipment_crsmpimp_weight => (15000, "score", 0, 30000, "borg_power.rs:961", "Power equipment crsmpimp weight: existing policy score."),
    power_equipment_class_weight_2 => (15000, "score", 0, 30000, "borg_power.rs:969", "Power equipment class weight 2: existing policy score."),
    power_equipment_crsfear_weight => (400000, "score", 0, 800000, "borg_power.rs:972", "Power equipment crsfear weight: existing policy score."),
    power_equipment_class_weight_3 => (200000, "score", 0, 400000, "borg_power.rs:975", "Power equipment class weight 3: existing policy score."),
    power_equipment_crsdrain_xp_weight => (400000, "score", 0, 800000, "borg_power.rs:978", "Power equipment crsdrain xp weight: existing policy score."),
    power_equipment_crsfvuln_weight => (30000, "score", 0, 60000, "borg_power.rs:981", "Power equipment crsfvuln weight: existing policy score."),
    power_equipment_crsevuln_weight => (30000, "score", 0, 60000, "borg_power.rs:984", "Power equipment crsevuln weight: existing policy score."),
    power_equipment_crscvuln_weight => (30000, "score", 0, 60000, "borg_power.rs:987", "Power equipment crscvuln weight: existing policy score."),
    power_equipment_crsavuln_weight => (30000, "score", 0, 60000, "borg_power.rs:990", "Power equipment crsavuln weight: existing policy score."),
    power_equipment_crstele_weight => (100000, "score", 0, 200000, "borg_power.rs:994", "Power equipment crstele weight: existing policy score."),
    power_equipment_crsenveloping_weight => (50000, "score", 0, 100000, "borg_power.rs:997", "Power equipment crsenveloping weight: existing policy score."),
    power_equipment_crsirritation_weight => (20000, "score", 0, 40000, "borg_power.rs:1000", "Power equipment crsirritation weight: existing policy score."),
    power_equipment_crspois_weight => (10000, "score", 0, 20000, "borg_power.rs:1003", "Power equipment crspois weight: existing policy score."),
    power_equipment_crssiren_weight => (800000, "score", 0, 1600000, "borg_power.rs:1006", "Power equipment crssiren weight: existing policy score."),
    power_equipment_crshalu_weight => (100000, "score", 0, 200000, "borg_power.rs:1009", "Power equipment crshalu weight: existing policy score."),
    power_equipment_crspara_weight => (800000, "score", 0, 1600000, "borg_power.rs:1012", "Power equipment crspara weight: existing policy score."),
    power_equipment_crssdem_weight => (100000, "score", 0, 200000, "borg_power.rs:1015", "Power equipment crssdem weight: existing policy score."),
    power_equipment_crssdra_weight => (100000, "score", 0, 200000, "borg_power.rs:1018", "Power equipment crssdra weight: existing policy score."),
    power_equipment_crssund_weight => (100000, "score", 0, 200000, "borg_power.rs:1021", "Power equipment crssund weight: existing policy score."),
    power_equipment_speed_threshold_9 => (140, "speed", 0, 199, "borg_power.rs:1023", "Power equipment speed threshold 9: existing policy speed."),
    power_equipment_speed_weight_4 => (10000, "score", 0, 20000, "borg_power.rs:1024", "Power equipment speed weight 4: existing policy score."),
    power_equipment_speed_threshold_10 => (140, "speed", 0, 199, "borg_power.rs:1026", "Power equipment speed threshold 10: existing policy speed."),
    power_equipment_speed_weight_5 => (10000, "score", 0, 20000, "borg_power.rs:1027", "Power equipment speed weight 5: existing policy score."),
    power_equipment_crsnotel_weight => (700000, "score", 0, 1400000, "borg_power.rs:1030", "Power equipment crsnotel weight: existing policy score."),
    power_equipment_crstwep_weight => (100000, "score", 0, 200000, "borg_power.rs:1033", "Power equipment crstwep weight: existing policy score."),
    power_equipment_crsairswing_weight => (10000, "score", 0, 20000, "borg_power.rs:1036", "Power equipment crsairswing weight: existing policy score."),
    power_equipment_crsunkno_weight => (9999999, "score", 0, 19999998, "borg_power.rs:1039", "Power equipment crsunkno weight: existing policy score."),
    power_equipment_multiple_bonuses_weight => (3000, "score", 0, 6000, "borg_power.rs:1045", "Power equipment multiple bonuses weight: existing policy score."),
    power_equipment_worn_need_id_weight => (10000, "score", 0, 20000, "borg_power.rs:1048", "Power equipment worn need id weight: existing policy score."),
    power_equipment_powers_weight => (999999, "score", 0, 1999998, "borg_power.rs:1067", "Power equipment powers weight: existing policy score."),
    power_equipment_str_index_threshold => (15, "count", 0, 255, "borg_power.rs:1075", "Power equipment str index threshold: existing policy count."),
    power_equipment_powers_threshold => (200, "count", 0, 400, "borg_power.rs:1076", "Power equipment powers threshold: existing policy count."),
    power_equipment_str_index_weight_3 => (200, "score", 0, 400, "borg_power.rs:1077", "Power equipment str index weight 3: existing policy score."),
    power_equipment_str_index_weight_4 => (15, "score", 0, 100, "borg_power.rs:1077", "Power equipment str index weight 4: existing policy score."),
    power_equipment_powers_threshold_2 => (30, "count", 0, 255, "borg_power.rs:1079", "Power equipment powers threshold 2: existing policy count."),
    power_equipment_str_index_weight_5 => (250, "score", 0, 500, "borg_power.rs:1080", "Power equipment str index weight 5: existing policy score."),
    power_equipment_powers_threshold_3 => (10, "count", 0, 255, "borg_power.rs:1082", "Power equipment powers threshold 3: existing policy count."),
    power_equipment_str_index_weight_6 => (250, "score", 0, 500, "borg_power.rs:1083", "Power equipment str index weight 6: existing policy score."),
    power_equipment_powers_threshold_4 => (50, "count", 0, 255, "borg_power.rs:1085", "Power equipment powers threshold 4: existing policy count."),
    power_equipment_str_index_weight_7 => (250, "score", 0, 500, "borg_power.rs:1086", "Power equipment str index weight 7: existing policy score."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold => (300, "mana", 0, 1000, "borg_power.rs:1109", "Power equipment mega hack penalize heavy armor which h threshold: existing policy mana."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_2 => (350, "mana", 0, 1000, "borg_power.rs:1109", "Power equipment mega hack penalize heavy armor which h threshold 2: existing policy mana."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_weight => (400, "score", 0, 800, "borg_power.rs:1110", "Power equipment mega hack penalize heavy armor which h weight: existing policy score."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_3 => (200, "mana", 0, 1000, "borg_power.rs:1112", "Power equipment mega hack penalize heavy armor which h threshold 3: existing policy mana."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_4 => (299, "mana", 0, 1000, "borg_power.rs:1112", "Power equipment mega hack penalize heavy armor which h threshold 4: existing policy mana."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_2 => (800, "score", 0, 1600, "borg_power.rs:1113", "Power equipment mega hack penalize heavy armor which h weight 2: existing policy score."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_5 => (100, "mana", 0, 1000, "borg_power.rs:1115", "Power equipment mega hack penalize heavy armor which h threshold 5: existing policy mana."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_6 => (199, "mana", 0, 1000, "borg_power.rs:1115", "Power equipment mega hack penalize heavy armor which h threshold 6: existing policy mana."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_3 => (1600, "score", 0, 3200, "borg_power.rs:1116", "Power equipment mega hack penalize heavy armor which h weight 3: existing policy score."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_threshold_7 => (99, "mana", 0, 1000, "borg_power.rs:1118", "Power equipment mega hack penalize heavy armor which h threshold 7: existing policy mana."),
    power_equipment_mega_hack_penalize_heavy_armor_which_h_weight_4 => (3200, "score", 0, 6400, "borg_power.rs:1119", "Power equipment mega hack penalize heavy armor which h weight 4: existing policy score."),
    power_inventory_afuel_limit => (6, "count", 0, 255, "borg_power.rs:1146", "Power inventory afuel limit: existing policy count."),
    power_inventory_afuel_weight => (60000, "score", 0, 120000, "borg_power.rs:1146", "Power inventory afuel weight: existing policy score."),
    power_inventory_str_threshold => (15, "stat", 0, 238, "borg_power.rs:1147", "Power inventory str threshold: existing policy stat."),
    power_inventory_reward_fuel_threshold => (10, "count", 0, 255, "borg_power.rs:1148", "Power inventory reward fuel threshold: existing policy count."),
    power_inventory_food_limit => (7, "count", 0, 255, "borg_power.rs:1160", "Power inventory food limit: existing policy count."),
    power_inventory_food_weight => (50000, "score", 0, 100000, "borg_power.rs:1160", "Power inventory food weight: existing policy score."),
    power_inventory_str_threshold_2 => (15, "stat", 0, 238, "borg_power.rs:1161", "Power inventory str threshold 2: existing policy stat."),
    power_inventory_food_limit_2 => (10, "count", 0, 255, "borg_power.rs:1162", "Power inventory food limit 2: existing policy count."),
    power_inventory_food_weight_2 => (200, "score", 0, 400, "borg_power.rs:1162", "Power inventory food weight 2: existing policy score."),
    power_inventory_clevel_threshold => (15, "level", 0, 51, "borg_power.rs:1164", "Power inventory clevel threshold: existing policy level."),
    power_inventory_food_limit_3 => (15, "count", 0, 255, "borg_power.rs:1166", "Power inventory food limit 3: existing policy count."),
    power_inventory_food_weight_3 => (700, "score", 0, 1400, "borg_power.rs:1166", "Power inventory food weight 3: existing policy score."),
    power_inventory_food_hi_limit => (7, "count", 0, 255, "borg_power.rs:1171", "Power inventory food hi limit: existing policy count."),
    power_inventory_food_hi_weight => (52, "score", 0, 104, "borg_power.rs:1171", "Power inventory food hi weight: existing policy score."),
    power_inventory_food_lo_limit => (15, "count", 0, 255, "borg_power.rs:1173", "Power inventory food lo limit: existing policy count."),
    power_inventory_food_lo_weight => (2, "score", 0, 100, "borg_power.rs:1173", "Power inventory food lo weight: existing policy score."),
    power_inventory_acsw_limit => (5, "count", 0, 255, "borg_power.rs:1186", "Power inventory acsw limit: existing policy count."),
    power_inventory_acsw_weight => (25000, "score", 0, 50000, "borg_power.rs:1186", "Power inventory acsw weight: existing policy score."),
    power_inventory_acurepois_threshold => (20, "count", 0, 255, "borg_power.rs:1193", "Power inventory acurepois threshold: existing policy count."),
    power_inventory_arespois_limit => (4, "count", 0, 255, "borg_power.rs:1195", "Power inventory arespois limit: existing policy count."),
    power_inventory_arespois_weight => (300, "score", 0, 600, "borg_power.rs:1195", "Power inventory arespois weight: existing policy score."),
    power_inventory_maxdepth_threshold => (20, "depth", 0, 127, "borg_power.rs:1199", "Power inventory maxdepth threshold: existing policy depth."),
    power_inventory_aresheat_limit => (4, "count", 0, 255, "borg_power.rs:1204", "Power inventory aresheat limit: existing policy count."),
    power_inventory_aresheat_weight => (500, "score", 0, 1000, "borg_power.rs:1204", "Power inventory aresheat weight: existing policy score."),
    power_inventory_arescold_limit => (4, "count", 0, 255, "borg_power.rs:1209", "Power inventory arescold limit: existing policy count."),
    power_inventory_arescold_weight => (500, "score", 0, 1000, "borg_power.rs:1209", "Power inventory arescold weight: existing policy score."),
    power_inventory_arespois_limit_2 => (4, "count", 0, 255, "borg_power.rs:1214", "Power inventory arespois limit 2: existing policy count."),
    power_inventory_arespois_weight_2 => (500, "score", 0, 1000, "borg_power.rs:1214", "Power inventory arespois weight 2: existing policy score."),
    power_inventory_clevel_threshold_2 => (10, "level", 0, 51, "borg_power.rs:1221", "Power inventory clevel threshold 2: existing policy level."),
    power_inventory_aid_limit => (5, "count", 0, 255, "borg_power.rs:1222", "Power inventory aid limit: existing policy count."),
    power_inventory_aid_weight => (6000, "score", 0, 12000, "borg_power.rs:1222", "Power inventory aid weight: existing policy score."),
    power_inventory_str_threshold_3 => (15, "stat", 0, 238, "borg_power.rs:1223", "Power inventory str threshold 3: existing policy stat."),
    power_inventory_aid_limit_2 => (15, "count", 0, 255, "borg_power.rs:1224", "Power inventory aid limit 2: existing policy count."),
    power_inventory_aid_weight_2 => (600, "score", 0, 1200, "borg_power.rs:1224", "Power inventory aid weight 2: existing policy score."),
    power_inventory_apfe_limit => (10, "count", 0, 255, "borg_power.rs:1239", "Power inventory apfe limit: existing policy count."),
    power_inventory_apfe_weight => (10000, "score", 0, 20000, "borg_power.rs:1239", "Power inventory apfe weight: existing policy score."),
    power_inventory_apfe_limit_2 => (25, "count", 0, 255, "borg_power.rs:1240", "Power inventory apfe limit 2: existing policy count."),
    power_inventory_apfe_weight_2 => (2000, "score", 0, 4000, "borg_power.rs:1240", "Power inventory apfe weight 2: existing policy score."),
    power_inventory_aglyph_limit => (10, "count", 0, 255, "borg_power.rs:1244", "Power inventory aglyph limit: existing policy count."),
    power_inventory_aglyph_weight => (10000, "score", 0, 20000, "borg_power.rs:1244", "Power inventory aglyph weight: existing policy score."),
    power_inventory_aglyph_limit_2 => (25, "count", 0, 255, "borg_power.rs:1245", "Power inventory aglyph limit 2: existing policy count."),
    power_inventory_aglyph_weight_2 => (2000, "score", 0, 4000, "borg_power.rs:1245", "Power inventory aglyph weight 2: existing policy score."),
    power_inventory_maxdepth_threshold_2 => (100, "depth", 0, 127, "borg_power.rs:1246", "Power inventory maxdepth threshold 2: existing policy depth."),
    power_inventory_aglyph_limit_3 => (10, "count", 0, 255, "borg_power.rs:1248", "Power inventory aglyph limit 3: existing policy count."),
    power_inventory_aglyph_weight_3 => (2500, "score", 0, 5000, "borg_power.rs:1248", "Power inventory aglyph weight 3: existing policy score."),
    power_inventory_aglyph_limit_4 => (75, "count", 0, 255, "borg_power.rs:1249", "Power inventory aglyph limit 4: existing policy count."),
    power_inventory_aglyph_weight_4 => (2500, "score", 0, 5000, "borg_power.rs:1249", "Power inventory aglyph weight 4: existing policy score."),
    power_inventory_maxdepth_threshold_3 => (100, "depth", 0, 127, "borg_power.rs:1253", "Power inventory maxdepth threshold 3: existing policy depth."),
    power_inventory_amassban_limit => (99, "count", 0, 255, "borg_power.rs:1255", "Power inventory amassban limit: existing policy count."),
    power_inventory_amassban_weight => (2500, "score", 0, 5000, "borg_power.rs:1255", "Power inventory amassban weight: existing policy score."),
    power_inventory_clevel_threshold_3 => (7, "level", 0, 51, "borg_power.rs:1260", "Power inventory clevel threshold 3: existing policy level."),
    power_inventory_recall_limit => (3, "count", 0, 255, "borg_power.rs:1261", "Power inventory recall limit: existing policy count."),
    power_inventory_recall_weight => (50000, "score", 0, 100000, "borg_power.rs:1261", "Power inventory recall weight: existing policy score."),
    power_inventory_str_threshold_4 => (15, "stat", 0, 238, "borg_power.rs:1262", "Power inventory str threshold 4: existing policy stat."),
    power_inventory_recall_limit_2 => (7, "count", 0, 255, "borg_power.rs:1263", "Power inventory recall limit 2: existing policy count."),
    power_inventory_recall_weight_2 => (5000, "score", 0, 10000, "borg_power.rs:1263", "Power inventory recall weight 2: existing policy score."),
    power_inventory_maxdepth_threshold_4 => (50, "depth", 0, 127, "borg_power.rs:1266", "Power inventory maxdepth threshold 4: existing policy depth."),
    power_inventory_aphase_limit => (8, "count", 0, 255, "borg_power.rs:1278", "Power inventory aphase limit: existing policy count."),
    power_inventory_aphase_weight => (500, "score", 0, 1000, "borg_power.rs:1278", "Power inventory aphase weight: existing policy score."),
    power_inventory_str_threshold_5 => (15, "stat", 0, 238, "borg_power.rs:1279", "Power inventory str threshold 5: existing policy stat."),
    power_inventory_aphase_limit_2 => (15, "count", 0, 255, "borg_power.rs:1280", "Power inventory aphase limit 2: existing policy count."),
    power_inventory_aphase_weight_2 => (500, "score", 0, 1000, "borg_power.rs:1280", "Power inventory aphase weight 2: existing policy score."),
    power_inventory_aescape_limit => (2, "count", 0, 255, "borg_power.rs:1287", "Power inventory aescape limit: existing policy count."),
    power_inventory_aescape_weight => (10000, "score", 0, 20000, "borg_power.rs:1287", "Power inventory aescape weight: existing policy score."),
    power_inventory_maxdepth_threshold_5 => (70, "depth", 0, 127, "borg_power.rs:1288", "Power inventory maxdepth threshold 5: existing policy depth."),
    power_inventory_aescape_limit_2 => (3, "count", 0, 255, "borg_power.rs:1290", "Power inventory aescape limit 2: existing policy count."),
    power_inventory_aescape_weight_2 => (10000, "score", 0, 20000, "borg_power.rs:1290", "Power inventory aescape weight 2: existing policy score."),
    power_inventory_clevel_threshold_4 => (3, "level", 0, 51, "borg_power.rs:1296", "Power inventory clevel threshold 4: existing policy level."),
    power_inventory_clevel_threshold_5 => (7, "level", 0, 51, "borg_power.rs:1299", "Power inventory clevel threshold 5: existing policy level."),
    power_inventory_ateleport_limit => (3, "count", 0, 255, "borg_power.rs:1300", "Power inventory ateleport limit: existing policy count."),
    power_inventory_ateleport_weight => (10000, "score", 0, 20000, "borg_power.rs:1300", "Power inventory ateleport weight: existing policy score."),
    power_inventory_clevel_threshold_6 => (30, "level", 0, 51, "borg_power.rs:1302", "Power inventory clevel threshold 6: existing policy level."),
    power_inventory_ateleport_limit_2 => (10, "count", 0, 255, "borg_power.rs:1303", "Power inventory ateleport limit 2: existing policy count."),
    power_inventory_ateleport_weight_2 => (10000, "score", 0, 20000, "borg_power.rs:1303", "Power inventory ateleport weight 2: existing policy score."),
    power_inventory_clevel_threshold_7 => (15, "level", 0, 51, "borg_power.rs:1308", "Power inventory clevel threshold 7: existing policy level."),
    power_inventory_ateleportlvl_limit => (5, "count", 0, 255, "borg_power.rs:1309", "Power inventory ateleportlvl limit: existing policy count."),
    power_inventory_ateleportlvl_weight => (5000, "score", 0, 10000, "borg_power.rs:1309", "Power inventory ateleportlvl weight: existing policy score."),
    power_inventory_maxdepth_threshold_6 => (46, "depth", 0, 127, "borg_power.rs:1315", "Power inventory maxdepth threshold 6: existing policy depth."),
    power_inventory_aezheal_limit => (1, "count", 0, 255, "borg_power.rs:1317", "Power inventory aezheal limit: existing policy count."),
    power_inventory_aezheal_weight => (10000, "score", 0, 20000, "borg_power.rs:1317", "Power inventory aezheal weight: existing policy score."),
    power_inventory_aezheal_limit_2 => (2, "count", 0, 255, "borg_power.rs:1319", "Power inventory aezheal limit 2: existing policy count."),
    power_inventory_aezheal_weight_2 => (10000, "score", 0, 20000, "borg_power.rs:1319", "Power inventory aezheal weight 2: existing policy score."),
    power_inventory_aheal_limit => (15, "count", 0, 255, "borg_power.rs:1329", "Power inventory aheal limit: existing policy count."),
    power_inventory_aheal_weight => (8000, "score", 0, 16000, "borg_power.rs:1329", "Power inventory aheal weight: existing policy score."),
    power_inventory_rod_healing_limit => (6, "count", 0, 255, "borg_power.rs:1335", "Power inventory rod healing limit: existing policy count."),
    power_inventory_rod_healing_weight => (20000, "score", 0, 40000, "borg_power.rs:1335", "Power inventory rod healing weight: existing policy score."),
    power_inventory_aheal_limit_2 => (10, "count", 0, 255, "borg_power.rs:1342", "Power inventory aheal limit 2: existing policy count."),
    power_inventory_aheal_weight_2 => (4000, "score", 0, 8000, "borg_power.rs:1342", "Power inventory aheal weight 2: existing policy score."),
    power_inventory_potion_healing_limit => (3, "count", 0, 255, "borg_power.rs:1349", "Power inventory potion healing limit: existing policy count."),
    power_inventory_potion_healing_weight => (5000, "score", 0, 10000, "borg_power.rs:1349", "Power inventory potion healing weight: existing policy score."),
    power_inventory_rod_healing_limit_2 => (4, "count", 0, 255, "borg_power.rs:1354", "Power inventory rod healing limit 2: existing policy count."),
    power_inventory_rod_healing_weight_2 => (20000, "score", 0, 40000, "borg_power.rs:1354", "Power inventory rod healing weight 2: existing policy score."),
    power_inventory_clevel_threshold_8 => (1, "level", 0, 51, "borg_power.rs:1359", "Power inventory clevel threshold 8: existing policy level."),
    power_inventory_potion_healing_limit_2 => (10, "count", 0, 255, "borg_power.rs:1361", "Power inventory potion healing limit 2: existing policy count."),
    power_inventory_potion_healing_weight_2 => (2000, "score", 0, 4000, "borg_power.rs:1361", "Power inventory potion healing weight 2: existing policy score."),
    power_inventory_potion_healing_limit_3 => (5, "count", 0, 255, "borg_power.rs:1365", "Power inventory potion healing limit 3: existing policy count."),
    power_inventory_potion_healing_weight_3 => (2000, "score", 0, 4000, "borg_power.rs:1365", "Power inventory potion healing weight 3: existing policy score."),
    power_inventory_maxdepth_threshold_7 => (99, "depth", 0, 127, "borg_power.rs:1373", "Power inventory maxdepth threshold 7: existing policy depth."),
    power_inventory_potion_healing_limit_4 => (99, "count", 0, 255, "borg_power.rs:1375", "Power inventory potion healing limit 4: existing policy count."),
    power_inventory_potion_healing_weight_4 => (8000, "score", 0, 16000, "borg_power.rs:1375", "Power inventory potion healing weight 4: existing policy score."),
    power_inventory_aezheal_limit_3 => (99, "count", 0, 255, "borg_power.rs:1377", "Power inventory aezheal limit 3: existing policy count."),
    power_inventory_aezheal_weight_3 => (10000, "score", 0, 20000, "borg_power.rs:1377", "Power inventory aezheal weight 3: existing policy score."),
    power_inventory_aspeed_limit => (99, "count", 0, 255, "borg_power.rs:1379", "Power inventory aspeed limit: existing policy count."),
    power_inventory_aspeed_weight => (8000, "score", 0, 16000, "borg_power.rs:1379", "Power inventory aspeed weight: existing policy score."),
    power_inventory_alife_limit => (99, "count", 0, 255, "borg_power.rs:1381", "Power inventory alife limit: existing policy count."),
    power_inventory_alife_weight => (10000, "score", 0, 20000, "borg_power.rs:1381", "Power inventory alife weight: existing policy score."),
    power_inventory_potion_restore_mana_limit => (99, "count", 0, 255, "borg_power.rs:1384", "Power inventory potion restore mana limit: existing policy count."),
    power_inventory_potion_restore_mana_weight => (5000, "score", 0, 10000, "borg_power.rs:1384", "Power inventory potion restore mana weight: existing policy score."),
    power_inventory_mush_stoneskin_limit => (40, "count", 0, 255, "borg_power.rs:1387", "Power inventory mush stoneskin limit: existing policy count."),
    power_inventory_mush_stoneskin_weight => (5000, "score", 0, 10000, "borg_power.rs:1387", "Power inventory mush stoneskin weight: existing policy score."),
    power_inventory_amassban_limit_2 => (99, "count", 0, 255, "borg_power.rs:1393", "Power inventory amassban limit 2: existing policy count."),
    power_inventory_amassban_weight_2 => (2500, "score", 0, 5000, "borg_power.rs:1393", "Power inventory amassban weight 2: existing policy score."),
    power_inventory_maxsp_threshold => (100, "mana", 0, 1000, "borg_power.rs:1398", "Power inventory maxsp threshold: existing policy mana."),
    power_inventory_potion_restore_mana_limit_2 => (10, "count", 0, 255, "borg_power.rs:1400", "Power inventory potion restore mana limit 2: existing policy count."),
    power_inventory_potion_restore_mana_weight_2 => (4000, "score", 0, 8000, "borg_power.rs:1400", "Power inventory potion restore mana weight 2: existing policy score."),
    power_inventory_astfmagi_limit => (100, "count", 0, 255, "borg_power.rs:1402", "Power inventory astfmagi limit: existing policy count."),
    power_inventory_astfmagi_weight => (4000, "score", 0, 8000, "borg_power.rs:1402", "Power inventory astfmagi weight: existing policy score."),
    power_inventory_clevel_threshold_9 => (35, "level", 0, 51, "borg_power.rs:1406", "Power inventory clevel threshold 9: existing policy level."),
    power_inventory_clevel_threshold_10 => (10, "level", 0, 51, "borg_power.rs:1406", "Power inventory clevel threshold 10: existing policy level."),
    power_inventory_accw_limit => (10, "count", 0, 255, "borg_power.rs:1408", "Power inventory accw limit: existing policy count."),
    power_inventory_accw_weight => (5000, "score", 0, 10000, "borg_power.rs:1408", "Power inventory accw weight: existing policy score."),
    power_inventory_clevel_threshold_11 => (35, "level", 0, 51, "borg_power.rs:1409", "Power inventory clevel threshold 11: existing policy level."),
    power_inventory_accw_limit_2 => (10, "count", 0, 255, "borg_power.rs:1412", "Power inventory accw limit 2: existing policy count."),
    power_inventory_accw_weight_2 => (5000, "score", 0, 10000, "borg_power.rs:1412", "Power inventory accw weight 2: existing policy score."),
    power_inventory_str_threshold_6 => (15, "stat", 0, 238, "borg_power.rs:1413", "Power inventory str threshold 6: existing policy stat."),
    power_inventory_accw_limit_3 => (15, "count", 0, 255, "borg_power.rs:1414", "Power inventory accw limit 3: existing policy count."),
    power_inventory_accw_weight_3 => (500, "score", 0, 1000, "borg_power.rs:1414", "Power inventory accw weight 3: existing policy score."),
    power_inventory_accw_threshold => (5, "count", 0, 255, "borg_power.rs:1419", "Power inventory accw threshold: existing policy count."),
    power_inventory_maxclevel_threshold => (10, "level", 0, 51, "borg_power.rs:1419", "Power inventory maxclevel threshold: existing policy level."),
    power_inventory_clevel_threshold_12 => (35, "level", 0, 51, "borg_power.rs:1419", "Power inventory clevel threshold 12: existing policy level."),
    power_inventory_acsw_limit_2 => (7, "count", 0, 255, "borg_power.rs:1421", "Power inventory acsw limit 2: existing policy count."),
    power_inventory_acsw_weight_2 => (50, "score", 0, 100, "borg_power.rs:1421", "Power inventory acsw weight 2: existing policy score."),
    power_inventory_str_threshold_7 => (15, "stat", 0, 238, "borg_power.rs:1422", "Power inventory str threshold 7: existing policy stat."),
    power_inventory_acsw_limit_3 => (10, "count", 0, 255, "borg_power.rs:1423", "Power inventory acsw limit 3: existing policy count."),
    power_inventory_acsw_weight_3 => (5, "score", 0, 100, "borg_power.rs:1423", "Power inventory acsw weight 3: existing policy score."),
    power_inventory_acsw_threshold => (5, "count", 0, 255, "borg_power.rs:1428", "Power inventory acsw threshold: existing policy count."),
    power_inventory_clevel_threshold_13 => (8, "level", 0, 51, "borg_power.rs:1428", "Power inventory clevel threshold 13: existing policy level."),
    power_inventory_aclw_limit => (5, "count", 0, 255, "borg_power.rs:1430", "Power inventory aclw limit: existing policy count."),
    power_inventory_aclw_weight => (550, "score", 0, 1100, "borg_power.rs:1430", "Power inventory aclw weight: existing policy score."),
    power_inventory_food_cure_conf_limit => (10, "count", 0, 255, "borg_power.rs:1439", "Power inventory food cure conf limit: existing policy count."),
    power_inventory_food_cure_conf_weight => (400, "score", 0, 800, "borg_power.rs:1439", "Power inventory food cure conf weight: existing policy score."),
    power_inventory_food_cure_blind_limit => (5, "count", 0, 255, "borg_power.rs:1445", "Power inventory food cure blind limit: existing policy count."),
    power_inventory_food_cure_blind_weight => (300, "score", 0, 600, "borg_power.rs:1445", "Power inventory food cure blind weight: existing policy score."),
    power_inventory_acurepois_limit => (5, "count", 0, 255, "borg_power.rs:1451", "Power inventory acurepois limit: existing policy count."),
    power_inventory_acurepois_weight => (250, "score", 0, 500, "borg_power.rs:1451", "Power inventory acurepois weight: existing policy score."),
    power_inventory_adettrap_limit => (1, "count", 0, 255, "borg_power.rs:1459", "Power inventory adettrap limit: existing policy count."),
    power_inventory_adettrap_weight => (4000, "score", 0, 8000, "borg_power.rs:1459", "Power inventory adettrap weight: existing policy score."),
    power_inventory_adetdoor_limit => (1, "count", 0, 255, "borg_power.rs:1463", "Power inventory adetdoor limit: existing policy count."),
    power_inventory_adetdoor_weight => (2000, "score", 0, 4000, "borg_power.rs:1463", "Power inventory adetdoor weight: existing policy score."),
    power_inventory_adetevil_limit => (1, "count", 0, 255, "borg_power.rs:1468", "Power inventory adetevil limit: existing policy count."),
    power_inventory_adetevil_weight => (1000, "score", 0, 2000, "borg_power.rs:1468", "Power inventory adetevil weight: existing policy score."),
    power_inventory_amagicmap_limit => (1, "count", 0, 255, "borg_power.rs:1473", "Power inventory amagicmap limit: existing policy count."),
    power_inventory_amagicmap_weight => (4000, "score", 0, 8000, "borg_power.rs:1473", "Power inventory amagicmap weight: existing policy score."),
    power_inventory_alite_limit => (1, "count", 0, 255, "borg_power.rs:1478", "Power inventory alite limit: existing policy count."),
    power_inventory_alite_weight => (1000, "score", 0, 2000, "borg_power.rs:1478", "Power inventory alite weight: existing policy score."),
    power_inventory_maxdepth_threshold_8 => (100, "depth", 0, 127, "borg_power.rs:1482", "Power inventory maxdepth threshold 8: existing policy depth."),
    power_inventory_scroll_mass_banishment_limit => (10, "count", 0, 255, "borg_power.rs:1484", "Power inventory scroll mass banishment limit: existing policy count."),
    power_inventory_scroll_mass_banishment_weight => (10000, "score", 0, 20000, "borg_power.rs:1484", "Power inventory scroll mass banishment weight: existing policy score."),
    power_inventory_scroll_mass_banishment_limit_2 => (25, "count", 0, 255, "borg_power.rs:1485", "Power inventory scroll mass banishment limit 2: existing policy count."),
    power_inventory_scroll_mass_banishment_weight_2 => (2000, "score", 0, 4000, "borg_power.rs:1485", "Power inventory scroll mass banishment weight 2: existing policy score."),
    power_inventory_aspeed_limit_2 => (20, "count", 0, 255, "borg_power.rs:1491", "Power inventory aspeed limit 2: existing policy count."),
    power_inventory_aspeed_weight_2 => (5000, "score", 0, 10000, "borg_power.rs:1491", "Power inventory aspeed weight 2: existing policy score."),
    power_inventory_maxdepth_threshold_9 => (99, "depth", 0, 127, "borg_power.rs:1495", "Power inventory maxdepth threshold 9: existing policy depth."),
    power_inventory_amissiles_limit => (40, "count", 0, 255, "borg_power.rs:1504", "Power inventory amissiles limit: existing policy count."),
    power_inventory_amissiles_weight => (100, "score", 0, 200, "borg_power.rs:1504", "Power inventory amissiles weight: existing policy score."),
    power_inventory_str_threshold_8 => (15, "stat", 0, 238, "borg_power.rs:1505", "Power inventory str threshold 8: existing policy stat."),
    power_inventory_str_threshold_9 => (18, "stat", 0, 238, "borg_power.rs:1505", "Power inventory str threshold 9: existing policy stat."),
    power_inventory_amissiles_limit_2 => (80, "count", 0, 255, "borg_power.rs:1506", "Power inventory amissiles limit 2: existing policy count."),
    power_inventory_amissiles_weight_2 => (10, "score", 0, 100, "borg_power.rs:1506", "Power inventory amissiles weight 2: existing policy score."),
    power_inventory_str_threshold_10 => (18, "stat", 0, 238, "borg_power.rs:1508", "Power inventory str threshold 10: existing policy stat."),
    power_inventory_amissiles_limit_3 => (180, "count", 0, 360, "borg_power.rs:1509", "Power inventory amissiles limit 3: existing policy count."),
    power_inventory_amissiles_weight_3 => (8, "score", 0, 100, "borg_power.rs:1509", "Power inventory amissiles weight 3: existing policy score."),
    power_inventory_penalize_use_of_too_many_quiver_slots_threshold => (4, "count", 0, 255, "borg_power.rs:1513", "Power inventory penalize use of too many quiver slots threshold: existing policy count."),
    power_inventory_amissiles_limit_4 => (20, "count", 0, 255, "borg_power.rs:1520", "Power inventory amissiles limit 4: existing policy count."),
    power_inventory_amissiles_weight_4 => (100, "score", 0, 200, "borg_power.rs:1520", "Power inventory amissiles weight 4: existing policy score."),
    power_inventory_str_threshold_11 => (15, "stat", 0, 238, "borg_power.rs:1521", "Power inventory str threshold 11: existing policy stat."),
    power_inventory_amissiles_limit_5 => (50, "count", 0, 255, "borg_power.rs:1522", "Power inventory amissiles limit 5: existing policy count."),
    power_inventory_amissiles_weight_5 => (10, "score", 0, 100, "borg_power.rs:1522", "Power inventory amissiles weight 5: existing policy score."),
    power_inventory_str_threshold_12 => (15, "stat", 0, 238, "borg_power.rs:1525", "Power inventory str threshold 12: existing policy stat."),
    power_inventory_amissiles_threshold => (20, "count", 0, 255, "borg_power.rs:1525", "Power inventory amissiles threshold: existing policy count."),
    power_inventory_penalize_use_of_too_many_quiver_slots_threshold_2 => (2, "count", 0, 255, "borg_power.rs:1529", "Power inventory penalize use of too many quiver slots threshold 2: existing policy count."),
    power_inventory_astfdest_limit => (9, "count", 0, 255, "borg_power.rs:1549", "Power inventory astfdest limit: existing policy count."),
    power_inventory_astfdest_weight => (200, "score", 0, 400, "borg_power.rs:1549", "Power inventory astfdest weight: existing policy score."),
    power_inventory_atportother_limit => (15, "count", 0, 255, "borg_power.rs:1561", "Power inventory atportother limit: existing policy count."),
    power_inventory_atportother_weight => (5000, "score", 0, 10000, "borg_power.rs:1561", "Power inventory atportother weight: existing policy score."),
    power_inventory_maxdepth_threshold_10 => (30, "depth", 0, 127, "borg_power.rs:1565", "Power inventory maxdepth threshold 10: existing policy depth."),
    power_inventory_cdepth_threshold => (30, "depth", 0, 127, "borg_power.rs:1569", "Power inventory cdepth threshold: existing policy depth."),
    power_inventory_clevel_threshold_14 => (20, "level", 0, 51, "borg_power.rs:1573", "Power inventory clevel threshold 14: existing policy level."),
    power_inventory_good_s_chg_limit => (3, "count", 0, 255, "borg_power.rs:1589", "Power inventory good s chg limit: existing policy count."),
    power_inventory_good_s_chg_weight => (500, "score", 0, 1000, "borg_power.rs:1589", "Power inventory good s chg weight: existing policy score."),
    power_inventory_arod1_limit => (6, "count", 0, 255, "borg_power.rs:1593", "Power inventory arod1 limit: existing policy count."),
    power_inventory_arod1_weight => (8000, "score", 0, 16000, "borg_power.rs:1593", "Power inventory arod1 weight: existing policy score."),
    power_inventory_arod2_limit => (6, "count", 0, 255, "borg_power.rs:1595", "Power inventory arod2 limit: existing policy count."),
    power_inventory_arod2_weight => (12000, "score", 0, 24000, "borg_power.rs:1595", "Power inventory arod2 weight: existing policy score."),
    power_inventory_aench_arm_threshold => (1000, "count", 0, 2000, "borg_power.rs:1662", "Power inventory aench arm threshold: existing policy count."),
    power_inventory_aench_toh_threshold => (1000, "count", 0, 2000, "borg_power.rs:1667", "Power inventory aench toh threshold: existing policy count."),
    power_inventory_aench_tod_threshold => (1000, "count", 0, 2000, "borg_power.rs:1672", "Power inventory aench tod threshold: existing policy count."),
    power_inventory_empty_limit => (6, "count", 0, 255, "borg_power.rs:1688", "Power inventory empty limit: existing policy count."),
    power_inventory_empty_weight => (40, "score", 0, 100, "borg_power.rs:1688", "Power inventory empty weight: existing policy score."),
    power_inventory_maxdepth_threshold_11 => (40, "depth", 0, 127, "borg_power.rs:1696", "Power inventory maxdepth threshold 11: existing policy depth."),
    power_inventory_maxdepth_threshold_12 => (25, "depth", 0, 127, "borg_power.rs:1697", "Power inventory maxdepth threshold 12: existing policy depth."),
    power_inventory_gold_threshold => (100000, "gold", 0, 200000, "borg_power.rs:1698", "Power inventory gold threshold: existing policy gold."),
    power_inventory_ignore_difficult_normal_books_threshold => (5, "level", 0, 51, "borg_power.rs:1759", "Power inventory ignore difficult normal books threshold: existing policy level."),
    power_inventory_maxclevel_threshold_3 => (2, "level", 0, 51, "borg_power.rs:1759", "Power inventory maxclevel threshold 3: existing policy level."),
    power_inventory_reward_the_book_limit => (1, "count", 0, 255, "borg_power.rs:1766", "Power inventory reward the book limit: existing policy count."),
    power_inventory_reward_the_book_weight => (500000, "score", 0, 1000000, "borg_power.rs:1766", "Power inventory reward the book weight: existing policy score."),
    power_inventory_str_threshold_13 => (5, "stat", 0, 238, "borg_power.rs:1767", "Power inventory str threshold 13: existing policy stat."),
    power_inventory_reward_the_book_limit_2 => (2, "count", 0, 255, "borg_power.rs:1768", "Power inventory reward the book limit 2: existing policy count."),
    power_inventory_reward_the_book_weight_2 => (10000, "score", 0, 20000, "borg_power.rs:1768", "Power inventory reward the book weight 2: existing policy score."),
    power_inventory_aench_arm_threshold_2 => (1000, "count", 0, 2000, "borg_power.rs:1800", "Power inventory aench arm threshold 2: existing policy count."),
    power_inventory_aench_toh_threshold_2 => (1000, "count", 0, 2000, "borg_power.rs:1803", "Power inventory aench toh threshold 2: existing policy count."),
    power_inventory_aench_tod_threshold_2 => (1000, "count", 0, 2000, "borg_power.rs:1806", "Power inventory aench tod threshold 2: existing policy count."),
    power_prepared_depth_weight => (40000, "score", 0, 80000, "borg_power.rs:1856", "Power prepared depth weight: existing policy score."),
    power_equipment_unidentified_weapon_max_depth => (10, "depth", 0, 127, "borg_power.rs:320", "Power equipment unidentified weapon max depth: existing policy depth."),
    power_equipment_unidentified_weapon_max_level => (15, "level", 0, 51, "borg_power.rs:320", "Power equipment unidentified weapon max level: existing policy level."),
    power_equipment_unidentified_bow_max_depth => (10, "depth", 0, 127, "borg_power.rs:453", "Power equipment unidentified bow max depth: existing policy depth."),
    power_equipment_unidentified_bow_max_level => (15, "level", 0, 51, "borg_power.rs:453", "Power equipment unidentified bow max level: existing policy level."),
    power_equipment_bow_enchant_floor => (8, "damage", 0, 5000, "borg_power.rs:461", "Power equipment bow enchant floor: existing policy damage."),
    power_equipment_light_penalty => (10000, "score", 0, 100000, "borg_power.rs:511", "Power equipment light penalty: existing policy score."),
    power_equipment_sp_warrior_divisor => (2, "divisor", 1, 1000, "borg_power.rs:595", "Power equipment sp warrior divisor: existing policy divisor."),
    power_equipment_sp_warrior_weight => (255, "score", 0, 100000, "borg_power.rs:595", "Power equipment sp warrior weight: existing policy score."),
    power_equipment_sp_caster_divisor => (2, "divisor", 1, 1000, "borg_power.rs:597", "Power equipment sp caster divisor: existing policy divisor."),
    power_equipment_sp_caster_weight => (155, "score", 0, 100000, "borg_power.rs:597", "Power equipment sp caster weight: existing policy score."),
    power_equipment_bow_enchant_threshold => (8, "damage", 0, 5000, "borg_power.rs:458", "Power equipment bow enchant threshold: existing policy damage."),
    power_equipment_bow_enchant_min_level => (25, "level", 0, 51, "borg_power.rs:458", "Power equipment bow enchant min level: existing policy level."),
    power_equipment_hold_life_depth => (46, "depth", 0, 127, "borg_power.rs:864", "Power equipment hold life depth: existing policy depth."),
    power_prepared_depth_lookahead => (50, "depth", 0, 50, "borg_power.rs:1849", "Power prepared depth lookahead: existing policy depth."),
    power_equipment_spell_reliability_weight => (100, "score", 0, 200, "borg_power.rs:603", "Power equipment spell reliability weight: existing policy score."),

    // #433: M13 inventory policy, pre-extraction source at 68e39b7e.
    buy_attack_wand_charge_limit => (40, "count", 0, 1000, "borg_store_buy.rs:543", "buy attack wand charge limit: inherited policy; neutral default."),
    buy_attack_wand_max_level => (30, "level", 1, 50, "borg_store_buy.rs:552", "buy attack wand max level: inherited policy; neutral default."),
    buy_black_market_early_gold => (20000, "gold", 0, 1000000, "borg_store_buy.rs:310", "buy black market early gold: inherited policy; neutral default."),
    buy_black_market_early_level => (15, "level", 1, 50, "borg_store_buy.rs:310", "buy black market early level: inherited policy; neutral default."),
    buy_black_market_middle_gold => (15000, "gold", 0, 1000000, "borg_store_buy.rs:313", "buy black market middle gold: inherited policy; neutral default."),
    buy_black_market_middle_level => (35, "level", 1, 50, "borg_store_buy.rs:313", "buy black market middle level: inherited policy; neutral default."),
    buy_black_market_min_gold => (10000, "gold", 0, 1000000, "borg_store_buy.rs:316", "buy black market min gold: inherited policy; neutral default."),
    buy_enchanted_ammo_min_level => (35, "level", 1, 50, "borg_store_buy.rs:231", "buy enchanted ammo min level: inherited policy; neutral default."),
    buy_extra_book_min_level => (5, "level", 1, 50, "borg_store_buy.rs:346", "buy extra book min level: inherited policy; neutral default."),
    buy_home_stock_gold_divisor => (10, "divisor", 1, 1000, "borg_store_buy.rs:1056", "buy home stock gold divisor: inherited policy; neutral default."),
    buy_home_stock_gold_reserve => (1000, "gold", 0, 1000000, "borg_store_buy.rs:1013", "buy home stock gold reserve: inherited policy; neutral default."),
    buy_home_stock_min_level => (15, "level", 1, 50, "borg_store_buy.rs:971", "buy home stock min level: inherited policy; neutral default."),
    buy_home_stock_price_multiplier => (5, "multiplier", 1, 100, "borg_store_buy.rs:1013", "buy home stock price multiplier: inherited policy; neutral default."),
    buy_hybrid_extra_book_max_level => (8, "level", 1, 50, "borg_store_buy.rs:355", "buy hybrid extra book max level: inherited policy; neutral default."),
    home_amulets_limit => (3, "count", 0, 100, "borg_home_power.rs:188", "home amulets limit: inherited policy; neutral default."),
    home_amulets_penalty => (1500, "score", 0, 1000000, "borg_home_power.rs:188", "home amulets penalty: inherited policy; neutral default."),
    home_armor_limit => (6, "count", 0, 100, "borg_home_power.rs:189", "home armor limit: inherited policy; neutral default."),
    home_armor_penalty => (1000, "score", 0, 1000000, "borg_home_power.rs:189", "home armor penalty: inherited policy; neutral default."),
    home_artifact_weight => (500, "score", 0, 1000000, "borg_home_power.rs:449", "home artifact weight: inherited policy; neutral default."),
    home_bad_gloves_penalty => (3000, "score", 0, 1000000, "borg_home_power.rs:203", "home bad gloves penalty: inherited policy; neutral default."),
    home_banishment_stack_multiplier => (2, "multiplier", 1, 10, "borg_home_power.rs:305", "home banishment stack multiplier: inherited policy; neutral default."),
    home_book_stock_base => (5000, "score", 0, 1000000, "borg_home_power.rs:441", "home book stock base: inherited policy; neutral default."),
    home_book_stock_decay => (10, "score", 0, 1000000, "borg_home_power.rs:441", "home book stock decay: inherited policy; neutral default."),
    home_book_stock_limit => (5, "count", 0, 100, "borg_home_power.rs:438", "home book stock limit: inherited policy; neutral default."),
    home_book_stock_max_level => (15, "level", 1, 50, "borg_home_power.rs:435", "home book stock max level: inherited policy; neutral default."),
    home_boots_limit => (3, "count", 0, 100, "borg_home_power.rs:194", "home boots limit: inherited policy; neutral default."),
    home_boots_penalty => (1000, "score", 0, 1000000, "borg_home_power.rs:194", "home boots penalty: inherited policy; neutral default."),
    home_bows_limit => (2, "count", 0, 100, "borg_home_power.rs:181", "home bows limit: inherited policy; neutral default."),
    home_bows_penalty => (1000, "score", 0, 1000000, "borg_home_power.rs:181", "home bows penalty: inherited policy; neutral default."),
    home_cloaks_limit => (3, "count", 0, 100, "borg_home_power.rs:190", "home cloaks limit: inherited policy; neutral default."),
    home_cloaks_penalty => (1000, "score", 0, 1000000, "borg_home_power.rs:190", "home cloaks penalty: inherited policy; neutral default."),
    home_cold_resist_base => (100, "score", 0, 1000000, "borg_home_power.rs:341", "home cold resist base: inherited policy; neutral default."),
    home_cold_resist_decay => (10, "score", 0, 1000000, "borg_home_power.rs:341", "home cold resist decay: inherited policy; neutral default."),
    home_cure_serious_penalty_min_level => (35, "level", 1, 50, "borg_home_power.rs:412", "home cure serious penalty min level: inherited policy; neutral default."),
    home_duplicate_item_penalty => (50000, "score", 0, 1000000, "borg_home_power.rs:208", "home duplicate item penalty: inherited policy; neutral default."),
    home_edged_weapon_penalty => (50, "score", 0, 1000000, "borg_home_power.rs:200", "home edged weapon penalty: inherited policy; neutral default."),
    home_ego_weight => (5000, "score", 0, 1000000, "borg_home_power.rs:452", "home ego weight: inherited policy; neutral default."),
    home_enchant_armor_max_level => (45, "level", 1, 50, "borg_home_power.rs:254", "home enchant armor max level: inherited policy; neutral default."),
    home_enchant_damage_max_level => (45, "level", 1, 50, "borg_home_power.rs:274", "home enchant damage max level: inherited policy; neutral default."),
    home_enchant_hit_max_level => (45, "level", 1, 50, "borg_home_power.rs:264", "home enchant hit max level: inherited policy; neutral default."),
    home_exp_stock_early_max_level => (35, "level", 1, 50, "borg_home_power.rs:425", "home exp stock early max level: inherited policy; neutral default."),
    home_exp_stock_late_min_level => (35, "level", 1, 50, "borg_home_power.rs:423", "home exp stock late min level: inherited policy; neutral default."),
    home_ffall_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:121", "home ffall each weight: inherited policy; neutral default."),
    home_ffall_one_weight => (10, "score", 0, 1000000, "borg_home_power.rs:121", "home ffall one weight: inherited policy; neutral default."),
    home_ffall_two_weight => (15, "score", 0, 1000000, "borg_home_power.rs:121", "home ffall two weight: inherited policy; neutral default."),
    home_food_stock_max_level => (10, "level", 1, 50, "borg_home_power.rs:234", "home food stock max level: inherited policy; neutral default."),
    home_free_act_each_weight => (10, "score", 0, 1000000, "borg_home_power.rs:122", "home free act each weight: inherited policy; neutral default."),
    home_free_act_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:122", "home free act one weight: inherited policy; neutral default."),
    home_free_act_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:122", "home free act two weight: inherited policy; neutral default."),
    home_gloves_limit => (3, "count", 0, 100, "borg_home_power.rs:193", "home gloves limit: inherited policy; neutral default."),
    home_gloves_penalty => (1000, "score", 0, 1000000, "borg_home_power.rs:193", "home gloves penalty: inherited policy; neutral default."),
    home_hats_limit => (4, "count", 0, 100, "borg_home_power.rs:192", "home hats limit: inherited policy; neutral default."),
    home_hats_penalty => (1000, "score", 0, 1000000, "borg_home_power.rs:192", "home hats penalty: inherited policy; neutral default."),
    home_heat_resist_base => (100, "score", 0, 1000000, "borg_home_power.rs:337", "home heat resist base: inherited policy; neutral default."),
    home_heat_resist_decay => (10, "score", 0, 1000000, "borg_home_power.rs:337", "home heat resist decay: inherited policy; neutral default."),
    home_hold_life_each_weight => (10, "score", 0, 1000000, "borg_home_power.rs:123", "home hold life each weight: inherited policy; neutral default."),
    home_hold_life_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:123", "home hold life one weight: inherited policy; neutral default."),
    home_hold_life_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:123", "home hold life two weight: inherited policy; neutral default."),
    home_immune_acid_each_weight => (30, "score", 0, 1000000, "borg_home_power.rs:126", "home immune acid each weight: inherited policy; neutral default."),
    home_immune_acid_one_weight => (3000, "score", 0, 1000000, "borg_home_power.rs:126", "home immune acid one weight: inherited policy; neutral default."),
    home_immune_acid_two_weight => (5000, "score", 0, 1000000, "borg_home_power.rs:126", "home immune acid two weight: inherited policy; neutral default."),
    home_immune_cold_each_weight => (30, "score", 0, 1000000, "borg_home_power.rs:132", "home immune cold each weight: inherited policy; neutral default."),
    home_immune_cold_one_weight => (3000, "score", 0, 1000000, "borg_home_power.rs:132", "home immune cold one weight: inherited policy; neutral default."),
    home_immune_cold_two_weight => (5000, "score", 0, 1000000, "borg_home_power.rs:132", "home immune cold two weight: inherited policy; neutral default."),
    home_immune_elec_each_weight => (30, "score", 0, 1000000, "borg_home_power.rs:128", "home immune elec each weight: inherited policy; neutral default."),
    home_immune_elec_one_weight => (3000, "score", 0, 1000000, "borg_home_power.rs:128", "home immune elec one weight: inherited policy; neutral default."),
    home_immune_elec_two_weight => (5000, "score", 0, 1000000, "borg_home_power.rs:128", "home immune elec two weight: inherited policy; neutral default."),
    home_immune_fire_each_weight => (30, "score", 0, 1000000, "borg_home_power.rs:130", "home immune fire each weight: inherited policy; neutral default."),
    home_immune_fire_one_weight => (3000, "score", 0, 1000000, "borg_home_power.rs:130", "home immune fire one weight: inherited policy; neutral default."),
    home_immune_fire_two_weight => (5000, "score", 0, 1000000, "borg_home_power.rs:130", "home immune fire two weight: inherited policy; neutral default."),
    home_initial_heal_sell_level => (1, "level", 1, 50, "borg_home_power.rs:390", "home initial heal sell level: inherited policy; neutral default."),
    home_light_each_weight => (5, "score", 0, 1000000, "borg_home_power.rs:116", "home light each weight: inherited policy; neutral default."),
    home_light_one_weight => (150, "score", 0, 1000000, "borg_home_power.rs:116", "home light one weight: inherited policy; neutral default."),
    home_light_two_weight => (170, "score", 0, 1000000, "borg_home_power.rs:116", "home light two weight: inherited policy; neutral default."),
    home_mana_stock_min_sp => (1, "sp", 0, 10000, "borg_home_power.rs:377", "home mana stock min sp: inherited policy; neutral default."),
    home_mass_banishment_stack_multiplier => (2, "multiplier", 1, 10, "borg_home_power.rs:314", "home mass banishment stack multiplier: inherited policy; neutral default."),
    home_max_level_exp_penalty => (7500, "score", 0, 1000000, "borg_home_power.rs:421", "home max level exp penalty: inherited policy; neutral default."),
    home_phase_stock_max_level => (10, "level", 1, 50, "borg_home_power.rs:363", "home phase stock max level: inherited policy; neutral default."),
    home_regenerate_each_weight => (10, "score", 0, 1000000, "borg_home_power.rs:118", "home regenerate each weight: inherited policy; neutral default."),
    home_regenerate_one_weight => (75, "score", 0, 1000000, "borg_home_power.rs:118", "home regenerate one weight: inherited policy; neutral default."),
    home_regenerate_two_weight => (100, "score", 0, 1000000, "borg_home_power.rs:118", "home regenerate two weight: inherited policy; neutral default."),
    home_resist_acid_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:125", "home resist acid each weight: inherited policy; neutral default."),
    home_resist_acid_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:125", "home resist acid one weight: inherited policy; neutral default."),
    home_resist_acid_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:125", "home resist acid two weight: inherited policy; neutral default."),
    home_resist_blind_each_weight => (5, "score", 0, 1000000, "borg_home_power.rs:143", "home resist blind each weight: inherited policy; neutral default."),
    home_resist_blind_one_weight => (500, "score", 0, 1000000, "borg_home_power.rs:143", "home resist blind one weight: inherited policy; neutral default."),
    home_resist_blind_two_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:143", "home resist blind two weight: inherited policy; neutral default."),
    home_resist_chaos_each_weight => (10, "score", 0, 1000000, "borg_home_power.rs:139", "home resist chaos each weight: inherited policy; neutral default."),
    home_resist_chaos_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:139", "home resist chaos one weight: inherited policy; neutral default."),
    home_resist_chaos_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:139", "home resist chaos two weight: inherited policy; neutral default."),
    home_resist_cold_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:131", "home resist cold each weight: inherited policy; neutral default."),
    home_resist_cold_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:131", "home resist cold one weight: inherited policy; neutral default."),
    home_resist_cold_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:131", "home resist cold two weight: inherited policy; neutral default."),
    home_resist_conf_each_weight => (45, "score", 0, 1000000, "borg_home_power.rs:135", "home resist conf each weight: inherited policy; neutral default."),
    home_resist_conf_one_weight => (2000, "score", 0, 1000000, "borg_home_power.rs:135", "home resist conf one weight: inherited policy; neutral default."),
    home_resist_conf_two_weight => (8000, "score", 0, 1000000, "borg_home_power.rs:135", "home resist conf two weight: inherited policy; neutral default."),
    home_resist_dark_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:138", "home resist dark each weight: inherited policy; neutral default."),
    home_resist_dark_one_weight => (100, "score", 0, 1000000, "borg_home_power.rs:138", "home resist dark one weight: inherited policy; neutral default."),
    home_resist_dark_two_weight => (150, "score", 0, 1000000, "borg_home_power.rs:138", "home resist dark two weight: inherited policy; neutral default."),
    home_resist_disen_each_weight => (35, "score", 0, 1000000, "borg_home_power.rs:140", "home resist disen each weight: inherited policy; neutral default."),
    home_resist_disen_one_weight => (5000, "score", 0, 1000000, "borg_home_power.rs:140", "home resist disen one weight: inherited policy; neutral default."),
    home_resist_disen_two_weight => (7000, "score", 0, 1000000, "borg_home_power.rs:140", "home resist disen two weight: inherited policy; neutral default."),
    home_resist_elec_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:127", "home resist elec each weight: inherited policy; neutral default."),
    home_resist_elec_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:127", "home resist elec one weight: inherited policy; neutral default."),
    home_resist_elec_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:127", "home resist elec two weight: inherited policy; neutral default."),
    home_resist_fire_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:129", "home resist fire each weight: inherited policy; neutral default."),
    home_resist_fire_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:129", "home resist fire one weight: inherited policy; neutral default."),
    home_resist_fire_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:129", "home resist fire two weight: inherited policy; neutral default."),
    home_resist_light_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:137", "home resist light each weight: inherited policy; neutral default."),
    home_resist_light_one_weight => (100, "score", 0, 1000000, "borg_home_power.rs:137", "home resist light one weight: inherited policy; neutral default."),
    home_resist_light_two_weight => (150, "score", 0, 1000000, "borg_home_power.rs:137", "home resist light two weight: inherited policy; neutral default."),
    home_resist_neth_each_weight => (45, "score", 0, 1000000, "borg_home_power.rs:144", "home resist neth each weight: inherited policy; neutral default."),
    home_resist_neth_one_weight => (3000, "score", 0, 1000000, "borg_home_power.rs:144", "home resist neth one weight: inherited policy; neutral default."),
    home_resist_neth_two_weight => (4000, "score", 0, 1000000, "borg_home_power.rs:144", "home resist neth two weight: inherited policy; neutral default."),
    home_resist_nexus_each_weight => (2, "score", 0, 1000000, "borg_home_power.rs:142", "home resist nexus each weight: inherited policy; neutral default."),
    home_resist_nexus_one_weight => (200, "score", 0, 1000000, "borg_home_power.rs:142", "home resist nexus one weight: inherited policy; neutral default."),
    home_resist_nexus_two_weight => (300, "score", 0, 1000000, "borg_home_power.rs:142", "home resist nexus two weight: inherited policy; neutral default."),
    home_resist_pois_each_weight => (40, "score", 0, 1000000, "borg_home_power.rs:134", "home resist pois each weight: inherited policy; neutral default."),
    home_resist_pois_one_weight => (5000, "score", 0, 1000000, "borg_home_power.rs:134", "home resist pois one weight: inherited policy; neutral default."),
    home_resist_pois_two_weight => (9000, "score", 0, 1000000, "borg_home_power.rs:134", "home resist pois two weight: inherited policy; neutral default."),
    home_resist_shard_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:141", "home resist shard each weight: inherited policy; neutral default."),
    home_resist_shard_one_weight => (100, "score", 0, 1000000, "borg_home_power.rs:141", "home resist shard one weight: inherited policy; neutral default."),
    home_resist_shard_two_weight => (150, "score", 0, 1000000, "borg_home_power.rs:141", "home resist shard two weight: inherited policy; neutral default."),
    home_resist_sound_each_weight => (30, "score", 0, 1000000, "borg_home_power.rs:136", "home resist sound each weight: inherited policy; neutral default."),
    home_resist_sound_one_weight => (500, "score", 0, 1000000, "borg_home_power.rs:136", "home resist sound one weight: inherited policy; neutral default."),
    home_resist_sound_two_weight => (700, "score", 0, 1000000, "borg_home_power.rs:136", "home resist sound two weight: inherited policy; neutral default."),
    home_resistance_max_depth => (80, "depth", 0, 127, "borg_home_power.rs:332", "home resistance max depth: inherited policy; neutral default."),
    home_resistance_min_depth => (20, "depth", 0, 127, "borg_home_power.rs:331", "home resistance min depth: inherited policy; neutral default."),
    home_rings_excess_weight => (4000, "score", 0, 1000000, "borg_home_power.rs:184", "home rings excess weight: inherited policy; neutral default."),
    home_rings_soft_limit => (6, "count", 0, 100, "borg_home_power.rs:183", "home rings soft limit: inherited policy; neutral default."),
    home_rings_spare_limit => (4, "count", 0, 100, "borg_home_power.rs:186", "home rings spare limit: inherited policy; neutral default."),
    home_rings_spare_penalty => (2000, "score", 0, 1000000, "borg_home_power.rs:186", "home rings spare penalty: inherited policy; neutral default."),
    home_see_inv_each_weight => (10, "score", 0, 1000000, "borg_home_power.rs:120", "home see inv each weight: inherited policy; neutral default."),
    home_see_inv_one_weight => (800, "score", 0, 1000000, "borg_home_power.rs:120", "home see inv one weight: inherited policy; neutral default."),
    home_see_inv_two_weight => (1200, "score", 0, 1000000, "borg_home_power.rs:120", "home see inv two weight: inherited policy; neutral default."),
    home_shields_limit => (3, "count", 0, 100, "borg_home_power.rs:191", "home shields limit: inherited policy; neutral default."),
    home_shields_penalty => (1000, "score", 0, 1000000, "borg_home_power.rs:191", "home shields penalty: inherited policy; neutral default."),
    home_slow_digest_each_weight => (5, "score", 0, 1000000, "borg_home_power.rs:117", "home slow digest each weight: inherited policy; neutral default."),
    home_slow_digest_one_weight => (50, "score", 0, 1000000, "borg_home_power.rs:117", "home slow digest one weight: inherited policy; neutral default."),
    home_slow_digest_two_weight => (70, "score", 0, 1000000, "borg_home_power.rs:117", "home slow digest two weight: inherited policy; neutral default."),
    home_stat_caster_first_weight => (400, "score", 0, 1000000, "borg_home_power.rs:162", "home stat caster first weight: inherited policy; neutral default."),
    home_stat_caster_high_limit => (26, "count", 0, 100, "borg_home_power.rs:162", "home stat caster high limit: inherited policy; neutral default."),
    home_stat_caster_last_base => (3800, "score", 0, 1000000, "borg_home_power.rs:162", "home stat caster last base: inherited policy; neutral default."),
    home_stat_caster_last_weight => (5, "score", 0, 1000000, "borg_home_power.rs:162", "home stat caster last weight: inherited policy; neutral default."),
    home_stat_caster_low_limit => (20, "count", 0, 100, "borg_home_power.rs:162", "home stat caster low limit: inherited policy; neutral default."),
    home_stat_caster_middle_base => (8000, "score", 0, 1000000, "borg_home_power.rs:162", "home stat caster middle base: inherited policy; neutral default."),
    home_stat_caster_middle_weight => (300, "score", 0, 1000000, "borg_home_power.rs:162", "home stat caster middle weight: inherited policy; neutral default."),
    home_stat_con_first_weight => (300, "score", 0, 1000000, "borg_home_power.rs:155", "home stat con first weight: inherited policy; neutral default."),
    home_stat_con_high_limit => (21, "count", 0, 100, "borg_home_power.rs:155", "home stat con high limit: inherited policy; neutral default."),
    home_stat_con_last_base => (5700, "score", 0, 1000000, "borg_home_power.rs:155", "home stat con last base: inherited policy; neutral default."),
    home_stat_con_last_weight => (1, "score", 0, 1000000, "borg_home_power.rs:155", "home stat con last weight: inherited policy; neutral default."),
    home_stat_con_low_limit => (15, "count", 0, 100, "borg_home_power.rs:155", "home stat con low limit: inherited policy; neutral default."),
    home_stat_con_middle_base => (4500, "score", 0, 1000000, "borg_home_power.rs:155", "home stat con middle base: inherited policy; neutral default."),
    home_stat_con_middle_weight => (200, "score", 0, 1000000, "borg_home_power.rs:155", "home stat con middle weight: inherited policy; neutral default."),
    home_stat_dex_first_weight => (300, "score", 0, 1000000, "borg_home_power.rs:151", "home stat dex first weight: inherited policy; neutral default."),
    home_stat_dex_high_limit => (15, "count", 0, 100, "borg_home_power.rs:151", "home stat dex high limit: inherited policy; neutral default."),
    home_stat_dex_last_base => (3900, "score", 0, 1000000, "borg_home_power.rs:151", "home stat dex last base: inherited policy; neutral default."),
    home_stat_dex_last_weight => (1, "score", 0, 1000000, "borg_home_power.rs:151", "home stat dex last weight: inherited policy; neutral default."),
    home_stat_dex_low_limit => (9, "count", 0, 100, "borg_home_power.rs:151", "home stat dex low limit: inherited policy; neutral default."),
    home_stat_dex_middle_base => (2700, "score", 0, 1000000, "borg_home_power.rs:151", "home stat dex middle base: inherited policy; neutral default."),
    home_stat_dex_middle_weight => (200, "score", 0, 1000000, "borg_home_power.rs:151", "home stat dex middle weight: inherited policy; neutral default."),
    home_stat_str_first_weight => (300, "score", 0, 1000000, "borg_home_power.rs:149", "home stat str first weight: inherited policy; neutral default."),
    home_stat_str_high_limit => (15, "count", 0, 100, "borg_home_power.rs:149", "home stat str high limit: inherited policy; neutral default."),
    home_stat_str_last_base => (3900, "score", 0, 1000000, "borg_home_power.rs:149", "home stat str last base: inherited policy; neutral default."),
    home_stat_str_last_weight => (1, "score", 0, 1000000, "borg_home_power.rs:149", "home stat str last weight: inherited policy; neutral default."),
    home_stat_str_low_limit => (9, "count", 0, 100, "borg_home_power.rs:149", "home stat str low limit: inherited policy; neutral default."),
    home_stat_str_middle_base => (2700, "score", 0, 1000000, "borg_home_power.rs:149", "home stat str middle base: inherited policy; neutral default."),
    home_stat_str_middle_weight => (200, "score", 0, 1000000, "borg_home_power.rs:149", "home stat str middle weight: inherited policy; neutral default."),
    home_stock_cure_critical_base => (1500, "score", 0, 1000000, "borg_home_power.rs:397", "home stock cure critical base: inherited policy; neutral default."),
    home_stock_cure_critical_decay => (10, "score", 0, 1000000, "borg_home_power.rs:397", "home stock cure critical decay: inherited policy; neutral default."),
    home_stock_cure_serious_base => (-1500, "score", -1000000, 0, "borg_home_power.rs:414", "home stock cure serious base: inherited policy; neutral default."),
    home_stock_cure_serious_decay => (-10, "score", -1000000, 0, "borg_home_power.rs:414", "home stock cure serious decay: inherited policy; neutral default."),
    home_stock_cure_serious_limit => (90, "count", 0, 512, "borg_home_power.rs:414", "home stock cure serious limit: inherited policy; neutral default."),
    home_stock_early_exp_base => (1000, "score", 0, 1000000, "borg_home_power.rs:426", "home stock early exp base: inherited policy; neutral default."),
    home_stock_early_exp_decay => (10, "score", 0, 1000000, "borg_home_power.rs:426", "home stock early exp decay: inherited policy; neutral default."),
    home_stock_early_exp_limit => (5, "count", 0, 512, "borg_home_power.rs:426", "home stock early exp limit: inherited policy; neutral default."),
    home_stock_enchant_to_a_base => (500, "score", 0, 1000000, "borg_home_power.rs:255", "home stock enchant to a base: inherited policy; neutral default."),
    home_stock_enchant_to_a_decay => (10, "score", 0, 1000000, "borg_home_power.rs:255", "home stock enchant to a decay: inherited policy; neutral default."),
    home_stock_enchant_to_d_base => (500, "score", 0, 1000000, "borg_home_power.rs:275", "home stock enchant to d base: inherited policy; neutral default."),
    home_stock_enchant_to_d_decay => (10, "score", 0, 1000000, "borg_home_power.rs:275", "home stock enchant to d decay: inherited policy; neutral default."),
    home_stock_enchant_to_h_base => (500, "score", 0, 1000000, "borg_home_power.rs:265", "home stock enchant to h base: inherited policy; neutral default."),
    home_stock_enchant_to_h_decay => (10, "score", 0, 1000000, "borg_home_power.rs:265", "home stock enchant to h decay: inherited policy; neutral default."),
    home_stock_escape_base => (2000, "score", 0, 1000000, "borg_home_power.rs:350", "home stock escape base: inherited policy; neutral default."),
    home_stock_escape_decay => (10, "score", 0, 1000000, "borg_home_power.rs:350", "home stock escape decay: inherited policy; neutral default."),
    home_stock_escape_limit => (85, "count", 0, 512, "borg_home_power.rs:350", "home stock escape limit: inherited policy; neutral default."),
    home_stock_ezheal_base => (8000, "score", 0, 1000000, "borg_home_power.rs:407", "home stock ezheal base: inherited policy; neutral default."),
    home_stock_ezheal_decay => (0, "score", 0, 1000000, "borg_home_power.rs:407", "home stock ezheal decay: inherited policy; neutral default."),
    home_stock_ezheal_limit => (198, "count", 0, 512, "borg_home_power.rs:407", "home stock ezheal limit: inherited policy; neutral default."),
    home_stock_food_base => (8000, "score", 0, 1000000, "borg_home_power.rs:235", "home stock food base: inherited policy; neutral default."),
    home_stock_food_decay => (10, "score", 0, 1000000, "borg_home_power.rs:235", "home stock food decay: inherited policy; neutral default."),
    home_stock_genocide_base => (500, "score", 0, 1000000, "borg_home_power.rs:303", "home stock genocide base: inherited policy; neutral default."),
    home_stock_genocide_decay => (10, "score", 0, 1000000, "borg_home_power.rs:303", "home stock genocide decay: inherited policy; neutral default."),
    home_stock_glyph_base => (500, "score", 0, 1000000, "borg_home_power.rs:294", "home stock glyph base: inherited policy; neutral default."),
    home_stock_glyph_decay => (10, "score", 0, 1000000, "borg_home_power.rs:294", "home stock glyph decay: inherited policy; neutral default."),
    home_stock_heal_base => (3000, "score", 0, 1000000, "borg_home_power.rs:406", "home stock heal base: inherited policy; neutral default."),
    home_stock_heal_decay => (0, "score", 0, 1000000, "borg_home_power.rs:406", "home stock heal decay: inherited policy; neutral default."),
    home_stock_heal_limit => (90, "count", 0, 512, "borg_home_power.rs:406", "home stock heal limit: inherited policy; neutral default."),
    home_stock_ident_base => (2000, "score", 0, 1000000, "borg_home_power.rs:245", "home stock ident base: inherited policy; neutral default."),
    home_stock_ident_decay => (10, "score", 0, 1000000, "borg_home_power.rs:245", "home stock ident decay: inherited policy; neutral default."),
    home_stock_initial_heal_base => (-5000, "score", -1000000, 0, "borg_home_power.rs:391", "home stock initial heal base: inherited policy; neutral default."),
    home_stock_initial_heal_decay => (0, "score", 0, 1000000, "borg_home_power.rs:391", "home stock initial heal decay: inherited policy; neutral default."),
    home_stock_initial_heal_limit => (10, "count", 0, 512, "borg_home_power.rs:391", "home stock initial heal limit: inherited policy; neutral default."),
    home_stock_late_exp_base => (1000, "score", 0, 1000000, "borg_home_power.rs:424", "home stock late exp base: inherited policy; neutral default."),
    home_stock_late_exp_decay => (10, "score", 0, 1000000, "borg_home_power.rs:424", "home stock late exp decay: inherited policy; neutral default."),
    home_stock_late_exp_limit => (70, "count", 0, 512, "borg_home_power.rs:424", "home stock late exp limit: inherited policy; neutral default."),
    home_stock_life_base => (9000, "score", 0, 1000000, "borg_home_power.rs:408", "home stock life base: inherited policy; neutral default."),
    home_stock_life_decay => (0, "score", 0, 1000000, "borg_home_power.rs:408", "home stock life decay: inherited policy; neutral default."),
    home_stock_life_limit => (198, "count", 0, 512, "borg_home_power.rs:408", "home stock life limit: inherited policy; neutral default."),
    home_stock_mana_base => (6000, "score", 0, 1000000, "borg_home_power.rs:378", "home stock mana base: inherited policy; neutral default."),
    home_stock_mana_decay => (8, "score", 0, 1000000, "borg_home_power.rs:378", "home stock mana decay: inherited policy; neutral default."),
    home_stock_mass_genocide_base => (500, "score", 0, 1000000, "borg_home_power.rs:312", "home stock mass genocide base: inherited policy; neutral default."),
    home_stock_mass_genocide_decay => (0, "score", 0, 1000000, "borg_home_power.rs:312", "home stock mass genocide decay: inherited policy; neutral default."),
    home_stock_pfe_base => (500, "score", 0, 1000000, "borg_home_power.rs:285", "home stock pfe base: inherited policy; neutral default."),
    home_stock_pfe_decay => (10, "score", 0, 1000000, "borg_home_power.rs:285", "home stock pfe decay: inherited policy; neutral default."),
    home_stock_phase_base => (5000, "score", 0, 1000000, "borg_home_power.rs:364", "home stock phase base: inherited policy; neutral default."),
    home_stock_phase_decay => (0, "score", 0, 1000000, "borg_home_power.rs:364", "home stock phase decay: inherited policy; neutral default."),
    home_stock_recall_base => (100, "score", 0, 1000000, "borg_home_power.rs:347", "home stock recall base: inherited policy; neutral default."),
    home_stock_recall_decay => (0, "score", 0, 1000000, "borg_home_power.rs:347", "home stock recall decay: inherited policy; neutral default."),
    home_stock_recall_limit => (5, "count", 0, 512, "borg_home_power.rs:347", "home stock recall limit: inherited policy; neutral default."),
    home_stock_recharge_base => (500, "score", 0, 1000000, "borg_home_power.rs:321", "home stock recharge base: inherited policy; neutral default."),
    home_stock_recharge_decay => (10, "score", 0, 1000000, "borg_home_power.rs:321", "home stock recharge decay: inherited policy; neutral default."),
    home_stock_teleport_base => (5000, "score", 0, 1000000, "borg_home_power.rs:360", "home stock teleport base: inherited policy; neutral default."),
    home_stock_teleport_decay => (0, "score", 0, 1000000, "borg_home_power.rs:360", "home stock teleport decay: inherited policy; neutral default."),
    home_stock_teleport_limit => (85, "count", 0, 512, "borg_home_power.rs:360", "home stock teleport limit: inherited policy; neutral default."),
    home_sustain_all_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:171", "home sustain all each weight: inherited policy; neutral default."),
    home_sustain_all_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:171", "home sustain all one weight: inherited policy; neutral default."),
    home_sustain_all_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:171", "home sustain all two weight: inherited policy; neutral default."),
    home_sustain_con_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:169", "home sustain con each weight: inherited policy; neutral default."),
    home_sustain_con_one_weight => (200, "score", 0, 1000000, "borg_home_power.rs:169", "home sustain con one weight: inherited policy; neutral default."),
    home_sustain_con_two_weight => (250, "score", 0, 1000000, "borg_home_power.rs:169", "home sustain con two weight: inherited policy; neutral default."),
    home_sustain_dex_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:170", "home sustain dex each weight: inherited policy; neutral default."),
    home_sustain_dex_one_weight => (200, "score", 0, 1000000, "borg_home_power.rs:170", "home sustain dex one weight: inherited policy; neutral default."),
    home_sustain_dex_two_weight => (250, "score", 0, 1000000, "borg_home_power.rs:170", "home sustain dex two weight: inherited policy; neutral default."),
    home_sustain_int_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:167", "home sustain int each weight: inherited policy; neutral default."),
    home_sustain_int_one_weight => (200, "score", 0, 1000000, "borg_home_power.rs:167", "home sustain int one weight: inherited policy; neutral default."),
    home_sustain_int_two_weight => (250, "score", 0, 1000000, "borg_home_power.rs:167", "home sustain int two weight: inherited policy; neutral default."),
    home_sustain_str_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:166", "home sustain str each weight: inherited policy; neutral default."),
    home_sustain_str_one_weight => (200, "score", 0, 1000000, "borg_home_power.rs:166", "home sustain str one weight: inherited policy; neutral default."),
    home_sustain_str_two_weight => (250, "score", 0, 1000000, "borg_home_power.rs:166", "home sustain str two weight: inherited policy; neutral default."),
    home_sustain_wis_each_weight => (1, "score", 0, 1000000, "borg_home_power.rs:168", "home sustain wis each weight: inherited policy; neutral default."),
    home_sustain_wis_one_weight => (200, "score", 0, 1000000, "borg_home_power.rs:168", "home sustain wis one weight: inherited policy; neutral default."),
    home_sustain_wis_two_weight => (250, "score", 0, 1000000, "borg_home_power.rs:168", "home sustain wis two weight: inherited policy; neutral default."),
    home_telepathy_each_weight => (10, "score", 0, 1000000, "borg_home_power.rs:119", "home telepathy each weight: inherited policy; neutral default."),
    home_telepathy_one_weight => (1000, "score", 0, 1000000, "borg_home_power.rs:119", "home telepathy one weight: inherited policy; neutral default."),
    home_telepathy_two_weight => (1500, "score", 0, 1000000, "borg_home_power.rs:119", "home telepathy two weight: inherited policy; neutral default."),
    home_teleport_staff_excess_penalty => (50000, "score", 0, 1000000, "borg_home_power.rs:355", "home teleport staff excess penalty: inherited policy; neutral default."),
    home_unidentified_weight => (1005, "score", 0, 1000000, "borg_home_power.rs:457", "home unidentified weight: inherited policy; neutral default."),
    home_weapons_excess_weight => (2000, "score", 0, 1000000, "borg_home_power.rs:177", "home weapons excess weight: inherited policy; neutral default."),
    home_weapons_soft_limit => (5, "count", 0, 100, "borg_home_power.rs:176", "home weapons soft limit: inherited policy; neutral default."),
    home_weapons_spare_limit => (1, "count", 0, 100, "borg_home_power.rs:179", "home weapons spare limit: inherited policy; neutral default."),
    home_weapons_spare_penalty => (100, "score", 0, 1000000, "borg_home_power.rs:179", "home weapons spare penalty: inherited policy; neutral default."),
    power_inventory_annihilation_wand_bonus => (5000, "score", 0, 1000000, "borg_power.rs:1564", "power inventory annihilation wand bonus: inherited policy; neutral default."),
    power_inventory_attack_wand_charge_weight => (50, "score", 0, 1000000, "borg_power.rs:1575", "power inventory attack wand charge weight: inherited policy; neutral default."),
    power_inventory_constitution_potion_bonus => (550000, "score", 0, 1000000, "borg_power.rs:1635", "power inventory constitution potion bonus: inherited policy; neutral default."),
    power_inventory_poisoned_cure_poison_bonus => (15000, "score", 0, 1000000, "borg_power.rs:1185", "Bonus for poison cures while poisoned (BI_ACUREPOIS)."),
    power_inventory_cursed_missile_penalty => (1000, "score", 0, 1000000, "borg_power.rs:1531", "power inventory cursed missile penalty: inherited policy; neutral default."),
    power_inventory_hurt_heal_bonus => (50000, "score", 0, 1000000, "borg_power.rs:1177", "Bonus for Healing while cut or poisoned (BI_AHEAL)."),
    power_inventory_destruction_staff_first_bonus => (5000, "score", 0, 1000000, "borg_power.rs:1540", "power inventory destruction staff first bonus: inherited policy; neutral default."),
    power_inventory_dexterity_potion_bonus => (550000, "score", 0, 1000000, "borg_power.rs:1632", "power inventory dexterity potion bonus: inherited policy; neutral default."),
    power_inventory_dungeon_book_spell_weight => (15000, "score", 0, 1000000, "borg_power.rs:1729", "power inventory dungeon book spell weight: inherited policy; neutral default."),
    power_inventory_early_attack_wand_bonus => (5000, "score", 0, 1000000, "borg_power.rs:1561", "power inventory early attack wand bonus: inherited policy; neutral default."),
    power_inventory_early_shovel_bonus => (5000, "score", 0, 1000000, "borg_power.rs:1696", "power inventory early shovel bonus: inherited policy; neutral default."),
    power_inventory_ego_missile_bonus => (100, "score", 0, 1000000, "borg_power.rs:1534", "power inventory ego missile bonus: inherited policy; neutral default."),
    power_inventory_enchant_armor_first_bonus => (540, "score", 0, 1000000, "borg_power.rs:1657", "power inventory enchant armor first bonus: inherited policy; neutral default."),
    power_inventory_enchant_damage_first_bonus => (500, "score", 0, 1000000, "borg_power.rs:1667", "power inventory enchant damage first bonus: inherited policy; neutral default."),
    power_inventory_enchant_hit_first_bonus => (540, "score", 0, 1000000, "borg_power.rs:1662", "power inventory enchant hit first bonus: inherited policy; neutral default."),
    power_inventory_encumbrance_penalty_weight => (1000, "score", 0, 1000000, "borg_power.rs:1816", "power inventory encumbrance penalty weight: inherited policy; neutral default."),
    power_inventory_first_empty_slot_bonus => (4000, "score", 0, 1000000, "borg_power.rs:1686", "power inventory first empty slot bonus: inherited policy; neutral default."),
    power_inventory_fuel_extra_base => (6000, "score", 0, 1000000, "borg_power.rs:1145", "power inventory fuel extra base: inherited policy; neutral default."),
    power_inventory_fuel_extra_decay => (100, "score", 0, 1000000, "borg_power.rs:1145", "power inventory fuel extra decay: inherited policy; neutral default."),
    power_inventory_holiness_staff_first_bonus => (2500, "score", 0, 1000000, "borg_power.rs:1580", "power inventory holiness staff first bonus: inherited policy; neutral default."),
    power_inventory_hungry_food_bonus => (100000, "score", 0, 1000000, "borg_power.rs:1153", "power inventory hungry food bonus: inherited policy; neutral default."),
    power_inventory_identify_first_bonus => (6000, "score", 0, 1000000, "borg_power.rs:1227", "power inventory identify first bonus: inherited policy; neutral default."),
    power_inventory_intelligence_potion_bonus => (520000, "score", 0, 1000000, "borg_power.rs:1620", "power inventory intelligence potion bonus: inherited policy; neutral default."),
    power_inventory_max_constitution_bonus => (50000, "score", 0, 1000000, "borg_power.rs:1606", "power inventory max constitution bonus: inherited policy; neutral default."),
    power_inventory_max_dexterity_bonus => (50000, "score", 0, 1000000, "borg_power.rs:1603", "power inventory max dexterity bonus: inherited policy; neutral default."),
    power_inventory_max_intelligence_bonus => (20000, "score", 0, 1000000, "borg_power.rs:1597", "power inventory max intelligence bonus: inherited policy; neutral default."),
    power_inventory_max_spell_stat_bonus => (50000, "score", 0, 1000000, "borg_power.rs:1611", "power inventory max spell stat bonus: inherited policy; neutral default."),
    power_inventory_max_strength_bonus => (50000, "score", 0, 1000000, "borg_power.rs:1594", "power inventory max strength bonus: inherited policy; neutral default."),
    power_inventory_max_wisdom_bonus => (20000, "score", 0, 1000000, "borg_power.rs:1600", "power inventory max wisdom bonus: inherited policy; neutral default."),
    power_inventory_other_quiver_slot_penalty => (10000, "score", 0, 1000000, "borg_power.rs:1525", "power inventory other quiver slot penalty: inherited policy; neutral default."),
    power_inventory_phase_first_bonus => (50000, "score", 0, 1000000, "borg_power.rs:1271", "power inventory phase first bonus: inherited policy; neutral default."),
    power_inventory_hurt_ccw_bonus => (100000, "score", 0, 1000000, "borg_power.rs:1174", "Bonus for Cure Critical Wounds while cut or poisoned (BI_ACCW)."),
    power_inventory_ranged_missile_excess_penalty => (1000, "score", 0, 1000000, "borg_power.rs:1520", "power inventory ranged missile excess penalty: inherited policy; neutral default."),
    power_inventory_ranged_quiver_slot_penalty => (10000, "score", 0, 1000000, "borg_power.rs:1509", "power inventory ranged quiver slot penalty: inherited policy; neutral default."),
    power_inventory_recall_rod_bonus => (12000, "score", 0, 1000000, "borg_power.rs:1263", "power inventory recall rod bonus: inherited policy; neutral default."),
    power_inventory_recharge_first_bonus => (5000, "score", 0, 1000000, "borg_power.rs:1490", "power inventory recharge first bonus: inherited policy; neutral default."),
    power_inventory_remove_curse_bonus => (90000, "score", 0, 1000000, "borg_power.rs:1644", "power inventory remove curse bonus: inherited policy; neutral default."),
    power_inventory_restore_experience_bonus => (50000, "score", 0, 1000000, "borg_power.rs:1650", "power inventory restore experience bonus: inherited policy; neutral default."),
    power_inventory_spell_stat_potion_bonus => (575000, "score", 0, 1000000, "borg_power.rs:1626", "power inventory spell stat potion bonus: inherited policy; neutral default."),
    power_inventory_star_enchant_armor_bonus => (5000, "score", 0, 1000000, "borg_power.rs:1677", "power inventory star enchant armor bonus: inherited policy; neutral default."),
    power_inventory_star_enchant_weapon_bonus => (5000, "score", 0, 1000000, "borg_power.rs:1672", "power inventory star enchant weapon bonus: inherited policy; neutral default."),
    power_inventory_star_remove_curse_bonus => (90000, "score", 0, 1000000, "borg_power.rs:1641", "power inventory star remove curse bonus: inherited policy; neutral default."),
    power_inventory_strength_potion_bonus => (550000, "score", 0, 1000000, "borg_power.rs:1617", "power inventory strength potion bonus: inherited policy; neutral default."),
    power_inventory_teleport_first_bonus => (10000, "score", 0, 1000000, "borg_power.rs:1293", "power inventory teleport first bonus: inherited policy; neutral default."),
    power_inventory_teleport_other_wand_first_bonus => (5000, "score", 0, 1000000, "borg_power.rs:1547", "power inventory teleport other wand first bonus: inherited policy; neutral default."),
    power_inventory_warrior_early_attack_wand_bonus => (10000, "score", 0, 1000000, "borg_power.rs:1572", "power inventory warrior early attack wand bonus: inherited policy; neutral default."),
    power_inventory_warrior_teleport_other_bonus => (50000, "score", 0, 1000000, "borg_power.rs:1551", "power inventory warrior teleport other bonus: inherited policy; neutral default."),
    power_inventory_wisdom_potion_bonus => (520000, "score", 0, 1000000, "borg_power.rs:1629", "power inventory wisdom potion bonus: inherited policy; neutral default."),
    sell_ammo_batch_count => (5, "count", 1, 5, "borg_store_sell.rs:380", "sell ammo batch count: inherited policy; neutral default."),
    sell_ammo_reserve_count => (45, "count", 0, 1000, "borg_store_sell.rs:1010", "sell ammo reserve count: inherited policy; neutral default."),
    sell_attack_device_min_level => (35, "level", 1, 50, "borg_store_sell.rs:1030", "sell attack device min level: inherited policy; neutral default."),
    sell_batch_max_item_value => (5, "gold", 0, 1000000, "borg_store_sell.rs:367", "sell batch max item value: inherited policy; neutral default."),
    sell_batch_min_gold => (250, "gold", 0, 1000000, "borg_store_sell.rs:362", "sell batch min gold: inherited policy; neutral default."),
    sell_count_power_tolerance => (50, "score", 0, 100000, "borg_store_sell.rs:1335", "sell count power tolerance: inherited policy; neutral default."),
    sell_food_batch_count => (3, "count", 1, 5, "borg_store_sell.rs:386", "sell food batch count: inherited policy; neutral default."),
    sell_gold_greed_base => (100, "gold", 0, 1000000, "borg_store_sell.rs:1257", "sell gold greed base: inherited policy; neutral default."),
    sell_gold_greed_base_10 => (5, "gold", 0, 1000000, "borg_store_sell.rs:1258", "sell gold greed base 10: inherited policy; neutral default."),
    sell_gold_greed_base_13 => (10, "gold", 0, 1000000, "borg_store_sell.rs:1255", "sell gold greed base 13: inherited policy; neutral default."),
    sell_gold_greed_base_15 => (20, "gold", 0, 1000000, "borg_store_sell.rs:1271", "sell gold greed base 15: inherited policy; neutral default."),
    sell_gold_greed_base_20 => (35, "gold", 0, 1000000, "borg_store_sell.rs:1268", "sell gold greed base 20: inherited policy; neutral default."),
    sell_gold_greed_base_25 => (50, "gold", 0, 1000000, "borg_store_sell.rs:1265", "sell gold greed base 25: inherited policy; neutral default."),
    sell_gold_greed_base_5 => (0, "gold", 0, 1000000, "borg_store_sell.rs:1261", "sell gold greed base 5: inherited policy; neutral default."),
    sell_gold_greed_ceiling => (25000, "gold", 0, 1000000, "borg_store_sell.rs:1260", "sell gold greed ceiling: inherited policy; neutral default."),
    sell_gold_greed_deep_depth => (50, "depth", 0, 127, "borg_store_sell.rs:1261", "sell gold greed deep depth: inherited policy; neutral default."),
    sell_gold_greed_deep_value => (75000, "gold", 0, 1000000, "borg_store_sell.rs:1262", "sell gold greed deep value: inherited policy; neutral default."),
    sell_gold_greed_divisor => (100, "divisor", 1, 10000, "borg_store_sell.rs:1241", "sell gold greed divisor: inherited policy; neutral default."),
    sell_gold_greed_floor => (1000, "gold", 0, 1000000, "borg_store_sell.rs:1260", "sell gold greed floor: inherited policy; neutral default."),
    sell_gold_greed_level_10 => (10, "level", 1, 50, "borg_store_sell.rs:1257", "sell gold greed level 10: inherited policy; neutral default."),
    sell_gold_greed_level_13 => (13, "level", 1, 50, "borg_store_sell.rs:1254", "sell gold greed level 13: inherited policy; neutral default."),
    sell_gold_greed_level_15 => (15, "level", 1, 50, "borg_store_sell.rs:1270", "sell gold greed level 15: inherited policy; neutral default."),
    sell_gold_greed_level_20 => (20, "level", 1, 50, "borg_store_sell.rs:1267", "sell gold greed level 20: inherited policy; neutral default."),
    sell_gold_greed_level_25 => (25, "level", 1, 50, "borg_store_sell.rs:1264", "sell gold greed level 25: inherited policy; neutral default."),
    sell_gold_greed_level_5 => (5, "level", 1, 50, "borg_store_sell.rs:1260", "sell gold greed level 5: inherited policy; neutral default."),
    sell_healing_rod_reserve_count => (3, "count", 0, 1000, "borg_store_sell.rs:1015", "sell healing rod reserve count: inherited policy; neutral default."),
    sell_home_stack_soft_limit => (90, "count", 0, 90, "borg_store_sell.rs:679", "sell home stack soft limit: inherited policy; neutral default."),
    sell_identify_max_depth => (35, "depth", 0, 127, "borg_store_sell.rs:929", "sell identify max depth: inherited policy; neutral default."),
    sell_mana_reserve_count => (99, "count", 0, 1000, "borg_store_sell.rs:920", "sell mana reserve count: inherited policy; neutral default."),
    sell_mana_reserve_min_sp => (100, "sp", 0, 10000, "borg_store_sell.rs:917", "sell mana reserve min sp: inherited policy; neutral default."),
    sell_mapping_rod_reserve_count => (2, "count", 0, 255, "borg_store_sell.rs:1023", "sell mapping rod reserve count: inherited policy; neutral default."),
    sell_unidentified_max_level => (10, "level", 1, 50, "borg_store_sell.rs:898", "sell unidentified max level: inherited policy; neutral default."),
    sell_value_cap => (30000, "gold", 0, 1000000, "borg_store_sell.rs:133", "sell value cap: inherited policy; neutral default."),
    shopping_emergency_light_max_level => (10, "level", 1, 50, "borg_think_store.rs:235", "shopping emergency light max level: inherited policy; neutral default."),
    shopping_light_min_gold => (100, "gold", 0, 1000000, "borg_think_store.rs:224", "shopping light min gold: inherited policy; neutral default."),
    shopping_max_bounces => (1350, "count", 0, 30000, "borg_think_store.rs:203", "shopping max bounces: inherited policy; neutral default."),
    shopping_max_turns => (2000, "turns", 0, 30000, "borg_think_store.rs:200", "shopping max turns: inherited policy; neutral default."),
    shopping_unlit_min_level => (2, "level", 1, 50, "borg_think_store.rs:218", "shopping unlit min level: inherited policy; neutral default."),
    wear_backup_danger_divisor => (2, "divisor", 1, 100, "borg_item_wear.rs:789", "wear backup danger divisor: inherited policy; neutral default."),
    wear_backup_max_bounces => (300, "count", 0, 30000, "borg_item_wear.rs:680", "wear backup max bounces: inherited policy; neutral default."),
    wear_backup_unique_danger_divisor => (3, "divisor", 1, 100, "borg_item_wear.rs:789", "wear backup unique danger divisor: inherited policy; neutral default."),
    wear_backup_unique_danger_numerator => (2, "multiplier", 0, 100, "borg_item_wear.rs:789", "wear backup unique danger numerator: inherited policy; neutral default."),
    wear_best_max_bounces => (1300, "count", 0, 30000, "borg_item_wear.rs:903", "wear best max bounces: inherited policy; neutral default."),
    wear_best_max_turns => (2000, "turns", 0, 30000, "borg_item_wear.rs:900", "wear best max turns: inherited policy; neutral default."),
    wear_full_ring_trial_rng_bound => (100, "rng_bound", 11, 10000, "borg_item_wear.rs:1025", "wear full ring trial rng bound: inherited policy; neutral default."),
    wear_identify_artifact_bonus => (150000, "score", 0, 1000000, "borg_item_wear.rs:190", "wear identify artifact bonus: inherited policy; neutral default."),
    wear_identify_consumable_depth_weight => (500, "score", 0, 100000, "borg_item_wear.rs:258", "wear identify consumable depth weight: inherited policy; neutral default."),
    wear_identify_consumable_min_depth => (5, "depth", 0, 127, "borg_item_wear.rs:256", "wear identify consumable min depth: inherited policy; neutral default."),
    wear_identify_device_depth_weight => (2000, "score", 0, 100000, "borg_item_wear.rs:253", "wear identify device depth weight: inherited policy; neutral default."),
    wear_identify_ego_bonus => (100000, "score", 0, 1000000, "borg_item_wear.rs:194", "wear identify ego bonus: inherited policy; neutral default."),
    wear_identify_food_depth_weight => (10, "score", 0, 100000, "borg_item_wear.rs:261", "wear identify food depth weight: inherited policy; neutral default."),
    wear_identify_jewelry_depth_weight => (5000, "score", 0, 100000, "borg_item_wear.rs:251", "wear identify jewelry depth weight: inherited policy; neutral default."),
    wear_identify_magic_bonus => (20000, "score", 0, 1000000, "borg_item_wear.rs:199", "wear identify magic bonus: inherited policy; neutral default."),
    wear_identify_max_danger => (1, "danger", 0, 10000, "borg_item_wear.rs:171", "wear identify max danger: inherited policy; neutral default."),
    wear_identify_min_sp => (50, "sp", 0, 10000, "borg_item_wear.rs:163", "wear identify min sp: inherited policy; neutral default."),
    wear_identify_rod_depth_weight => (3000, "score", 0, 100000, "borg_item_wear.rs:252", "wear identify rod depth weight: inherited policy; neutral default."),
    wear_min_power_gain => (50, "score", 0, 100000, "borg_item_wear.rs:1015", "wear min power gain: inherited policy; neutral default."),
    wear_one_ring_power_multiplier => (2, "multiplier", 1, 100, "borg_item_wear.rs:549", "wear one ring power multiplier: inherited policy; neutral default."),
    wear_repeat_cooldown_turns => (10, "turns", 0, 30000, "borg_item_wear.rs:949", "wear repeat cooldown turns: inherited policy; neutral default."),
    wear_ring_reorder_max_turns => (1000, "turns", 0, 30000, "borg_item_wear.rs:340", "wear ring reorder max turns: inherited policy; neutral default."),
    wear_ring_select_max_turns => (2000, "turns", 0, 30000, "borg_item_wear.rs:496", "wear ring select max turns: inherited policy; neutral default."),
    sell_unidentified_ego_min_copies => (2, "count", 1, 99, "borg_store_sell.rs:956", "sell unidentified ego min copies: inherited policy; neutral default."),
    shopping_upgrade_light_radius => (1, "grids", 1, 5, "borg_think_store.rs:224", "shopping upgrade light radius: inherited policy; neutral default."),

    // borg-reborn-rs (#680): enchant/decurse policy, neutral inherited defaults.
    decurse_spell_fail => (40, "percent", 0, 100, "borg_item_decurse.rs:65", "Maximum acceptable Remove Curse spell failure rate."),
    enchant_spell_fail => (65, "percent", 0, 100, "borg_item_enchant.rs:98", "Maximum acceptable enchant spell failure rate, for availability and casting."),
    enchant_brand_spell_fail => (65, "percent", 0, 100, "borg_item_enchant.rs:486", "Maximum acceptable Brand Ammunition spell failure rate."),
    enchant_plain_target => (8, "bonus", 0, 20, "borg_item_enchant.rs:96", "Target bonus without an acceptable spell or star-enchant scroll."),
    enchant_ammo_target => (10, "bonus", 0, 20, "borg_item_enchant.rs:391", "Ammo target with an acceptable spell or star-enchant scroll; equipment keeps BORG_ENCHANT_LIMIT."),
    enchant_min_shooter_power => (3, "multiplier", 0, 10, "borg_item_enchant.rs:318", "Minimum launcher multiplier to enchant a plain bow or ammunition; other ego/artifact gates unchanged."),
    enchant_ammo_min_quantity => (5, "count", 1, 255, "borg_item_enchant.rs:373", "Minimum ammunition stack eligible for enchantment."),

    // borg-reborn-rs (#1151): a worn curse's stat modifiers.
    trait_curse_stats => (0, "flag", 0, 1, "borg_trait.rs:1346", "#1151: 1 = the stat modifiers of a worn dullness, sickliness, weakness or clumsiness curse count in the Borg's stats (curse.txt; the engine adds them to stat_add like an item's own, player-calcs.c:calc_bonuses), so its carrying capacity, spell stat and the power model's price of the cursed item match the engine. 0 = C: the curse's BI_CINT.. changes are overwritten by stat_cur and the stats ignore the curse (a CL17 Gnome Mage wore a sickliness and dullness cap at -12 speed, 5 STR short of what the Borg believed)."),
    enchant_brand_min_quantity => (20, "count", 1, 255, "borg_item_enchant.rs:447", "Minimum ammunition stack eligible for branding."),
    enchant_dungeon_max_turns => (5500, "turns", 0, 100000, "borg_item_enchant.rs:507", "Elapsed level-turn cutoff before dungeon curse removal; strict greater-than comparison."),
    enchant_town_max_turns => (3501, "turns", 0, 100000, "borg_item_enchant.rs:508", "Elapsed level-turn cutoff before town curse removal and enchantment; strict greater-than comparison."),
}

/// borg-reborn-rs (#520): the class names, `class.txt` order (`cidx`).
pub(crate) const CLASSES: [&str; 9] =
    ["warrior", "mage", "druid", "priest", "necromancer", "paladin", "rogue", "ranger", "blackguard"];

/// borg-reborn-rs (#1061, owner 2026-11-22: "race would be a delta"): the
/// race names, `p_race.txt` order (`ridx`: init.c:2869-2877 numbers the list
/// in file order), in lower case with `-` as `_`. A setting's race is its
/// `__<race>` (or `@<race>`) suffix, as a class's is.
pub(crate) const RACES: [&str; 11] = [
    "human", "half_elf", "elf", "hobbit", "gnome", "dwarf", "half_orc", "half_troll", "dunadan",
    "high_elf", "kobold",
];

/// A setting's scope in the parse keys: `Some(c)` for class `c`,
/// `Some(RACE_SCOPE + r)` for race `r` (#1061), `None` for the common value.
const RACE_SCOPE: usize = CLASSES.len();

/// The scope of a class or race suffix (`None`: neither).
fn suffix_scope(suffix: &str) -> Option<usize> {
    CLASSES.iter().position(|k| *k == suffix)
        .or_else(|| RACES.iter().position(|k| *k == suffix).map(|r| RACE_SCOPE + r))
}

/// A class's own value of a parameter (`<name>@<class>`, a class file, or a
/// legacy `<name>_<class>` alias).
#[derive(Serialize)]
struct ClassValue {
    value: i32,
    origin: &'static str,
}

#[derive(Serialize)]
struct Setting<'a> {
    #[serde(flatten)]
    spec: &'a Spec,
    value: i32,
    origin: &'static str,
    /// borg-reborn-rs (#520): per-class values, by class name.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    class_values: BTreeMap<&'static str, ClassValue>,
    /// borg-reborn-rs (#1061): per-race values, by race name.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    race_values: BTreeMap<&'static str, ClassValue>,
}

/// A parameter's values: the common one, then the class's, then the race's
/// (#1061). `pick` reads the class's value first, then the race's, then the
/// common one.
#[derive(Clone, Copy)]
struct Resolved {
    value: i32,
    origin: &'static str,
    class: [Option<(i32, &'static str)>; 9],
    race: [Option<(i32, &'static str)>; 11],
}

static VALUES: OnceLock<Vec<Resolved>> = OnceLock::new();

/// borg-reborn-rs (#520): the character's class (`cidx`), -1 while none is
/// known. Set when the parameters load and whenever the Borg notices its
/// race and class (start and reincarnation).
static CLASS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);

/// borg-reborn-rs (#520): the class whose values `value` returns.
pub(crate) fn set_class(class: i32) {
    CLASS.store(class, std::sync::atomic::Ordering::Relaxed);
}

fn current_class() -> Option<usize> {
    let class = usize::try_from(CLASS.load(std::sync::atomic::Ordering::Relaxed)).ok()?;
    (class < CLASSES.len()).then_some(class)
}

/// borg-reborn-rs (#1061): the character's race (`ridx`), -1 while none is
/// known; set beside the class.
static RACE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);

/// borg-reborn-rs (#1061): the race whose values `value` returns.
pub(crate) fn set_race(race: i32) {
    RACE.store(race, std::sync::atomic::Ordering::Relaxed);
}

fn current_race() -> Option<usize> {
    let race = usize::try_from(RACE.load(std::sync::atomic::Ordering::Relaxed)).ok()?;
    (race < RACES.len()).then_some(race)
}

fn pick(r: &Resolved, class: Option<usize>) -> (i32, &'static str) {
    pick_for(r, class, current_race())
}

/// The class's value, else the race's (#1061), else the common one.
fn pick_for(r: &Resolved, class: Option<usize>, race: Option<usize>) -> (i32, &'static str) {
    class.and_then(|c| r.class[c])
        .or_else(|| race.and_then(|k| r.race[k]))
        .unwrap_or((r.value, r.origin))
}

fn compiled_default(spec: &Spec, class: Option<usize>) -> i32 {
    spec.class_defaults.and_then(|defaults| class.map(|class| defaults[class])).unwrap_or(spec.default)
}

pub(crate) fn resolved(param: Param) -> (i32, &'static str) {
    if let Some(v) = breeder_policy_switch(param) {
        return (v, "caution_breeder_policy");
    }
    let i = param as usize;
    VALUES.get().map_or_else(|| (compiled_default(&SPECS[i], current_class()), "default"), |values| pick(&values[i], current_class()))
}

/// Pure lookup after init: the current class's value when it has one, else
/// the common one. Isolated decision tests use the same defaults.
pub(crate) fn value(param: Param) -> i32 {
    if let Some(v) = breeder_policy_switch(param) {
        return v;
    }
    let i = param as usize;
    VALUES.get().map_or_else(|| compiled_default(&SPECS[i], current_class()), |values| pick(&values[i], current_class()).0)
}

thread_local! {
    /// borg-reborn-rs (#878): inside `damage_only`, the danger's fear terms
    /// read 0. Per thread: the game runs on one, the unit tests on many.
    static DAMAGE_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// borg-reborn-rs (#878): a danger parameter that prices something other
/// than hit points: every `danger_*_fear` / `danger_*_fear_<n>` term (never a
/// divisor: they are only added), and the breeder and friends bonuses of
/// `danger_one_kill` (borg_danger.rs, multipliers in tenths: 0 adds
/// nothing). The fear maps are skipped by `borg_danger` itself.
fn fear_term_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("danger_") else { return false };
    // `_fear`, `_fear_2` ... `_fear_4`: the numbered prices of one effect.
    let base = match rest.rsplit_once('_') {
        Some((head, n)) if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => head,
        _ => rest,
    };
    base.ends_with("_fear")
        || matches!(name, "danger_breeder_melee_extra_tenths" | "danger_breeder_spell_extra_tenths"
            | "danger_friends_melee_extra_tenths" | "danger_friends_melee_low_extra_tenths"
            | "danger_friends_spell_extra_tenths")
}

fn is_fear_term(i: usize) -> bool {
    static TERMS: OnceLock<Vec<bool>> = OnceLock::new();
    // A switch is never a price (review of #891, F1:
    // `danger_hunt_breeder_no_acid_fear` is a flag).
    TERMS.get_or_init(|| SPECS.iter().map(|s| !matches!(s.unit, "flag" | "mode" | "choice") && fear_term_name(s.name)).collect())[i]
}

/// borg-reborn-rs (#878): whether a `damage_only` scope is open (the fear
/// maps of `borg_danger` read it).
pub(crate) fn damage_only_active() -> bool {
    DAMAGE_ONLY.with(|d| d.get())
}

/// borg-reborn-rs (#878): `value` as the danger reads it: inside a
/// `damage_only` scope a fear term (`fear_term_name`) is 0. Only
/// borg_danger.rs reads the fear terms; `value` itself is unchanged (its
/// size decides what the release unit tests link).
pub(crate) fn danger_value(param: Param) -> i32 {
    if damage_only_active() && is_fear_term(param as usize) {
        0
    } else {
        value(param)
    }
}

/// borg-reborn-rs (#878): `f` with the danger's fear prices at 0
/// (`fear_term_name`): what the danger says about hit points alone. Draws no
/// random numbers of its own.
pub(crate) fn damage_only<T>(f: impl FnOnce() -> T) -> T {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) { DAMAGE_ONLY.with(|d| d.set(self.0)); }
    }
    let _reset = Reset(DAMAGE_ONLY.with(|d| d.replace(true)));
    f()
}

/// borg-reborn-rs (#722): the layer of `caution_breeder_policy` that turns
/// each breeder switch on (`None`: not one of them).
fn breeder_policy_layer(param: Param) -> Option<i32> {
    match param {
        Param::caution_breeder_fight | Param::danger_hunt_breeder_no_acid_fear => Some(1),
        Param::recover_rest_breeders => Some(2),
        Param::caution_breeder_stoppable => Some(3),
        Param::caution_breeder_avoid => Some(4),
        Param::caution_breeder_exit => Some(5),
        _ => None,
    }
}

/// borg-reborn-rs (#722): with `caution_breeder_policy` 0..5, the breeder
/// switches follow it (each on from its layer); at -1 (compiled) they are
/// read as set.
fn breeder_policy_switch(param: Param) -> Option<i32> {
    let layer = breeder_policy_layer(param)?;
    let i = Param::caution_breeder_policy as usize;
    let policy = VALUES.get().map_or(SPECS[i].default, |values| pick(&values[i], current_class()).0);
    applied_breeder_layer(policy, layer)
}

fn applied_breeder_layer(policy: i32, layer: i32) -> Option<i32> {
    (policy >= 0).then(|| i32::from(policy >= layer))
}

/// borg-reborn-rs (#520): the per-class names of the families that #520
/// folded into one parameter, accepted as aliases of `<name>@<class>`. A
/// value equal to the family's old "use the general value" sentinel sets no
/// class value (divisor and curve scale 0, reserve -1).
fn legacy_alias(name: &str) -> Option<(&'static Spec, usize, Option<i32>)> {
    let spec = |n: &str| SPECS.iter().find(|s| s.name == n);
    for (c, class) in CLASSES.iter().enumerate() {
        if let Some(fam) = name.strip_prefix(&format!("attack_preference_{class}_")).and_then(|r| r.strip_suffix("_permille")) {
            return spec(&format!("attack_preference_{fam}_permille")).map(|s| (s, c, None));
        }
        for (base, sentinel) in [
            ("attack_arrow_point_blank_divisor", 0),
            ("town_heal_curve_scale_percent", 0),
            ("town_heal_reserve_hp_percent", -1),
        ] {
            if name == format!("{base}_{class}") {
                return spec(base).map(|s| (s, c, Some(sentinel)));
            }
        }
    }
    None
}

/// A setting's name: the parameter, its class if any, and the alias's
/// "general value" sentinel.
fn split_name(name: &str) -> Result<(&'static Spec, Option<usize>, Option<i32>), String> {
    // #696: `<name>__<class>` is `<name>@<class>` in the characters a zoo
    // profile can write (`[A-Za-z0-9_.-]`); no parameter name has `__`.
    // #1061: `<name>__<race>` / `<name>@<race>` the same way (RACES).
    let at;
    let name = match name.rsplit_once("__") {
        Some((base, suffix)) if !name.contains('@') && suffix_scope(suffix).is_some() => {
            at = format!("{base}@{suffix}");
            at.as_str()
        }
        _ => name,
    };
    if let Some((base, class)) = name.split_once('@') {
        let spec = SPECS.iter().find(|s| s.name == base).ok_or(format!("unknown parameter {base}"))?;
        let c = suffix_scope(class).ok_or(format!("{name}: unknown class or race {class}"))?;
        return Ok((spec, Some(c), None));
    }
    if let Some(spec) = SPECS.iter().find(|s| s.name == name) {
        return Ok((spec, None, None));
    }
    legacy_alias(name).map(|(s, c, sentinel)| (s, Some(c), sentinel)).ok_or(format!("unknown parameter {name}"))
}

/// The environment's name for a setting: `BORG_PARAM_<NAME>` with the class
/// as `__<CLASS>` (shells cannot export a name with `@`); `@` and the
/// legacy aliases are accepted too. Upper case only, as before #520: a
/// lower-case name is an unknown parameter, not a silent match.
fn split_env_name(name: &str) -> Result<(&'static Spec, Option<usize>, Option<i32>), String> {
    if name != name.to_ascii_uppercase() {
        return Err(format!("unknown parameter {name}"));
    }
    let lower = name.to_ascii_lowercase();
    if let Some((base, class)) = lower.rsplit_once("__") {
        if suffix_scope(class).is_some() {
            return split_name(&format!("{base}@{class}"));
        }
    }
    split_name(&lower)
}

/// Allow old profiles and environment overrides to load after settings are removed.
fn retired_parameter_notice(name: &str) -> Option<&'static str> {
    let base = match name.rsplit_once("__").or_else(|| name.rsplit_once('@')) {
        Some((base, class)) if suffix_scope(class).is_some() => base,
        _ => name,
    };
    match base {
        "prepared_depth_plan" | "prepared_depth_early_levels_per_dl"
        | "prepared_depth_early_until_level" | "prepared_depth_late_levels_below" =>
            Some("ignoring retired preparedness depth cap settings (#768)"),
        "item_test_unknown_potions" | "item_test_unknown_min_hp_percent"
        | "item_test_unknown_danger_turns" | "item_test_unknown_max_depth"
        | "item_test_unknown_min_hp" | "item_test_unknown_min_hp_cure_exempt"
        | "item_test_unknown_hold_turns" =>
            Some("ignoring retired unknown-potion test settings (#1213)"),
        _ => None,
    }
}

type Key = (&'static str, Option<usize>);

fn parse<'a>(
    text: &str,
    env: &BTreeMap<String, String>,
    specs: &'a [Spec],
) -> Result<Vec<Setting<'a>>, String> {
    let mut retired_notices = BTreeSet::new();
    // (parameter, class) -> (raw value, alias sentinel)
    let mut file: BTreeMap<Key, (String, Option<i32>)> = BTreeMap::new();
    for (line, raw) in text.lines().enumerate() {
        let raw = raw.split('#').next().unwrap_or("").trim();
        if !raw.starts_with("borg_param_") {
            continue;
        }
        let (name, value) = raw
            .split_once('=')
            .ok_or_else(|| format!("borg.txt:{}: expected name = integer", line + 1))?;
        let name = name.trim().strip_prefix("borg_param_").unwrap();
        if let Some(notice) = retired_parameter_notice(name) {
            retired_notices.insert(notice);
            continue;
        }
        let (spec, class, sentinel) = split_name(name).map_err(|_| format!("unknown borg_param_{name}"))?;
        if file.insert((spec.name, class), (value.trim().to_string(), sentinel)).is_some() {
            return Err(format!("duplicate borg_param_{name}"));
        }
    }
    let mut envs: BTreeMap<Key, (String, Option<i32>)> = BTreeMap::new();
    for (name, value) in env {
        if *name == name.to_ascii_uppercase() {
            if let Some(notice) = retired_parameter_notice(&name.to_ascii_lowercase()) {
                retired_notices.insert(notice);
                continue;
            }
        }
        let (spec, class, sentinel) = split_env_name(name).map_err(|_| format!("unknown BORG_PARAM_{name}"))?;
        if envs.insert((spec.name, class), (value.clone(), sentinel)).is_some() {
            return Err(format!("duplicate BORG_PARAM_{name}"));
        }
    }
    for notice in retired_notices {
        eprintln!("borg-reborn-rs: {notice}");
    }
    let check = |spec: &Spec, raw: &str| -> Result<i32, String> {
        let value = raw.parse::<i32>().map_err(|_| format!("{}: expected integer, got {raw:?}", spec.name))?;
        if !(spec.min..=spec.max).contains(&value) {
            return Err(format!("{}: {value} outside {}..={} {}", spec.name, spec.min, spec.max, spec.unit));
        }
        Ok(value)
    };
    let mut settings = Vec::with_capacity(specs.len());
    for spec in specs {
        let (value, origin) = if let Some((raw, _)) = envs.get(&(spec.name, None)) {
            (check(spec, raw)?, "environment")
        } else if let Some((raw, _)) = file.get(&(spec.name, None)) {
            (check(spec, raw)?, "borg.txt")
        } else {
            (spec.default, "default")
        };
        let mut class_values = BTreeMap::new();
        for (c, class) in CLASSES.iter().enumerate() {
            let found = envs.get(&(spec.name, Some(c))).map(|v| (v, "environment"))
                .or_else(|| file.get(&(spec.name, Some(c))).map(|v| (v, "borg.txt")));
            if let Some(((raw, sentinel), class_origin)) = found {
                let raw = raw.trim();
                // An alias at its old "general value" sentinel sets nothing.
                if sentinel.is_some() && raw.parse::<i32>().ok() == *sentinel {
                    continue;
                }
                class_values.insert(*class, ClassValue { value: check(spec, raw)?, origin: class_origin });
            } else if let Some(defaults) = spec.class_defaults.filter(|_| origin == "default") {
                class_values.insert(*class, ClassValue { value: defaults[c], origin: "default" });
            }
        }
        // #1061: the race values (no legacy aliases, no compiled defaults).
        let mut race_values = BTreeMap::new();
        for (r, race) in RACES.iter().enumerate() {
            let key = (spec.name, Some(RACE_SCOPE + r));
            let found = envs.get(&key).map(|v| (v, "environment"))
                .or_else(|| file.get(&key).map(|v| (v, "borg.txt")));
            if let Some(((raw, _), race_origin)) = found {
                race_values.insert(*race, ClassValue { value: check(spec, raw.trim())?, origin: race_origin });
            }
        }
        settings.push(Setting { spec, value, origin, class_values, race_values });
    }
    let value = |name: &str| settings.iter().find(|s| s.spec.name == name).map(|s| s.value);
    for (low, high) in [
        ("sell_gold_greed_floor", "sell_gold_greed_ceiling"),
        ("home_stat_str_low_limit", "home_stat_str_high_limit"),
        ("home_stat_dex_low_limit", "home_stat_dex_high_limit"),
        ("home_stat_con_low_limit", "home_stat_con_high_limit"),
        ("home_stat_caster_low_limit", "home_stat_caster_high_limit"),
        ("home_resistance_min_depth", "home_resistance_max_depth"),
    ] {
        if value(low).zip(value(high)).is_some_and(|(a, b)| a > b) {
            return Err(format!("{low} must not exceed {high}"));
        }
    }
    Ok(settings)
}

fn resolve(settings: &[Setting<'_>]) -> Vec<Resolved> {
    settings
        .iter()
        .map(|s| {
            let mut class = [None; 9];
            for (c, name) in CLASSES.iter().enumerate() {
                class[c] = s.class_values.get(name).map(|v| (v.value, v.origin));
            }
            let mut race = [None; 11];
            for (r, name) in RACES.iter().enumerate() {
                race[r] = s.race_values.get(name).map(|v| (v.value, v.origin));
            }
            Resolved { value: s.value, origin: s.origin, class, race }
        })
        .collect()
}

/// Called before initialization consumes RNG or makes decisions. Configuration
/// is immutable for the process, including reincarnations and core switches;
/// which class's values apply follows the character (`set_class`, #520).
pub(crate) fn initialize() {
    VALUES.get_or_init(|| {
        let result = (|| -> Result<Vec<Resolved>, String> {
            let text = crate::shell::borg_txt_contents()?;
            let mut env = BTreeMap::new();
            for (name, value) in std::env::vars_os() {
                if let Some(name) = name.to_str().and_then(|n| n.strip_prefix("BORG_PARAM_")) {
                    let value = value
                        .into_string()
                        .map_err(|_| format!("BORG_PARAM_{name}: invalid UTF-8"))?;
                    env.insert(name.to_string(), value);
                }
            }
            let settings = parse(&text, &env, SPECS)?;
            if let Some(dir) = std::env::var_os("BORG_RUN_DIR") {
                let path = std::path::PathBuf::from(dir).join("params-core.json");
                let mut json = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
                json.push(b'\n');
                std::fs::write(&path, json).map_err(|e| format!("{}: {e}", path.display()))?;
            }
            Ok(resolve(&settings))
        })();
        result.unwrap_or_else(|e| {
            eprintln!("borg-reborn-rs: core parameters: {e}");
            std::process::exit(2)
        })
    });
    if let Some(class) = crate::engine::class_cidx_if_set() {
        set_class(class as i32);
    }
    if let Some(race) = crate::engine::race_ridx_if_set() {
        set_race(race as i32);
    }
}

#[derive(Serialize)]
struct SettingsReceipt {
    schema: &'static str,
    class_index: usize,
    class_name: &'static str,
    effective_settings: BTreeMap<&'static str, i32>,
}

fn settings_receipt(values: &[Resolved], class: usize) -> SettingsReceipt {
    let class_index = Some(class);
    let policy = pick(&values[Param::caution_breeder_policy as usize], class_index).0;
    let effective_settings = PARAMETERS.iter().filter_map(|&param| {
        let spec = &SPECS[param as usize];
        let value = breeder_policy_layer(param).and_then(|layer| applied_breeder_layer(policy, layer))
            .unwrap_or_else(|| pick(&values[param as usize], class_index).0);
        (value != compiled_default(spec, Some(class))).then_some((spec.name, value))
    }).collect();
    SettingsReceipt {
        schema: "borg-settings-resolved@1",
        class_index: class,
        class_name: CLASSES[class],
        effective_settings,
    }
}

/// Write after birth (or load), once the active class is known. The run
/// directory holds the current character's receipt, including after rebirth.
pub(crate) fn write_settings_receipt() {
    let (Some(dir), Some(values), Some(class)) = (
        std::env::var_os("BORG_RUN_DIR"), VALUES.get(), current_class(),
    ) else {
        return;
    };
    let path = std::path::PathBuf::from(dir).join("settings-resolved.json");
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut json = serde_json::to_vec_pretty(&settings_receipt(values, class))?;
        json.push(b'\n');
        std::fs::write(&path, json)?;
        Ok(())
    })();
    if let Err(e) = result {
        eprintln!("borg-reborn-rs: {}: {e}", path.display());
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enchant_decurse_bounds_precedence_and_provenance() {
        let names = ["decurse_spell_fail", "enchant_spell_fail", "enchant_brand_spell_fail",
            "enchant_plain_target", "enchant_ammo_target", "enchant_min_shooter_power",
            "enchant_ammo_min_quantity", "enchant_brand_min_quantity",
            "enchant_dungeon_max_turns", "enchant_town_max_turns"];
        for name in names {
            let spec = SPECS.iter().find(|s| s.name == name).unwrap();
            for value in [spec.min - 1, spec.max + 1] {
                assert!(parse(&format!("borg_param_{name} = {value}"), &BTreeMap::new(), SPECS).is_err());
            }
            for value in [spec.min, spec.default, spec.max] {
                let env = BTreeMap::from([(name.to_uppercase(), value.to_string())]);
                let parsed = parse(&format!("borg_param_{name} = {}", spec.min), &env, SPECS).unwrap();
                let row = parsed.iter().find(|s| s.spec.name == name).unwrap();
                assert_eq!((row.value, row.origin), (value, "environment"));
                let json = serde_json::to_value(row).unwrap();
                assert_eq!(json["name"], name);
                assert_eq!(json["default"], spec.default);
                assert_eq!(json["value"], value);
            }
        }
    }

    #[test]
    fn birth_receipt_resolves_class_overrides_and_omits_compiled_defaults() {
        let settings = parse(
            "borg_param_town_phase_reserve = 1\n\
             borg_param_town_phase_reserve__ranger = 2\n\
             borg_param_town_phase_reserve__mage = 0\n\
             borg_param_caution_breeder_policy__ranger = 0\n",
            &BTreeMap::new(), SPECS,
        ).unwrap();
        let values = resolve(&settings);
        let ranger = serde_json::to_value(settings_receipt(&values, 7)).unwrap();
        assert_eq!(ranger["class_index"], 7);
        assert_eq!(ranger["class_name"], "ranger");
        assert_eq!(ranger["effective_settings"]["town_phase_reserve"], 2);
        assert_eq!(ranger["effective_settings"]["caution_breeder_policy"], 0);
        assert!(ranger["effective_settings"].get("town_phase_reserve_percent").is_none());
        assert_eq!(settings_receipt(&values, 0).effective_settings["town_phase_reserve"], 1);
        assert!(!settings_receipt(&values, 1).effective_settings.contains_key("town_phase_reserve"));
        assert!(settings_receipt(&resolve(&parse("", &BTreeMap::new(), SPECS).unwrap()), 7)
            .effective_settings.is_empty());
    }

    #[test]
    fn throw_point_blank_preserves_inheritance_and_validates_bounds() {
        let p = Param::attack_throw_point_blank_divisor;
        assert_eq!(SPECS[p as usize].default, 0);
        for valid in [0, 1, 5, 1000] {
            let settings = parse(&format!("borg_param_attack_throw_point_blank_divisor = {valid}"), &BTreeMap::new(), SPECS).unwrap();
            assert_eq!(settings[p as usize].value, valid);
        }
        for invalid in [-1, 1001] {
            assert!(parse(&format!("borg_param_attack_throw_point_blank_divisor = {invalid}"), &BTreeMap::new(), SPECS).is_err());
        }
        let settings = parse("borg_param_attack_throw_point_blank_divisor__necromancer = 1", &BTreeMap::new(), SPECS).unwrap();
        let values = resolve(&settings);
        assert_eq!(pick(&values[p as usize], Some(4)).0, 1);
        assert_eq!(pick(&values[p as usize], Some(7)).0, 0);
    }

    #[test]
    fn inventory_bounds_reject_unsafe_quantity_rng_and_ordering() {
        for text in [
            "borg_param_sell_ammo_batch_count = 0",
            "borg_param_sell_ammo_batch_count = 6",
            "borg_param_sell_food_batch_count = 6",
            "borg_param_sell_home_stack_soft_limit = 91",
            "borg_param_sell_mapping_rod_reserve_count = 256",
            "borg_param_sell_mapping_rod_reserve_count = 300",
            "borg_param_sell_mapping_rod_reserve_count = 1000",
            "borg_param_wear_full_ring_trial_rng_bound = 10",
            "borg_param_wear_full_ring_trial_rng_bound = 10001",
            "borg_param_wear_backup_max_bounces = 32768",
            "borg_param_buy_home_stock_gold_divisor = 0",
            "borg_param_sell_gold_greed_floor = 30000",
            "borg_param_home_stat_caster_low_limit = 30",
        ] {
            assert!(parse(text, &BTreeMap::new(), SPECS).is_err(), "{text}");
        }
        for valid in [0, 2, 255] {
            let settings = parse(&format!("borg_param_sell_mapping_rod_reserve_count = {valid}"), &BTreeMap::new(), SPECS).unwrap();
            assert_eq!(settings[Param::sell_mapping_rod_reserve_count as usize].value, valid);
        }
        for (p, max) in [
            (Param::sell_mapping_rod_reserve_count, i32::from(u8::MAX)),
            (Param::sell_ammo_batch_count, i32::from(u8::MAX)),
            (Param::sell_food_batch_count, i32::from(u8::MAX)),
            (Param::sell_home_stack_soft_limit, i32::from(u8::MAX)),
            (Param::sell_unidentified_ego_min_copies, i32::from(u8::MAX)),
            (Param::wear_backup_max_bounces, i32::from(i16::MAX)),
            (Param::wear_best_max_bounces, i32::from(i16::MAX)),
        ] {
            let spec = &SPECS[p as usize];
            assert!(spec.min >= 0 && spec.max <= max, "{} exceeds its operand type", spec.name);
        }
        let env = BTreeMap::from([("SELL_GOLD_GREED_CEILING".into(), "40000".into())]);
        assert!(parse("borg_param_sell_gold_greed_floor = 30000", &env, SPECS).is_ok());
    }

    /// #520: the class value of a setting and its origin.
    fn class_value(settings: &[Setting<'_>], p: Param, class: &str) -> Option<(i32, &'static str)> {
        settings[p as usize].class_values.get(class).map(|v| (v.value, v.origin))
    }

    #[test]
    fn attack_preferences_validate_bounds_and_record_override_provenance() {
        // #520: the legacy per-class name is an alias of `<name>@<class>`.
        let legacy = "attack_preference_mage_spell_permille";
        let p = Param::attack_preference_spell_permille;
        for name in [legacy, "attack_preference_spell_permille@mage"] {
            for bad in ["499", "2001", "1.5", "no", "2147483648"] {
                assert!(parse(&format!("borg_param_{name} = {bad}"), &BTreeMap::new(), SPECS).is_err(), "{name} {bad}");
            }
            for valid in [500, 1000, 2000] {
                let values = parse(&format!("borg_param_{name} = {valid}"), &BTreeMap::new(), SPECS).unwrap();
                assert_eq!(class_value(&values, p, "mage"), Some((valid, "borg.txt")));
                assert_eq!((values[p as usize].value, values[p as usize].origin), (1000, "default"), "the common value stays");
                assert_eq!(class_value(&values, p, "warrior"), None);
            }
        }
        let env = BTreeMap::from([("ATTACK_PREFERENCE_MAGE_SPELL_PERMILLE".into(), "1600".into())]);
        let values = parse(&format!("borg_param_{legacy} = 1500"), &env, SPECS).unwrap();
        assert_eq!(class_value(&values, p, "mage"), Some((1600, "environment")));
        let json = serde_json::to_value(&values[p as usize]).unwrap();
        assert_eq!(json["name"], "attack_preference_spell_permille");
        assert_eq!(json["unit"], "utility_permille");
        assert_eq!(json["default"], 1000);
        assert_eq!(json["class_values"]["mage"]["value"], 1600);
        assert_eq!(json["class_values"]["mage"]["origin"], "environment");
        // The same class value twice, by either name, is a duplicate.
        assert!(parse(&format!("borg_param_{legacy} = 1000\nborg_param_{legacy} = 1500"), &BTreeMap::new(), SPECS).is_err());
        assert!(parse(&format!("borg_param_{legacy} = 1000\nborg_param_attack_preference_spell_permille@mage = 1500"), &BTreeMap::new(), SPECS).is_err());
        assert!(parse("borg_param_attack_preference_mage_wait_permille = 1000", &BTreeMap::new(), SPECS).is_err());
        assert!(parse("borg_param_attack_preference_spell_permille@wizard = 1000", &BTreeMap::new(), SPECS).is_err());
        // #696: `<name>__<class>` is `<name>@<class>`, for every parameter.
        let d = Param::town_devices_percent;
        let values = parse("borg_param_town_devices_percent__mage = 150", &BTreeMap::new(), SPECS).unwrap();
        assert_eq!(class_value(&values, d, "mage"), Some((150, "borg.txt")));
        assert_eq!(values[d as usize].value, 100, "the common value stays");
        assert!(parse("borg_param_town_devices_percent__mage = 150\nborg_param_town_devices_percent@mage = 140", &BTreeMap::new(), SPECS).is_err(), "a duplicate");
        assert!(parse("borg_param_town_devices_percent__wizard = 150", &BTreeMap::new(), SPECS).is_err());
    }

    /// #520: precedence (default < borg.txt < class line < environment <
    /// class environment), the `__CLASS` environment form, and the legacy
    /// aliases' "general value" sentinels.
    #[test]
    fn class_values_override_the_common_value() {
        let p = Param::town_heal_reserve_hp_percent;
        let text = "borg_param_town_heal_reserve_hp_percent = 100\nborg_param_town_heal_reserve_hp_percent@mage = 300\nborg_param_town_heal_reserve_hp_percent_warrior = 50";
        let values = parse(text, &BTreeMap::new(), SPECS).unwrap();
        assert_eq!((values[p as usize].value, values[p as usize].origin), (100, "borg.txt"));
        assert_eq!(class_value(&values, p, "mage"), Some((300, "borg.txt")));
        assert_eq!(class_value(&values, p, "warrior"), Some((50, "borg.txt")));
        let env = BTreeMap::from([
            ("TOWN_HEAL_RESERVE_HP_PERCENT".into(), "120".into()),
            ("TOWN_HEAL_RESERVE_HP_PERCENT__MAGE".into(), "250".into()),
        ]);
        let values = parse(text, &env, SPECS).unwrap();
        assert_eq!((values[p as usize].value, values[p as usize].origin), (120, "environment"));
        assert_eq!(class_value(&values, p, "mage"), Some((250, "environment")));
        assert_eq!(class_value(&values, p, "warrior"), Some((50, "borg.txt")));
        // The old "use the general value" sentinels set no class value.
        for text in ["borg_param_town_heal_reserve_hp_percent_mage = -1",
                     "borg_param_town_heal_curve_scale_percent_mage = 0",
                     "borg_param_attack_arrow_point_blank_divisor_mage = 0"] {
            let values = parse(text, &BTreeMap::new(), SPECS).unwrap();
            assert!(
                values.iter().all(|s| s.class_values.values().all(|v| v.origin == "default")),
                "{text}"
            );
        }
        // #534 review: environment names stay upper case.
        let lower = BTreeMap::from([("town_heal_reserve_hp_percent".into(), "100".into())]);
        assert!(parse("", &lower, SPECS).is_err());
        let lower_class = BTreeMap::from([("TOWN_HEAL_RESERVE_HP_PERCENT__mage".into(), "100".into())]);
        assert!(parse("", &lower_class, SPECS).is_err());
        // Without the alias, the sentinel is out of range.
        assert!(parse("borg_param_town_heal_reserve_hp_percent@mage = -1", &BTreeMap::new(), SPECS).is_err());
        assert!(parse("borg_param_attack_arrow_point_blank_divisor_ranger = 2", &BTreeMap::new(), SPECS).is_ok());
    }

    /// #1061: a race suffix (`__<race>`, `@<race>`, `BORG_PARAM_..__<RACE>`)
    /// sets a race value; a race name is never a class name, so neither
    /// suffix can be read as the other.
    #[test]
    fn race_values_set_a_race_delta() {
        for race in RACES {
            assert!(!CLASSES.contains(&race), "{race} is also a class");
            assert!(!race.contains("__") && !race.contains('@'), "{race}");
        }
        let p = Param::stair_scum_max_hp_race_delta;
        let race_value = |settings: &[Setting<'_>], race: &str| {
            settings[p as usize].race_values.get(race).map(|v| (v.value, v.origin))
        };
        let text = "borg_param_stair_scum_max_hp_race_delta__hobbit = 3\nborg_param_stair_scum_max_hp_race_delta@kobold = 2";
        let env = BTreeMap::from([("STAIR_SCUM_MAX_HP_RACE_DELTA__HALF_TROLL".into(), "-5".into())]);
        let values = parse(text, &env, SPECS).unwrap();
        assert_eq!(values[p as usize].value, 0);
        assert_eq!(race_value(&values, "hobbit"), Some((3, "borg.txt")));
        assert_eq!(race_value(&values, "kobold"), Some((2, "borg.txt")));
        assert_eq!(race_value(&values, "half_troll"), Some((-5, "environment")));
        assert_eq!(race_value(&values, "human"), None);
        assert!(values[p as usize].class_values.is_empty(), "a race is not a class");
        // A race value is range-checked, and an unknown race is an unknown name.
        assert!(parse("borg_param_stair_scum_max_hp_race_delta__elf = 101", &BTreeMap::new(), SPECS).is_err());
        assert!(parse("borg_param_stair_scum_max_hp_race_delta__orc = 1", &BTreeMap::new(), SPECS).is_err());
        // The JSON carries race_values only where some are set.
        let json = serde_json::to_value(&values).unwrap();
        assert_eq!(json[p as usize]["race_values"]["hobbit"]["value"], 3);
        assert!(json[Param::stair_scum as usize].get("race_values").is_none());
        // The class's value comes first, then the race's, then the common one.
        let resolved = resolve(&values);
        let r = resolved[p as usize];
        assert_eq!(pick_for(&r, Some(1), Some(3)), (3, "borg.txt"), "a hobbit mage: the race");
        assert_eq!(pick_for(&r, Some(1), Some(0)), (0, "default"), "a human mage: the common value");
        let mut class = [None; 9];
        class[1] = Some((7, "borg.txt"));
        let r = Resolved { class, ..r };
        assert_eq!(pick_for(&r, Some(1), Some(3)), (7, "borg.txt"), "a class value wins");
    }

    /// #520: `pick` returns the current class's value, else the common one.
    #[test]
    fn lookup_follows_the_class() {
        let mut class = [None; 9];
        class[1] = Some((300, "borg.txt"));
        let r = Resolved { value: 100, origin: "borg.txt", class, race: [None; 11] };
        assert_eq!(pick(&r, None), (100, "borg.txt"));
        assert_eq!(pick(&r, Some(1)), (300, "borg.txt"));
        assert_eq!(pick(&r, Some(0)), (100, "borg.txt"));
    }

    #[test]
    fn m13_defaults_preserve_healing_fixes_and_retire_removed_branches() {
        for (name, expected) in [
            ("caution_clw_estimate_percent", 0), ("caution_clw_estimate_min", 20),
            ("caution_csw_estimate_percent", 0), ("caution_csw_estimate_min", 40),
            ("caution_ccw_estimate_percent", 0), ("caution_ccw_estimate_min", 60),
        ] {
            assert_eq!(SPECS.iter().find(|s| s.name == name).unwrap().default, expected);
        }
        for name in ["power_inventory_poison_cure_bonus", "power_inventory_cut_cure_bonus",
                     "power_inventory_cure_serious_first_bonus",
                     "recover_munchkin_rest_min_food", "power_inventory_maxclevel_threshold_2",
                     "defend_shield_low_danger_numerator", "defend_shield_low_danger_divisor",
                     "defend_shield_low_fail_adjustment"] {
            assert!(parse(&format!("borg_param_{name} = 1"), &BTreeMap::new(), SPECS).is_err());
        }
    }

    #[test]
    fn dive_gates_ship_explicit_values_and_accept_overrides() {
        let compiled = parse("", &BTreeMap::new(), SPECS).unwrap();
        let shipped = parse(include_str!("../../harness/borg.txt"), &BTreeMap::new(), SPECS).unwrap();
        for (param, expected) in [
            (Param::prepared_clevel_depth_margin, 0),
            (Param::prepared_dl2_min_clevel, 0),
            (Param::prepared_recall_depth, 5),
            (Param::prepared_cure_min, 2),
            (Param::prepared_cure_until_clevel, 30),
            (Param::prepared_mid_max_depth, 19),
            (Param::prepared_mid_caster_max_clevel, 28),
            (Param::prepared_deep_caster_ramp_max_clevel, 44),
            (Param::prepared_deep_caster_ramp_base_depth, 30),
            (Param::prepared_deep_caster_ramp_depth_per_level, 2),
            (Param::prepared_dl34_plus_min_clevel, 40),
        ] {
            assert_eq!(compiled[param as usize].value, expected);
            assert_eq!(shipped[param as usize].value, expected);
            assert_eq!(shipped[param as usize].origin, "borg.txt");
        }
        for (param, compiled_values, retained) in [
            (Param::prepared_dl34_min_hp, [50, 60, 40, 40, 60, 50, 50, 50, 50], false),
            (Param::prepared_dl59_min_hp, [60, 80, 60, 60, 80, 60, 60, 60, 60], false),
            (Param::prepared_dl34_min_clevel, [4, 11, 9, 9, 11, 4, 8, 4, 4], false),
            (Param::prepared_dl59_min_clevel, [6, 15, 15, 15, 15, 6, 10, 6, 6], false),
            (Param::prepared_mid_clevel_offset, [-4, 5, 0, 0, 5, 0, 0, 0, -4], true),
            (Param::prepared_deep_clevel_offset, [5, 8, 13, 13, 8, 7, 10, 8, 5], true),
            (Param::prepared_deep_min_clevel, [0, 0, 0, 0, 0, 0, 0, 29, 0], true),
            (Param::prepared_deep_max_clevel, [38, 38, 46, 46, 38, 40, 43, 41, 38], true),
        ] {
            for (i, class) in CLASSES.into_iter().enumerate() {
                assert_eq!(class_value(&compiled, param, class), Some((compiled_values[i], "default")));
                if retained {
                    assert_eq!(class_value(&shipped, param, class), Some((compiled_values[i], "borg.txt")));
                }
            }
        }
        let dl2_hp = Param::prepared_dl2_min_hp as usize;
        assert_eq!(compiled[dl2_hp].value, 30);
        assert_eq!((shipped[dl2_hp].value, shipped[dl2_hp].origin), (0, "borg.txt"));

        let dl34_levels = [7, 11, 10, 12, 13, 10, 8, 9, 9];
        let dl59_levels = [8, 13, 12, 14, 14, 12, 10, 11, 10];
        for (index, class) in CLASSES.into_iter().enumerate() {
            for param in [Param::prepared_dl34_min_hp, Param::prepared_dl59_min_hp] {
                assert_eq!(class_value(&shipped, param, class), Some((0, "borg.txt")), "{class}");
            }

            let (mix, _) = class_value(&shipped, Param::explore_mix_max_clevel, class).unwrap();
            for (param, expected) in [
                (Param::prepared_dl2_min_clevel, mix + 1),
                (Param::prepared_dl34_min_clevel, dl34_levels[index]),
                (Param::prepared_dl59_min_clevel, dl59_levels[index]),
            ] {
                assert_eq!(class_value(&shipped, param, class), Some((expected, "borg.txt")), "{class}");
            }
        }

        let effective = resolve(&shipped);
        for (index, spec) in SPECS.iter().enumerate().filter(|(_, spec)| spec.name.starts_with("prepared_")) {
            for (class, name) in CLASSES.into_iter().enumerate() {
                assert_eq!(pick_for(&effective[index], Some(class), None).1, "borg.txt",
                    "{} for {name}", spec.name);
            }
        }

        let param = Param::prepared_mid_clevel_offset;
        let overrides = parse(
            "borg_param_prepared_mid_clevel_offset = -2\nborg_param_prepared_mid_clevel_offset__mage = 2",
            &BTreeMap::new(),
            SPECS,
        ).unwrap();
        assert_eq!(overrides[param as usize].value, -2);
        assert_eq!(class_value(&overrides, param, "warrior"), None);
        assert_eq!(class_value(&overrides, param, "mage"), Some((2, "borg.txt")));
    }

    #[test]
    fn retired_depth_cap_settings_can_load() {
        let text = "borg_param_prepared_depth_plan = 1\nborg_param_prepared_depth_early_until_level__warrior = 4";
        let env = BTreeMap::from([("PREPARED_DEPTH_LATE_LEVELS_BELOW__MAGE".into(), "3".into())]);
        let settings = parse(text, &env, SPECS).unwrap();
        assert_eq!(settings[Param::prepared_clevel_depth_margin as usize].value, 0);
        assert!(parse("borg_param_prepared_depth_plan__typo = 1", &env, SPECS).is_err());
    }

    #[test]
    fn retired_potion_test_settings_can_load_but_are_not_registered() {
        let text = "\
borg_param_item_test_unknown_potions = 1
borg_param_item_test_unknown_min_hp_percent@ranger = 50
borg_param_item_test_unknown_danger_turns = 10
borg_param_item_test_unknown_max_depth__half_orc = 30
borg_param_item_test_unknown_min_hp = 60
borg_param_item_test_unknown_min_hp_cure_exempt = 1
borg_param_item_test_unknown_hold_turns = 180
borg_param_attack_on_sight = 1";
        let env = BTreeMap::from([("ITEM_TEST_UNKNOWN_POTIONS__RANGER".into(), "1".into())]);
        let settings = parse(text, &env, SPECS).unwrap();
        assert!(!settings.iter().any(|s| s.spec.name.starts_with("item_test_unknown_")));
        assert_eq!(settings[Param::attack_on_sight as usize].value, 1);
        assert!(parse("borg_param_item_test_unknown_potions__typo = 1", &env, SPECS).is_err());
        assert!(parse("borg_param_item_test_unknown_scrolls = 1", &env, SPECS).is_err());
    }

    /// #878: what `damage_only` zeroes, and only inside the scope.
    #[test]
    fn damage_only_zeroes_the_fear_prices_alone() {
        for name in ["danger_monblow_poison_fear", "danger_monblow_blind_fear_2", "danger_rsf_ba_wate_fear_4", "danger_rsf_br_acid_fear",
            "danger_exp10_fear", "danger_breeder_melee_extra_tenths", "danger_friends_spell_extra_tenths"] {
            assert!(fear_term_name(name), "{name}");
        }
        for name in ["danger_monster_fear_bounce_limit", "danger_regional_fear_max_depth", "danger_morgoth_regional_fear_cap",
            "danger_breeder_melee_max_level", "danger_monblow_poison_hit_power", "caution_fear_x"] {
            assert!(!fear_term_name(name), "{name}");
        }
        let poison = value(Param::danger_monblow_poison_fear);
        assert_eq!(poison, 10, "C: z += 10");
        assert_eq!(danger_value(Param::danger_monblow_poison_fear), poison, "outside the scope");
        let inside = damage_only(|| {
            assert!(damage_only_active());
            (danger_value(Param::danger_monblow_poison_fear), danger_value(Param::danger_breeder_melee_extra_tenths),
                danger_value(Param::danger_monblow_poison_hit_power), value(Param::danger_monblow_poison_fear))
        });
        assert_eq!(inside, (0, 0, value(Param::danger_monblow_poison_hit_power), poison), "value itself never changes");
        assert!(!damage_only_active(), "closed again");
        assert!(fear_term_name("danger_hunt_breeder_no_acid_fear"), "the name alone would catch it");
        assert!(!is_fear_term(Param::danger_hunt_breeder_no_acid_fear as usize), "but it is a flag, not a price");
        assert_eq!((0..SPECS.len()).filter(|&i| is_fear_term(i)).count(), 119, "114 fear terms and 5 bonuses");
    }

    #[test]
    fn defaults_and_registry_are_valid() {
        let settings = parse("", &BTreeMap::new(), SPECS).unwrap();
        let mut names = std::collections::BTreeSet::new();
        for s in settings {
            assert!(names.insert(s.spec.name));
            assert_eq!(s.value, s.spec.default);
            assert_eq!(s.origin, "default");
            assert!(!s.spec.description.is_empty());
            let (file, line) = s.spec.source.split_once(':').unwrap();
            let prefix = s.spec.name.split('_').next().unwrap();
            let module = match prefix {
                "attack" => "fight_attack",
                "defend" => "fight_defend",
                "item" => "item_use",
                "wear" => "item_wear",
                "enchant" => "item_enchant",
                "decurse" => "item_decurse",
                "buy" => "store_buy",
                "sell" => "store_sell",
                "shopping" => "think_store",
                "home" => "home_power",
                "town" => "town_budget",
                "kill" => "flow_kill",
                "fast" => "fast_fragile",
                "lone" => "lone_target",
                "cave" => "cave_light",
                "view" => "cave_view",
                "unseen" => "unseen_belief",
                "trap" => "trap_way",
                "explore" => "flow_dark",
                "stair" => "stair_scum",
                "stairs" => "flow_stairs",
                "deal" => "poison",
                "restock" => "healing_restock",
                "caution" if s.spec.name.starts_with("caution_combat_heal") => "combat_heal",
                _ => prefix,
            };
            if prefix == "town" {
                // The town planner's parameters live in its borg_town_*.rs modules.
                assert!(file.starts_with("borg_town_") && file.ends_with(".rs"), "{file}");
            } else {
                assert_eq!(file, format!("borg_{module}.rs"));
            }
            assert!(line.parse::<usize>().unwrap() > 0);
        }
    }

    #[test]
    fn p2_divisors_and_narrow_timers_reject_unsafe_inputs() {
        for spec in SPECS
            .iter()
            .filter(|s| matches!(s.unit, "divisor" | "denominator multiplier"))
        {
            let text = format!("borg_param_{} = 0", spec.name);
            assert!(
                parse(&text, &BTreeMap::new(), SPECS).is_err(),
                "{}",
                spec.name
            );
        }
        for name in [
            "defend_see_invis_short_duration",
            "defend_see_invis_spell_duration",
            "defend_see_invis_potion_duration",
            "defend_refresh_resistance_duration",
        ] {
            let text = format!("borg_param_{name} = 32768");
            assert!(parse(&text, &BTreeMap::new(), SPECS).is_err());
        }
    }

    #[test]
    fn rejects_bad_values_and_names() {
        for text in [
            "borg_param_caution_clw_min_loss = 101",
            "borg_param_caution_clw_min_loss = -1",
            "borg_param_caution_clw_min_loss = foo",
            "borg_param_caution_clw_min_loss = 2147483648",
            "borg_param_caution_clw_danger_divisor = 0",
            "borg_param_typo = 1",
            "borg_param_caution_clw_min_loss",
            "borg_param_caution_clw_min_loss = 1\nborg_param_caution_clw_min_loss = 2",
        ] {
            assert!(parse(text, &BTreeMap::new(), SPECS).is_err(), "{text}");
        }
        assert!(parse("", &BTreeMap::from([("TYPO".into(), "1".into())]), SPECS).is_err());
    }

    #[test]
    fn danger_escape_configuration_bounds() {
        for text in [
            "borg_param_danger_fast_player_normal_speed = 190",
            "borg_param_danger_fast_player_threshold = 136",
            "borg_param_danger_summon_protected_divisor = 0",
            "borg_param_escape_landing_samples = 0",
            "borg_param_escape_landing_trials = 0",
            "borg_param_escape_phase_horizon = 0",
            "borg_param_escape_near_unique_max_count = 10",
        ] {
            assert!(parse(text, &BTreeMap::new(), SPECS).is_err(), "{text}");
        }
        let env = BTreeMap::from([("DANGER_MAXIMUM".into(), "1".into())]);
        let settings = parse("borg_param_escape_phase_tolerance = 12", &env, SPECS).unwrap();
        let cap = &settings[Param::danger_maximum as usize];
        assert_eq!((cap.value, cap.origin), (1, "environment"));
        let phase = &settings[Param::escape_phase_tolerance as usize];
        assert_eq!((phase.value, phase.origin), (12, "borg.txt"));
    }

    #[test]
    fn explicit_rest_durations_exclude_engine_repeat_sentinel() {
        // ui-command.c:textui_cmd_rest parses decimal turns; cmd-cave.c:
        // do_cmd_rest treats 1 as the previous rest count, not one turn.
        for name in ["caution_recall_rest_turns", "recover_rod_rest_turns"] {
            for value in [0, 1, 10000] {
                assert!(parse(
                    &format!("borg_param_{name} = {value}"),
                    &BTreeMap::new(),
                    SPECS
                )
                .is_err());
            }
            for value in [2, 12, 9999] {
                let values = parse(
                    &format!("borg_param_{name} = {value}"),
                    &BTreeMap::new(),
                    SPECS,
                )
                .unwrap();
                assert_eq!(
                    values.iter().find(|s| s.spec.name == name).unwrap().value,
                    value
                );
            }
        }
    }

    #[test]
    fn recovery_configuration_bounds_and_precedence() {
        for text in [
            "borg_param_recover_danger_divisor = 0",
            "borg_param_recover_danger_horizon = 0",
            "borg_param_recover_rod_rest_turns = 0",
            "borg_param_recover_cheap_poison_chance = 101",
            "borg_param_recover_rest_sp_tenths = 11",
        ] {
            assert!(parse(text, &BTreeMap::new(), SPECS).is_err(), "{text}");
        }
        let env = BTreeMap::from([("RECOVER_DANGER_DIVISOR".into(), "8".into())]);
        let values = parse("borg_param_recover_danger_divisor = 2", &env, SPECS).unwrap();
        let setting = &values[Param::recover_danger_divisor as usize];
        assert_eq!((setting.value, setting.origin), (8, "environment"));
    }

    #[test]
    fn environment_over_file_over_defaults_and_json_roundtrip() {
        let env = BTreeMap::from([("CAUTION_CLW_MIN_LOSS".into(), "20".into())]);
        let text = "# ignored\nborg_verbose = TRUE\nborg_param_caution_clw_min_loss = invalid\nborg_param_caution_csw_min_loss = 35 # comment";
        let settings = parse(text, &env, SPECS).unwrap();
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(
            json,
            serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&settings).unwrap())
                .unwrap()
        );
        let clw = settings
            .iter()
            .find(|s| s.spec.name == "caution_clw_min_loss")
            .unwrap();
        assert_eq!((clw.value, clw.origin), (20, "environment"));
        let csw = settings
            .iter()
            .find(|s| s.spec.name == "caution_csw_min_loss")
            .unwrap();
        assert_eq!((csw.value, csw.origin), (35, "borg.txt"));
    }
}
