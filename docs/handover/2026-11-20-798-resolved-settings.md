# #798 Resolved settings at birth

The borg-rs core writes `settings-resolved.json` when it attaches to the
character, after selecting the class. The receipt includes the class and
the non-default resolved parameter values, including the breeder policy's
derived values. The live lookup and receipt share class precedence and
breeder policy helpers.
`params-core.json` remains the full registry audit.

The unit test exercises Ranger, Mage and Warrior settings, including a
class override back to the compiled default. The pinned zoo adapter
contract verifies the receipt of a real Warrior cell.

The adapter work is tracked in [borg-zoo#539](https://github.com/diegoami/borg-zoo/issues/539).
Its acceptance test needs to preserve the canonical receipt and its hash
and match the written settings using the played class and alias rules.
An empty receipt can follow an explicit setting equal to a compiled class
default. An absent receipt leaves the resolved values unavailable.

The GLM-5.3-Flash review at `1575977187b5c25503164b43e341bd0c6408d319`
reproduced smoke, the release test and the pinned adapter contract. It
confirmed that the first implementation caused the linker failure and
that `15759771` fixed it. The review follow-up clarified receipt names and
explicit defaults in the interface and the Zoo request. The owner decides
when to merge.
