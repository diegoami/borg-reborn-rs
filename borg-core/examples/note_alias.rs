//! Differential check of `borg_item.note` aliasing (review of 4c06cec08,
//! finding 1): the Rust side of `harness/regress/note-alias/note_alias.c`.
//! The scenarios are in `borg_rs::regress`; `run.sh` there compares the two
//! outputs byte for byte.

fn main() {
    print!("{}", borg_rs::regress::note_alias_report());
}
