//! The executor (ARCHITECTURE.md L6, task T3.1, #17).
//!
//! An [`Action`] becomes the complete key sequence: the command, item
//! letters, pre-answered prompts and the closing `ESCAPE`s. Between thinks
//! the [prompt catcher](answer) answers the game's prompts with the C
//! Borg's own answers, and each action carries the [effect](Effect) the
//! next think checks; a mismatch goes into the trace as a violation, so a
//! key sequence the game silently ignored becomes visible.
//!
//! Every key sequence and prompt answer here is lifted from the C Borg
//! with a `file:function:lines` citation (RULES 16); the choice of what to
//! do is not (that is the strategy's, from T3.3 on).
//!
//! Key disciplines (spec §2.6):
//!
//! * a dungeon action opens with `ESCAPE`, as every action of the minimal
//!   Borg has since M1, so any prompt or menu a previous key left open is
//!   dismissed rather than answered by this action's first key;
//! * an in-store sequence begins after the `,` that opens the store
//!   (`do_cmd_hold` runs the whole store visit inside one keypress,
//!   `cmd-cave.c:do_cmd_hold:1580`), so it cannot open with `ESCAPE`
//!   (that would leave the store) and ends with the `ESCAPE`s that close
//!   it, as the C Borg's do.

use crate::engine::{ESCAPE, KC_ENTER};
use crate::observation::Observation;

/// The item-selection letters of the pack, `h j k l` left out because they
/// are movement keys (`ui-menu.c:44`; the pack labels at
/// `ui-object.c:292`). The C Borg presses the same list
/// (`borg-fight-attack.c:2372`).
pub const PACK_LETTERS: &[u8] = b"abcdefgimnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// The store stock's selection letters with the original keyset
/// (`ui-store.c:805`; the C Borg's copy at `borg-store.c:36`). The ware at
/// index `i` of the stock is chosen with `SHOP_MENU_ITEMS[i]`.
pub const SHOP_LETTERS: &[u8] = b"acfhjmnoqruvyzABDFGHJKLMNOPQRSTUVWXYZ";

/// A decision of the strategy, as the executor carries it out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// Rest this many player turns (`R`, the count, `Enter`).
    Rest(u32),
    /// Take the down staircase underfoot (`>`).
    Descend,
    /// Take the up staircase underfoot (`<`).
    Ascend,
    /// Walk one step in a direction 1–9 (keypad layout): moves, opens a
    /// closed door, or attacks a monster that is in the way
    /// (`move_player`, cmd-cave.c).
    Walk(u8),
    /// Attack the monster one step in a direction 1–9: the same key as
    /// [`Action::Walk`] (walking into a monster attacks,
    /// `py_attack` via `move_player`), named for the intent so its
    /// expected effect can differ.
    Attack(u8),
    /// Pick up what is underfoot (`g`, `do_cmd_pickup`, cmd-pickup.c).
    Get,
    /// Stand still one turn (`,`): uses game time, except on a shop
    /// entrance, where it enters the store (see [`stand_still`]).
    Hold,
    /// Alter the grid in a direction 1–9 (`+` and the direction): always
    /// uses game time (see [`stand_still`]).
    Alter(u8),
    /// Buy the ware at this index of the store the Borg stands on the
    /// entrance of: `,` opens the store, the ware's letter opens its
    /// command menu, `p` picks *Buy*, `Enter` takes the default quantity
    /// and `Enter` again accepts the purchase, then `ESCAPE`s leave
    /// (`borg-store-buy.c:borg_think_shop_buy:1206-1244`).
    Buy { ware: usize },
    /// Give (with `birth_no_selling`, the game's default) or sell the pack
    /// item at this index: `,` opens the store, `s` starts the sale, the
    /// pack item's letter selects it, the quantity digits and `Enter` when
    /// more than one could be taken, `Enter` accepts the hand-over, then
    /// `ESCAPE`s leave (`borg-store-sell.c:borg_think_shop_sell:1101-1146`).
    Sell { item: usize, qty: u32 },
    /// Quaff the pack item at this index (`q`, `do_cmd_quaff_potion`).
    Quaff { item: usize },
    /// Fire the first quiver ammunition the launcher takes at the closest
    /// visible monster (`h`, `do_cmd_fire_at_nearest`,
    /// `player-attack.c:1412-1443`: no prompt; nothing happens without a
    /// launcher, ammunition or a target) (T5.3, #127).
    Shoot,
    /// Read the pack item at this index (`r`, `do_cmd_read_scroll`).
    Read { item: usize },
    /// Eat the pack item at this index (`E`, `do_cmd_eat_food`).
    Eat { item: usize },
    /// Wield the pack item at this index (`w`, `do_cmd_wield`).
    Wield { item: usize },
    /// Learn a spell from the book at this pack index (`G`,
    /// `cmd-obj.c:do_cmd_study:1245`). A `CHOOSE_SPELLS` class then picks
    /// the spell at `letter` (its position in the book's menu,
    /// `ui-spell.c:spell_menu_new:251`); the others learn a random one
    /// (`do_cmd_study_book:1200`) and `letter` is `None` (#127).
    Study { book: usize, letter: Option<u8> },
    /// Cast the spell at `letter` of the book at this pack index (`m`,
    /// `cmd-obj.c:do_cmd_cast:1113`). A spell that aims answers the
    /// direction prompt with `'`, the closest target
    /// (`ui-input.c:textui_get_aim_dir:1652-1658`, `target_set_closest`)
    /// (#127).
    Cast { book: usize, letter: u8, aim: bool },
}

/// The spell menu's selection letters (`ui-menu.c:41`,
/// `all_letters_nohjkl`), by the spell's position in its book.
pub const SPELL_LETTERS: &[u8] = b"abcdefgimnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

impl Action {
    /// The complete key sequence, from what the strategy decided and the
    /// observation it decided on (the pack's stack sizes and the letters
    /// are read from it, as the C Borg reads them from its item list).
    pub fn keys(self, o: &Observation) -> Vec<u32> {
        let digit = |d: u8| (b'0' + d) as u32;
        let mut k = vec![ESCAPE];
        match self {
            Action::Rest(n) => {
                k.push(b'R' as u32);
                k.extend(n.to_string().bytes().map(u32::from));
                k.push(KC_ENTER);
            }
            Action::Descend => k.push(b'>' as u32),
            Action::Ascend => k.push(b'<' as u32),
            Action::Walk(d) | Action::Attack(d) => k.push(digit(d)),
            Action::Get => k.push(b'g' as u32),
            Action::Hold => k.push(b',' as u32),
            Action::Alter(d) => k.extend([b'+' as u32, digit(d)]),
            Action::Buy { ware } => {
                // The store visit runs inside the ','; see the module docs.
                k.push(b',' as u32);
                k.push(u32::from(
                    SHOP_LETTERS[ware.checked_index(SHOP_LETTERS.len(), "store ware")],
                ));
                k.extend([b'p' as u32, KC_ENTER, KC_ENTER]);
                // Leave the store (borg-store-buy.c:1242-1244).
                k.extend([ESCAPE, ESCAPE, ESCAPE]);
            }
            Action::Quaff { item } => {
                k.push(b'q' as u32);
                k.push(u32::from(
                    PACK_LETTERS[item.checked_index(PACK_LETTERS.len(), "pack item")],
                ));
            }
            Action::Shoot => k.push(b'h' as u32),
            Action::Read { item } => {
                k.push(b'r' as u32);
                k.push(u32::from(
                    PACK_LETTERS[item.checked_index(PACK_LETTERS.len(), "pack item")],
                ));
            }
            Action::Eat { item } => {
                k.push(b'E' as u32);
                k.push(u32::from(
                    PACK_LETTERS[item.checked_index(PACK_LETTERS.len(), "pack item")],
                ));
            }
            Action::Wield { item } => {
                k.push(b'w' as u32);
                k.push(u32::from(
                    PACK_LETTERS[item.checked_index(PACK_LETTERS.len(), "pack item")],
                ));
            }
            Action::Study { book, letter } => {
                k.push(b'G' as u32);
                k.push(u32::from(
                    PACK_LETTERS[book.checked_index(PACK_LETTERS.len(), "pack item")],
                ));
                if let Some(l) = letter {
                    k.push(u32::from(l));
                }
            }
            Action::Cast { book, letter, aim } => {
                k.push(b'm' as u32);
                k.push(u32::from(
                    PACK_LETTERS[book.checked_index(PACK_LETTERS.len(), "pack item")],
                ));
                k.push(u32::from(letter));
                if aim {
                    k.push(b'\'' as u32);
                }
            }
            Action::Sell { item, qty } => {
                k.push(b',' as u32);
                k.push(b's' as u32);
                let i = item.checked_index(PACK_LETTERS.len(), "pack item");
                k.push(u32::from(PACK_LETTERS[i]));
                // The quantity prompt appears only when the stack could
                // give more than one; the digits then replace its default
                // of 1 (get_quantity, ui-input.c:1206-1245; the C Borg at
                // borg-store-sell.c:1107-1116).
                // A single item has no prompt, so digits would be read as
                // commands; it sells its one (#139).
                let stacked = o.inventory.get(i).is_some_and(|it| it.number > 1);
                if stacked {
                    if qty >= 2 {
                        k.extend(qty.to_string().bytes().map(u32::from));
                    }
                    k.push(KC_ENTER);
                }
                // Accept the hand-over (store_get_check, ui-store.c:566).
                k.push(KC_ENTER);
                // Leave the store (borg-store-sell.c:1122-1124, 1144-1146).
                k.extend([ESCAPE, ESCAPE, ESCAPE, ESCAPE, ESCAPE, ESCAPE]);
            }
        }
        k
    }

    /// What the next think expects to hold after the keys were consumed.
    /// `o` is the observation the decision was made on.
    pub fn effect(self, o: &Observation) -> Effect {
        let (dx, dy) = DIRS
            .iter()
            .find(|(d, _, _)| matches!(self, Action::Walk(x) | Action::Attack(x) if *d == x))
            .map(|(_, x, y)| (*x, *y))
            .unwrap_or((0, 0));
        match self {
            // A step has two honest ends: the position moved as intended,
            // or game time passed anyway (a closed door opened, an unseen
            // monster was attacked). A step that did neither was refused.
            Action::Walk(_) | Action::Attack(_) => Effect::Step {
                dx,
                dy,
                from: (o.status.x, o.status.y),
                from_turn: o.turn,
            },
            Action::Descend => Effect::Depth {
                to: o.status.depth + 1,
                from_turn: o.turn,
            },
            Action::Ascend => Effect::Depth {
                to: o.status.depth - 1,
                from_turn: o.turn,
            },
            // Rest, hold and alter charge energy on every path; a shot does
            // when it fires (without a launcher, ammunition or a target the
            // command prints why and takes none, which this catches).
            // A cast uses the turn whether it works or fails
            // (`player-spell.c:spell_cast:508-511`, `do_cmd_cast:1166-1170`),
            // and so does a study (`do_cmd_study_spell:1196`).
            Action::Rest(_)
            | Action::Hold
            | Action::Alter(_)
            | Action::Shoot
            | Action::Study { .. }
            | Action::Cast { .. } => Effect::Time { from_turn: o.turn },
            // A store visit costs no game time (do_cmd_hold: "turn will be
            // taken exiting the shop" sets energy 0, cmd-cave.c), so what
            // the next think can check is the pack. Item count as well as
            // slot count matters: buying or selling one of a stack changes
            // no slot (`pack_slots_used`), but it still happened.
            Action::Get | Action::Buy { .. } => Effect::PackGrows {
                used: o.status.pack_used,
                items: carried_items(o),
            },
            Action::Sell { .. } => Effect::PackShrinks {
                used: o.status.pack_used,
                items: carried_items(o),
            },
            Action::Quaff { item } | Action::Read { item } | Action::Eat { item } => {
                Effect::Consumed {
                    kind: o.inventory.get(item).map_or(0, |i| i.kind),
                    number: o.inventory.get(item).map_or(0, |i| i.number),
                }
            }
            Action::Wield { item } => Effect::Wielded {
                kind: o.inventory.get(item).map_or(0, |i| i.kind),
                from_turn: o.turn,
            },
        }
    }
}

/// Directions 1–9 (keypad layout, 5 = stay) as (dir, dx, dy). The eight
/// walkable ones; the executor has no use for 5.
const DIRS: [(u8, i32, i32); 8] = [
    (1, -1, 1),
    (2, 0, 1),
    (3, 1, 1),
    (4, -1, 0),
    (6, 1, 0),
    (7, -1, -1),
    (8, 0, -1),
    (9, 1, -1),
];

/// A checked index into one of the letter lists: an index past the letters
/// is a strategy bug, and a run fails loudly rather than pressing a wrong
/// key (RULES 13).
trait CheckedIndex {
    fn checked_index(self, len: usize, what: &str) -> usize;
}

impl CheckedIndex for usize {
    fn checked_index(self, len: usize, what: &str) -> usize {
        assert!(
            self < len,
            "{what} index {self} is past the letters ({len})"
        );
        self
    }
}

/// Items carried, in units rather than pack slots: a second scroll bought
/// into an existing stack and one sold from it are both observable here.
fn carried_items(o: &Observation) -> i32 {
    o.inventory
        .iter()
        .chain(&o.quiver)
        .map(|i| i32::from(i.number))
        .sum::<i32>()
        + o.equipment
            .iter()
            .filter_map(|s| s.item.as_ref())
            .map(|i| i32::from(i.number))
            .sum::<i32>()
}

/// What the next think checks after an action's keys were consumed
/// (ARCHITECTURE.md L6: "did the potion get drunk?"). Each effect is
/// checked against the observation of the think after the action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// The position moved by `(dx, dy)` from `from`, or game time passed.
    Step {
        dx: i32,
        dy: i32,
        from: (i32, i32),
        from_turn: i64,
    },
    /// Game time passed.
    Time { from_turn: i64 },
    /// The depth became `to`.
    Depth { to: i32, from_turn: i64 },
    /// The pack grew: another slot, or more items in an existing stack.
    PackGrows { used: i32, items: i32 },
    /// The pack shrank: a slot freed, or fewer items in an existing stack.
    PackShrinks { used: i32, items: i32 },
    /// A consumable was used: its kind's count fell, or it is gone.
    Consumed { kind: u32, number: u8 },
    /// The item of this kind is now in an equipment slot.
    Wielded { kind: u32, from_turn: i64 },
}

impl Effect {
    /// Check against the next think's observation. `Ok(())` when the
    /// effect held; the string says what did not, for the trace.
    pub fn check(&self, o: &Observation) -> Result<(), String> {
        let s = &o.status;
        match *self {
            Effect::Step {
                dx,
                dy,
                from,
                from_turn,
            } => {
                let moved = (s.x, s.y) == (from.0 + dx, from.1 + dy);
                if moved || o.turn > from_turn {
                    Ok(())
                } else {
                    Err(format!(
                        "the step {dx},{dy} neither moved the character nor took game time: the game refused it"
                    ))
                }
            }
            Effect::Time { from_turn } => {
                if o.turn > from_turn {
                    Ok(())
                } else {
                    Err("the action took no game time: the game refused it".into())
                }
            }
            Effect::Depth { to, from_turn } => {
                if s.depth == to {
                    Ok(())
                } else if o.turn == from_turn {
                    Err(format!(
                        "the staircase took no game time; the depth is still {}",
                        s.depth
                    ))
                } else {
                    Err(format!(
                        "the depth is {}, not the {} the staircase promised",
                        s.depth, to
                    ))
                }
            }
            Effect::PackGrows { used, items } => {
                if s.pack_used > used || carried_items(o) > items {
                    Ok(())
                } else {
                    Err(format!(
                        "the pack is still {used} slots and {items} items; nothing was picked up or bought"
                    ))
                }
            }
            Effect::Consumed { kind, number } => {
                let left = o
                    .inventory
                    .iter()
                    .find(|i| i.kind == kind)
                    .map_or(0, |i| i.number);
                if left < number {
                    Ok(())
                } else {
                    Err(format!(
                        "the consumable kind {kind} is still {left} (was {number}): nothing was used"
                    ))
                }
            }
            Effect::Wielded { kind, from_turn } => {
                if o.equipment
                    .iter()
                    .filter_map(|s| s.item.as_ref())
                    .any(|i| i.kind == kind)
                {
                    Ok(())
                } else if o.turn == from_turn {
                    Err(format!(
                        "the item kind {kind} was not wielded: no game time passed"
                    ))
                } else {
                    Err(format!("the item kind {kind} is not in an equipment slot"))
                }
            }
            Effect::PackShrinks { used, items } => {
                if s.pack_used < used || carried_items(o) < items {
                    Ok(())
                } else {
                    Err(format!(
                        "the pack is still {used} slots and {items} items; nothing was handed over"
                    ))
                }
            }
        }
    }
}

/// The answer to a prompt the catcher recognised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prompt {
    /// The key that answers it.
    pub key: u32,
    /// What the trace records.
    pub why: &'static str,
}

/// The prompt catcher: the C Borg's `borg_react_prompted` and its `-more-`
/// test (`borg-messages-react.c:borg_react_prompted:95-324`;
/// `borg-messages.c:borg_get_messages:183-294`), lifted with the answers
/// unchanged. Excluded: `"Die?"` (borg-messages-react.c:105-140), which
/// only exists to cheat death (GOALS D5).
///
/// * `line`, `attr_dark` and `cursor` are the message line as
///   `borg_what_text` reads it and the terminal cursor (`borg.c:279`);
/// * `wants_command` is `inkey_flag` (ui-input.c:54): the game asking for
///   a command means the line holds a message, not a prompt;
/// * `queue_empty` mirrors the C Borg's `!borg_inkey(false)` guards.
///
/// `None` lets the key queue answer (a prompt the action's own keys
/// pre-answer, or one no rule knows).
pub fn answer(
    line: &str,
    attr_dark: bool,
    cursor: (i32, i32),
    wants_command: bool,
    queue_empty: bool,
) -> Option<Prompt> {
    let p = |key: u32, why: &'static str| Some(Prompt { key, why });
    // "If the first four characters on the message line all have the same
    // attribute (or are all spaces), and they are not all spaces" —
    // borg-messages.c:206-210, minus the C Borg's truncation of the text
    // at the first attribute change, which our read of the whole row
    // makes unnecessary.
    let first_cells_text = line.chars().take(4).any(|c| c != ' ');
    let is_prompt = !attr_dark && first_cells_text;
    if !is_prompt || line.is_empty() {
        return None;
    }
    // "check for strings starting "Type" because those are also prompts"
    // (they are not: borg-messages.c:212-215).
    if line.starts_with("Type") {
        return None;
    }
    if !wants_command {
        // The prompts that need an answer now, in the C Borg's order
        // (borg-messages-react.c:95-324).
        if line.starts_with("Atte") {
            // "Attempt it anyway? [y/n]" when casting short of mana.
            return p(b'y' as u32, "cast anyway: y");
        }
        if line.starts_with("(Equip: c-d,") {
            // Two rings: the left hand.
            return p(b'c' as u32, "second ring on the left hand: c");
        }
        if queue_empty && line.starts_with("Direction") {
            // An unexpected request for a direction: the C Borg aborts
            // (borg_oops, borg-messages-react.c:169-187). We answer ESCAPE
            // and let the trace carry the defect; the game stays up.
            return p(ESCAPE, "unexpected direction prompt: ESCAPE");
        }
        if line.starts_with("You have no room") {
            // A key of code 0 is a no-op (borg-messages-react.c:192-198).
            return p(0, "no room: no-op key");
        }
        if queue_empty && line.starts_with("Choose a monster race") {
            // Banishment picked by a random effect: refuse it.
            return p(ESCAPE, "banish choice: ESCAPE");
        }
        if line.starts_with("The lava will") || line.starts_with("Lava blocks y") {
            return p(b'y' as u32, "lava warning: y");
        }
        if queue_empty && line.starts_with("Set recall depth") {
            return p(b'n' as u32, "recall depth unchanged: n");
        }
        if line.starts_with("Target out of range") {
            return p(b'y' as u32, "fire anyway: y");
        }
        if line.starts_with("Word of Recall is already") {
            return p(b'n' as u32, "recall already set: n");
        }
        // Sub-screens the keys stumbled into: leave them. The C Borg
        // compares the raw line with its trailing space
        // (borg-messages-react.c:256-287); our read trims trailing blanks,
        // so the trimmed form is the faithful one, and "Wear o" is a
        // prefix rather than the C Borg's exact string, covering the wear
        // prompt it was written for.
        // The wear prompt is a stumble only when nothing is queued: our own
        // Wield opens it with the item letter waiting behind (#126: every
        // Wield was escaped here and its letter fell on the map as a
        // pickup, so no wield ever happened in a game).
        if line == "(Equipment)"
            || line == "(Inventory)"
            || (queue_empty && line.starts_with("Wear o"))
            || line == "Lv Mana Fail"
        {
            return p(ESCAPE, "sub-screen: ESCAPE");
        }
        if line.starts_with("Throw which item? (Throw") {
            // Switch the item list to the pack.
            return p(b'/' as u32, "throw list to inventory: /");
        }
        if cursor.1 == 1 && queue_empty && line.starts_with("Which effect?") {
            return p(b'a' as u32, "random effect: a");
        }
    }
    // "-more-" (borg-messages.c:223-247): the cursor sits on the top line
    // at or past column 7, and the text before it ends with " -more-".
    // With `auto_more` on this never appears for us (docs/ENGINE.md); the
    // answer is kept for the day the option changes.
    if !wants_command && cursor.1 == 0 && cursor.0 >= 7 && line.ends_with(" -more-") {
        return p(b' ' as u32, "-more-: space");
    }
    // A normal message while the game wants a command: the C Borg clears
    // it with a space (borg-messages.c:288-290). We do not: our option
    // `auto_more` keeps the game moving without it, and pressing it would
    // change every recorded key stream (declared in docs/ENGINE.md).
    None
}

/// The executor's state: the queued keys, the pending effect and the
/// prompts answered since the last think.
#[derive(Debug, Default)]
pub struct Executor {
    queue: std::collections::VecDeque<u32>,
    pending: Option<(Action, Effect)>,
    /// The prompts answered since the last think, for the trace.
    answered: Vec<String>,
}

impl Executor {
    /// Queue an action's keys and remember its effect for the next think.
    pub fn queue(&mut self, action: Action, o: &Observation) {
        self.pending = Some((action, action.effect(o)));
        self.queue.extend(action.keys(o));
    }

    /// The next key, if one is queued.
    pub fn pop(&mut self) -> Option<u32> {
        self.queue.pop_front()
    }

    /// Everything the queue still holds is dropped (a flush).
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Whether no keys remain queued.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Record a prompt the catcher answered, for the trace.
    pub fn answered(&mut self, why: &str) {
        self.answered.push(why.to_string());
    }

    /// Take the prompts answered since the last think (the trace line
    /// takes them).
    pub fn take_answered(&mut self) -> Vec<String> {
        std::mem::take(&mut self.answered)
    }

    /// Check the pending action's effect against this think's
    /// observation, and clear it: each action is checked exactly once, by
    /// the next think. A flushed queue leaves the action half-sent, which
    /// the effect check then reports.
    pub fn check_pending(&mut self, o: &Observation) -> Vec<String> {
        match self.pending.take() {
            Some((action, effect)) => match effect.check(o) {
                Ok(()) => vec![],
                Err(why) => vec![format!("after {action:?}: {why}")],
            },
            None => vec![],
        }
    }

    /// Drop the pending effect without checking it: no observation to
    /// check against. The trace notes what went unwatched.
    pub fn drop_pending(&mut self) -> Option<String> {
        self.pending
            .take()
            .map(|(action, _)| format!("after {action:?}: no observation to check the effect"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{Item, Status};

    /// An observation to compile keys against: an empty one, or one whose
    /// pack holds `stacks` items of `number` each.
    fn obs(packs: &[(u32, u8)]) -> Observation {
        let mut o = Observation::default();
        for (kidx, number) in packs {
            o.inventory.push(Item {
                kind: *kidx,
                tval: 70,
                sval: 0,
                number: *number,
                weight: 1,
                timeout: 0,
                level: 1,
                desc: String::new(),
                aware: true,
                fully_known: true,
                runes_known: false,
                flags: [0; 5],
                ac: 0,
                dd: 0,
                ds: 0,
                to_h: 0,
                to_d: 0,
                to_a: 0,
                modifiers: vec![],
                elements: vec![],
            });
        }
        o
    }

    /// An observation after an action: the fields the effects check.
    fn after(turn: i64, x: i32, y: i32, depth: i32, pack_used: i32) -> Observation {
        let mut o = obs(&[]);
        o.turn = turn;
        o.status = Status {
            x,
            y,
            depth,
            pack_used,
            ..Status::default()
        };
        o
    }

    #[test]
    fn dungeon_actions_open_with_escape() {
        let o = obs(&[]);
        assert_eq!(Action::Descend.keys(&o), vec![ESCAPE, b'>' as u32]);
        assert_eq!(Action::Ascend.keys(&o), vec![ESCAPE, b'<' as u32]);
        assert_eq!(Action::Walk(3).keys(&o), vec![ESCAPE, b'3' as u32]);
        assert_eq!(Action::Attack(7).keys(&o), vec![ESCAPE, b'7' as u32]);
        assert_eq!(Action::Get.keys(&o), vec![ESCAPE, b'g' as u32]);
        assert_eq!(Action::Hold.keys(&o), vec![ESCAPE, b',' as u32]);
        assert_eq!(
            Action::Alter(2).keys(&o),
            vec![ESCAPE, b'+' as u32, b'2' as u32]
        );
        // The C Borg's rest: 'R' '7' '5' Enter.
        assert_eq!(
            Action::Rest(75).keys(&o),
            vec![ESCAPE, b'R' as u32, b'7' as u32, b'5' as u32, KC_ENTER]
        );
    }

    /// The C Borg's purchase (borg-store-buy.c:1206-1211, 1242-1244): the
    /// ware's letter opens its command menu, `p` picks Buy, Enter takes
    /// the quantity default, Enter accepts, ESCAPEs leave.
    #[test]
    fn buy_queues_the_c_borgs_purchase() {
        let o = obs(&[]);
        assert_eq!(
            Action::Buy { ware: 0 }.keys(&o),
            vec![
                ESCAPE,
                b',' as u32,
                SHOP_LETTERS[0] as u32,
                b'p' as u32,
                KC_ENTER,
                KC_ENTER,
                ESCAPE,
                ESCAPE,
                ESCAPE,
            ]
        );
        assert_eq!(SHOP_LETTERS[0], b'a');
        assert_eq!(SHOP_LETTERS[1], b'c', "h j k l are not store letters");
        assert_eq!(
            SHOP_LETTERS, b"acfhjmnoqruvyzABDFGHJKLMNOPQRSTUVWXYZ",
            "the original-keyset store selections (ui-store.c:805)"
        );
    }

    /// The C Borg's sale (borg-store-sell.c:1101-1146): 's', the pack
    /// letter, the quantity when more than one could go, Enter to accept,
    /// ESCAPEs to leave. `birth_no_selling` is on by default, so the
    /// accept says "Give" (ui-store.c:500) and no gold moves.
    #[test]
    fn sell_queues_the_c_borgs_sale() {
        // A single, unstacked item: no quantity prompt (ui-input.c:1211).
        assert_eq!(
            Action::Sell { item: 0, qty: 1 }.keys(&obs(&[(1, 1)])),
            vec![
                ESCAPE,
                b',' as u32,
                b's' as u32,
                PACK_LETTERS[0] as u32,
                KC_ENTER,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
            ]
        );
        // A stack of 4, one given: the prompt shows, Enter takes its
        // default of one.
        assert_eq!(
            Action::Sell { item: 1, qty: 1 }.keys(&obs(&[(1, 1), (2, 4)])),
            vec![
                ESCAPE,
                b',' as u32,
                b's' as u32,
                PACK_LETTERS[1] as u32,
                KC_ENTER,
                KC_ENTER,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
            ]
        );
        // A single item asked for twice over: no quantity prompt appears,
        // so no digits and no extra Enter (#139: digits there would be
        // read as commands).
        assert_eq!(
            Action::Sell { item: 0, qty: 2 }.keys(&obs(&[(1, 1)])),
            Action::Sell { item: 0, qty: 1 }.keys(&obs(&[(1, 1)])),
        );
        // A stack of 4, three given: the digits replace the default.
        assert_eq!(
            Action::Sell { item: 0, qty: 3 }.keys(&obs(&[(2, 4)])),
            vec![
                ESCAPE,
                b',' as u32,
                b's' as u32,
                PACK_LETTERS[0] as u32,
                b'3' as u32,
                KC_ENTER,
                KC_ENTER,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
                ESCAPE,
            ]
        );
        assert_eq!(PACK_LETTERS[7], b'i', "h j k l are skipped in the pack");
        assert_eq!(
            PACK_LETTERS.len(),
            48,
            "26 lower minus h j k l, plus 26 upper"
        );
    }

    #[test]
    fn item_actions_use_the_pack_letter_and_check_consumption() {
        let o = obs(&[(231, 2), (11, 1)]);
        assert_eq!(
            Action::Quaff { item: 0 }.keys(&o),
            vec![ESCAPE, b'q' as u32, u32::from(PACK_LETTERS[0])]
        );
        assert_eq!(
            Action::Read { item: 1 }.keys(&o),
            vec![ESCAPE, b'r' as u32, u32::from(PACK_LETTERS[1])]
        );
        assert_eq!(
            Action::Eat { item: 1 }.keys(&o),
            vec![ESCAPE, b'E' as u32, u32::from(PACK_LETTERS[1])]
        );
        assert_eq!(
            Action::Wield { item: 1 }.keys(&o),
            vec![ESCAPE, b'w' as u32, u32::from(PACK_LETTERS[1])]
        );
        // Study: `G`, the book, the spell's menu letter for a chooser.
        assert_eq!(
            Action::Study {
                book: 1,
                letter: Some(b'b')
            }
            .keys(&o),
            vec![ESCAPE, b'G' as u32, u32::from(PACK_LETTERS[1]), b'b' as u32]
        );
        assert_eq!(
            Action::Study {
                book: 1,
                letter: None
            }
            .keys(&o),
            vec![ESCAPE, b'G' as u32, u32::from(PACK_LETTERS[1])]
        );
        // Cast: `m`, the book, the spell, `'` for the closest target.
        assert_eq!(
            Action::Cast {
                book: 0,
                letter: b'a',
                aim: true
            }
            .keys(&o),
            vec![
                ESCAPE,
                b'm' as u32,
                u32::from(PACK_LETTERS[0]),
                b'a' as u32,
                b'\'' as u32
            ]
        );
        let mut next = after(101, 0, 0, 1, 2);
        next.inventory = obs(&[(231, 1)]).inventory;
        assert!(Effect::Consumed {
            kind: 231,
            number: 2
        }
        .check(&next)
        .is_ok());
        let mut same = after(101, 0, 0, 1, 2);
        same.inventory = obs(&[(231, 2)]).inventory;
        assert!(Effect::Consumed {
            kind: 231,
            number: 2
        }
        .check(&same)
        .is_err());
        let mut held = after(101, 0, 0, 1, 2);
        held.equipment = vec![crate::observation::Slot {
            slot_type: 1,
            item: Some(obs(&[(7, 1)]).inventory.remove(0)),
        }];
        assert!(Effect::Wielded {
            kind: 7,
            from_turn: 100
        }
        .check(&held)
        .is_ok());
    }

    #[test]
    #[should_panic(expected = "past the letters")]
    fn an_index_past_the_letters_fails_loudly() {
        Action::Buy { ware: 99 }.keys(&obs(&[]));
    }

    /// A prompt is text with a lit attribute while the game does not want
    /// a command (borg-messages.c:206-218).
    #[test]
    fn prompt_answers_are_the_c_borgs() {
        let prompt = |line: &str| answer(line, false, (10, 0), false, false);
        assert_eq!(
            prompt("Attempt it anyway? [y/n]"),
            Some(Prompt {
                key: b'y' as u32,
                why: "cast anyway: y"
            })
        );
        assert_eq!(
            prompt("(Equip: c-d, (right hand))"),
            Some(Prompt {
                key: b'c' as u32,
                why: "second ring on the left hand: c"
            })
        );
        assert_eq!(
            prompt("You have no room to carry it."),
            Some(Prompt {
                key: 0,
                why: "no room: no-op key"
            })
        );
        assert_eq!(
            prompt("The lava will burn you! Proceed? "),
            Some(Prompt {
                key: b'y' as u32,
                why: "lava warning: y"
            })
        );
        assert_eq!(
            prompt("Lava blocks your way.  Use it? "),
            Some(Prompt {
                key: b'y' as u32,
                why: "lava warning: y"
            })
        );
        assert_eq!(
            prompt("Target out of range.  Fire anyway? [Y/N]"),
            Some(Prompt {
                key: b'y' as u32,
                why: "fire anyway: y"
            })
        );
        assert_eq!(
            prompt("Word of Recall is already active."),
            Some(Prompt {
                key: b'n' as u32,
                why: "recall already set: n"
            })
        );
        assert_eq!(
            prompt("(Inventory)"),
            Some(Prompt {
                key: ESCAPE,
                why: "sub-screen: ESCAPE"
            })
        );
        assert_eq!(
            prompt("(Equipment)"),
            Some(Prompt {
                key: ESCAPE,
                why: "sub-screen: ESCAPE"
            })
        );
        // The wear prompt is escaped only as a stumble (nothing queued);
        // our own Wield's letter waits behind it (#126).
        assert_eq!(
            answer("Wear or wield which item?", false, (10, 0), false, true),
            Some(Prompt {
                key: ESCAPE,
                why: "sub-screen: ESCAPE"
            })
        );
        assert_eq!(prompt("Wear or wield which item? (Inven: f-g, ESC)"), None);
        assert_eq!(
            prompt("Throw which item? (Throw a dagger)"),
            Some(Prompt {
                key: b'/' as u32,
                why: "throw list to inventory: /"
            })
        );
        // Only with the queue empty, as the C Borg guards some answers.
        assert_eq!(
            answer("Set recall depth", false, (5, 0), false, true),
            Some(Prompt {
                key: b'n' as u32,
                why: "recall depth unchanged: n"
            })
        );
        assert_eq!(
            answer("Set recall depth", false, (5, 0), false, false),
            None,
            "queued keys answer it"
        );
        assert_eq!(
            answer(
                "Direction (* for target, ESC to target closest)",
                false,
                (5, 0),
                false,
                true
            ),
            Some(Prompt {
                key: ESCAPE,
                why: "unexpected direction prompt: ESCAPE"
            })
        );
        // -more-: the cursor past column 7 on row 0, the text ending in it.
        assert_eq!(
            answer("You feel nervous. -more-", false, (23, 0), false, true),
            Some(Prompt {
                key: b' ' as u32,
                why: "-more-: space"
            })
        );
    }

    #[test]
    fn messages_and_blanks_are_not_prompts() {
        // The game wants a command: the line is a message, not a prompt.
        assert_eq!(answer("You feel nervous.", false, (5, 0), true, true), None);
        // Dark attribute: nothing drawn.
        assert_eq!(answer("You feel nervous.", true, (5, 0), false, true), None);
        // Blank start: no prompt (borg-messages.c:207).
        assert_eq!(answer("    Direction?", false, (5, 0), false, true), None);
        // "Type ..." lines are not prompts (borg-messages.c:212-215).
        assert_eq!(
            answer("Type '?' for help.", false, (5, 0), false, true),
            None
        );
        // An unknown prompt falls through to the queue.
        assert_eq!(
            answer(
                "Rest for how long? (& to rest as needed)",
                false,
                (5, 0),
                false,
                false
            ),
            None
        );
        // "Die?" is excluded: cheat death is not ours (GOALS D5).
        assert_eq!(answer("Die? [y/n]", false, (5, 0), false, true), None);
    }

    #[test]
    fn effects_hold_and_fail() {
        // A step that moved.
        let step = Action::Walk(6).effect(&after(100, 10, 10, 1, 3));
        assert_eq!(
            step,
            Effect::Step {
                dx: 1,
                dy: 0,
                from: (10, 10),
                from_turn: 100
            }
        );
        assert!(step.check(&after(101, 11, 10, 1, 3)).is_ok());
        // A step that took time without moving: a door opened.
        assert!(step.check(&after(102, 10, 10, 1, 3)).is_ok());
        // A refused step: no move, no time.
        assert!(step.check(&after(100, 10, 10, 1, 3)).is_err());
        // A staircase that did not fire.
        let down = Action::Descend.effect(&after(100, 10, 10, 3, 3));
        assert!(down.check(&after(104, 10, 10, 4, 3)).is_ok());
        assert!(down.check(&after(100, 10, 10, 3, 3)).is_err());
        assert!(down.check(&after(104, 10, 10, 3, 3)).is_err());
        // The pack.
        let get = Action::Get.effect(&after(100, 10, 10, 1, 3));
        assert!(get.check(&after(101, 10, 10, 1, 4)).is_ok());
        assert!(get.check(&after(101, 10, 10, 1, 3)).is_err());
        let sell = Action::Sell { item: 0, qty: 1 }.effect(&after(100, 10, 10, 0, 3));
        assert!(sell.check(&after(100, 10, 10, 0, 2)).is_ok());
        assert!(sell.check(&after(100, 10, 10, 0, 3)).is_err());
    }

    #[test]
    fn stacked_purchases_and_sales_change_the_item_count_not_the_slots() {
        let mut before = obs(&[(185, 1)]);
        before.status.pack_used = 1;
        let buy = Action::Buy { ware: 0 }.effect(&before);
        let mut after_buy = obs(&[(185, 2)]);
        after_buy.status.pack_used = 1;
        assert!(
            buy.check(&after_buy).is_ok(),
            "a second scroll joined the stack"
        );
        assert!(buy.check(&before).is_err());

        let mut before = obs(&[(190, 2)]);
        before.status.pack_used = 1;
        let sell = Action::Sell { item: 0, qty: 1 }.effect(&before);
        let mut after_sale = obs(&[(190, 1)]);
        after_sale.status.pack_used = 1;
        assert!(
            sell.check(&after_sale).is_ok(),
            "one of two was handed over"
        );
        assert!(sell.check(&before).is_err());
    }

    #[test]
    fn the_executor_checks_once_and_reports_the_action() {
        let mut x = Executor::default();
        let o = after(100, 10, 10, 1, 3);
        x.queue(Action::Descend, &o);
        assert_eq!(x.pop(), Some(ESCAPE));
        assert_eq!(x.pop(), Some(b'>' as u32));
        assert!(x.is_empty());
        // Depth did not change and no time passed: one violation.
        assert_eq!(
            x.check_pending(&after(100, 10, 10, 1, 3)),
            vec![
                "after Descend: the staircase took no game time; the depth is still 1".to_string()
            ]
        );
        // Checked exactly once.
        assert!(x.check_pending(&after(100, 10, 10, 1, 3)).is_empty());
        // Prompts answered between thinks are taken by the trace.
        x.answered("-more-: space");
        assert_eq!(x.take_answered(), vec!["-more-: space".to_string()]);
        assert!(x.take_answered().is_empty());
    }
}
