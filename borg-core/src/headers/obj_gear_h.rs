//! Read-only native carrying queries.
extern "C" {
    pub fn inven_quiver_stack_room(p: *const super::player_h::player, obj: *const super::object_h::object) -> i32;
    pub fn inven_merge_num(p: *const super::player_h::player, obj: *const super::object_h::object) -> i32;
    pub fn inven_carry_num(p: *const super::player_h::player, obj: *const super::object_h::object) -> i32;
}
