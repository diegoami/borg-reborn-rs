//! Stable town-record trait IDs. M12 IDs 0..264 remain unchanged; M13's
//! stun protection and four vulnerabilities append at 265..269. The engine
//! enum is independent: M13 inserted these fields inside its native vector.

/// Stable serialized ID consumed by the native Python coverage reader.
#[cfg_attr(not(test), expect(dead_code))]
pub const AMMO_TVAL: usize = 152;

pub fn record_index(native: usize) -> usize {
    match native {
        85 => 265,
        193..=196 => 266 + native - 193,
        _ => native - usize::from(native > 85) - 4 * usize::from(native > 196),
    }
}

pub fn engine_index(record: usize) -> Option<usize> {
    match record {
        0..=264 => Some(record + usize::from(record >= 85) + 4 * usize::from(record >= 192)),
        265 => Some(85),
        266..=269 => Some(193 + record - 266),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn m13_projection_is_a_bijection_and_keeps_m12_ids() {
        assert_eq!(BI_MAX,270,"new engine traits require an explicit serialization mapping");
        let mut ids=std::collections::BTreeSet::new();
        for native in 0..BI_MAX as usize {
            let record=record_index(native);assert!(ids.insert(record));
            assert_eq!(engine_index(record),Some(native));
        }
        use crate::borg_headers::borg_trait_h::*;
        assert_eq!(record_index(BI_AMMO_TVAL as usize),AMMO_TVAL);
        assert_eq!(record_index(BI_MAXDEPTH as usize),106);
        assert_eq!(record_index(BI_ACLW as usize),233);
        assert_eq!(record_index(BI_CARRY as usize),261);
        assert_eq!(record_index(BI_PSTUN as usize),265);
        assert_eq!(engine_index(270),None);
    }
}
