# #798 Resolved settings at birth

The borg-rs core writes `settings-resolved.json` when it attaches to the
character, after selecting the class. The receipt includes the class and
the non-default resolved parameter values, including the breeder policy's
derived values. The live lookup and receipt share class precedence and breeder policy helpers.
`params-core.json` remains the full registry audit.

The unit test exercises Ranger, Mage and Warrior settings, including a
class override back to the compiled default. The pinned zoo adapter
contract verifies the receipt of a real Warrior cell.

The zoo adapter change is tracked in [borg-zoo#539](https://github.com/diegoami/borg-zoo/issues/539). Its acceptance test needs
to preserve the receipt, hash it and populate the cell's effective
settings. Historical cells without a receipt remain distinguishable from
characters with no non-default settings. No population run is required.
