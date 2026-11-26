# #795 Preparedness trace

The existing dive and trip checks publish their first failed requirement
and operands to the core trace. The next depth observation links an outcome
to the checked think. The reporter counts one first adjacent descent refusal
per cell, trip and proposed depth. Historical logs can be read explicitly
in legacy mode.

The first historical table is on [#795](https://github.com/diegoami/borg-reborn-rs/issues/795#issuecomment-7052816300).
It uses the 18 shipped-control cells of Zoo run 44817311691. Four of six
Paladin trips that reached DL2 recorded an explicit 50-max-HP refusal for
DL3. No cell reached DL4. These are partial observations from two Paladin
births. The 90-start follow-up lacks raw streams.

The [proposed Zoo test](https://github.com/diegoami/borg-zoo/issues/544) changes only the Paladin DL3–4 max-HP minimum, from
50 to 45, with the P7 descent endpoint and a prespecified survival margin.
The owner chooses whether to run it and opens the confirmation block.
