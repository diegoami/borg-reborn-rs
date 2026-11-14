//! Engine-free entry points for the regression scripts in
//! `harness/regress/` (the crate's only public API besides the C-ABI
//! symbols the game links against).

use crate::borg_item::{blank_item, item_note, set_item_note, NoteOrigin};
use crate::borg_item_id::borg_item_note_needs_id;
use crate::borg_store::BorgShop;
use crate::state::BorgState;

#[derive(Clone, Copy)]
enum At {
    Item(usize),
    Ware(usize),
}

fn slot(st: &mut BorgState, at: At) -> &mut crate::borg_item::BorgItem {
    match at {
        At::Item(i) => &mut st.items.items[i],
        At::Ware(j) => &mut st.store.shops[0].ware[j],
    }
}

/// `put()` of the C side: the item is analyzed in place, so its note
/// points into its own live element.
fn put(st: &mut BorgState, at: At, desc: &str, note_off: usize, ident: bool, needs_ident: bool) {
    let it = slot(st, at);
    *it = blank_item();
    it.iqty = 1;
    it.aware = true;
    it.ident = ident;
    it.needs_ident = needs_ident;
    for (d, c) in it.desc.iter_mut().zip(desc.bytes()) {
        *d = c as core::ffi::c_char;
    }
    let origin = match at {
        At::Item(i) => NoteOrigin::Item(i as u16),
        At::Ware(j) => NoteOrigin::Ware { shop: 0, slot: j as u8 },
    };
    set_item_note(it, origin, note_off);
}

fn show(out: &mut String, st: &BorgState, tag: &str, it: &crate::borg_item::BorgItem) {
    let note = if it.note.is_some() {
        String::from_utf8(item_note(st, it)).expect("ASCII")
    } else {
        "(null)".to_string()
    };
    out.push_str(&format!("{tag} note=[{note}] needs_id={}\n", u8::from(borg_item_note_needs_id(st, it))));
}

/// The lines `harness/regress/note-alias/note_alias.c` prints, computed by
/// the Rust port (pure: no engine call).
pub fn note_alias_report() -> String {
    let mut out = String::new();
    let mut st = BorgState::default();
    st.items.items = vec![blank_item(); 4];
    st.items.safe_items = vec![blank_item(); 4];
    st.store.shops = vec![BorgShop { ware: vec![blank_item(); 2] }];
    st.store.safe_shops = vec![BorgShop { ware: vec![blank_item(); 2] }];

    // S1: wear trial
    put(&mut st, At::Item(0), "Old", 3, false, false);
    put(&mut st, At::Item(1), "New longer {??}", 11, false, false);
    st.items.items[2] = blank_item();
    st.items.safe_items = st.items.items.clone();
    st.items.items[2] = st.items.safe_items[0];
    st.items.items[0] = st.items.safe_items[1];
    show(&mut out, &st, "S1 slot", &st.items.items[0]);
    show(&mut out, &st, "S1 cand", &st.items.items[1]);
    show(&mut out, &st, "S1 hole", &st.items.items[2]);
    st.items.items[0] = st.items.safe_items[0];
    st.items.items[2] = st.items.safe_items[2];
    show(&mut out, &st, "S1 restored slot", &st.items.items[0]);

    // S2: long old description, short candidate
    put(&mut st, At::Item(0), "Long old desc {cursed}", 14, false, false);
    put(&mut st, At::Item(1), "Short", 5, false, false);
    st.items.items[2] = blank_item();
    st.items.safe_items = st.items.items.clone();
    st.items.items[2] = st.items.safe_items[0];
    st.items.items[0] = st.items.safe_items[1];
    show(&mut out, &st, "S2 hole", &st.items.items[2]);

    // S3: a ware copied into the pack, then the live ware changes
    put(&mut st, At::Ware(0), "A ware {??}", 7, false, false);
    put(&mut st, At::Ware(1), "Second", 6, false, true);
    st.store.safe_shops[0].ware = st.store.shops[0].ware.clone();
    st.items.items[3] = st.store.shops[0].ware[0];
    show(&mut out, &st, "S3 pack copy", &st.items.items[3]);
    put(&mut st, At::Ware(0), "Changed {magical}", 8, true, false);
    show(&mut out, &st, "S3 pack copy after ware change", &st.items.items[3]);
    show(&mut out, &st, "S3 safe ware", &st.store.safe_shops[0].ware[0]);

    // S4: unchanged origin
    put(&mut st, At::Item(0), "Plain {??}", 6, false, false);
    st.items.items[1] = st.items.items[0];
    show(&mut out, &st, "S4 copy", &st.items.items[1]);

    // S5: NULL note, needs_ident short cut
    st.items.items[0] = blank_item();
    show(&mut out, &st, "S5 null", &st.items.items[0]);
    put(&mut st, At::Item(0), "x", 1, false, true);
    show(&mut out, &st, "S5 needs_ident", &st.items.items[0]);
    out
}
