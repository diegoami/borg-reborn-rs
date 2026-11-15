# R3 preflight: restore historical evidence before main sync

R registry: `diegoami/borg-reborn-rs@fbd0f691de655c6db61565b1e3d2b5aea92184cd`.
Accepted main: `2f74e1c5d57ce2b41ded1b5b694174b91b51f265`.
Frozen legacy inventory: 2,455 cases from accepted #307 / `1710276`;
the inventory file is copied byte for byte, never regenerated to accept R replacements.

The earlier R import reused 13 accepted legacy paths for different deaths.
Recovery first retains every differing R packet, then restores the accepted
main packet at each original path. Both lineages survive; no source/build,
encounter family, attempt identity or original payload is relabelled.

The actual pinned Git tree contains **83 additional and 13 replaced packets:
96 complete packets**, not the review’s 97 estimate. No incomplete D directory
is tracked or present in the source worktrees. The full tracked inventory was
checked; no evidence was removed to make a count fit.

All 96 packets passed the accepted importer from main and have deterministic
full `D0000-q…` storage IDs. Their `.source-record.json` is byte-identical to
the original R record, every payload file is identical, and the working
record differs only in display/storage ID. These IDs are not canonical
numeric cemetery IDs; use the full ID/path. Central publication remains #344.

All 2,455 accepted legacy file inventories and bytes match the pinned main
snapshot, including undeclared legacy analysis files for this recovery audit.
107 accepted legacy dossiers were missing on R and are restored too.
The ongoing legacy CI gate protects the full record and declared payloads;
it does not freeze undeclared legacy analysis forever. Qualified packages
remain immutable in every file.

The resulting archive has **2,551 packages**, with the exact union of original
R and accepted legacy encounter/result keys: 2,508 death-attempt identities.
The owned index reports 2,505 distinct starts. Counts of storage packages
are not independent lives; byte-preserved copies do not create new trials.
Main’s 92 already-qualified packages remain on main and will arrive in the
separate #343 sync. This recovery changes no gameplay/build/default or pin.

Historical R1/R2 raw JSON is unchanged. Its numeric dossier references resolve
in the original registry above; do not resolve old R numbers against the
restored current legacy slots. Markdown reader notes and the machine-readable
map qualify them. Explicit original-main namespaces in R2-MAIN-SYNC retain
their own recorded main source. The map includes original artifact hashes.

Validation: accepted-main importer baseline passes 2,547 cases; recovered
archive passes importer integrity and storage-ID checks; 92 tool and 33
source-tool tests pass. Four preserved R2 full campaigns reconcile all 32
natural deaths, and the 16 grid-hardening games reconcile all three deaths,
using actual suite manifests/build IDs. No new game is played or claimed.

## Source-qualified lookup

| Original R ID (at the pinned registry) | Full qualified package |
| --- | --- |
| `D2356-rust-seed102` | [D0000-q2e47418bc21ba3f58e67b5f923ace110bab05a1ead95024a6a610c7d44101aea](../docs/deaths/D0000-q2e47418bc21ba3f58e67b5f923ace110bab05a1ead95024a6a610c7d44101aea/report.md) |
| `D2357-rust-seed1020` | [D0000-q8eb183f0ffb1e2a480ebb91085794261388a7bbb8e317d1cd3b73fce240a4ee7](../docs/deaths/D0000-q8eb183f0ffb1e2a480ebb91085794261388a7bbb8e317d1cd3b73fce240a4ee7/report.md) |
| `D2358-rust-seed102` | [D0000-q8f01e193031e90f1cf71eab8ea5d18e9d71fc12236c77bd0a28a8f6d17c9be8e](../docs/deaths/D0000-q8f01e193031e90f1cf71eab8ea5d18e9d71fc12236c77bd0a28a8f6d17c9be8e/report.md) |
| `D2359-rust-seed1020` | [D0000-q4956a6c5ba0363f89a6a896b22e08b66b69de95c5f0e900bcc295872a84ee86b](../docs/deaths/D0000-q4956a6c5ba0363f89a6a896b22e08b66b69de95c5f0e900bcc295872a84ee86b/report.md) |
| `D2360-rust-seed1011` | [D0000-q6b3845647d733b3251d32dd7d88851d11a89a9c988c6caa10d21b50a8d82c36e](../docs/deaths/D0000-q6b3845647d733b3251d32dd7d88851d11a89a9c988c6caa10d21b50a8d82c36e/report.md) |
| `D2361-rust-seed1020` | [D0000-q3b3d4796db3c95c95b6a46cca34c38d25ddeb5ad999cca2d9c69d95851ffe3c5](../docs/deaths/D0000-q3b3d4796db3c95c95b6a46cca34c38d25ddeb5ad999cca2d9c69d95851ffe3c5/report.md) |
| `D2362-rust-seed102` | [D0000-q3232d9e84ff4f07ab54940f0bf2ee17074a06cd32d5d46eee789fcf68f4a2040](../docs/deaths/D0000-q3232d9e84ff4f07ab54940f0bf2ee17074a06cd32d5d46eee789fcf68f4a2040/report.md) |
| `D2363-rust-seed1011` | [D0000-q8616fc404a69604eb50fe24387a0914cf13a47304779726f543e477acb092d45](../docs/deaths/D0000-q8616fc404a69604eb50fe24387a0914cf13a47304779726f543e477acb092d45/report.md) |
| `D2364-rust-seed1020` | [D0000-q326ea2e1f15abe062d305dcd56652facdb4a1c16ea19dac35d7490f66cfeeccb](../docs/deaths/D0000-q326ea2e1f15abe062d305dcd56652facdb4a1c16ea19dac35d7490f66cfeeccb/report.md) |
| `D2365-rust-seed1021` | [D0000-q8411094ca3a9d8cce07c8277c520c889935296ee95c11ba1088012d16ebda107](../docs/deaths/D0000-q8411094ca3a9d8cce07c8277c520c889935296ee95c11ba1088012d16ebda107/report.md) |
| `D2366-rust-seed1043` | [D0000-q35581d033336760ba17b1291dd1296c9da0dedb7592d8a6b395378ae4101ddaf](../docs/deaths/D0000-q35581d033336760ba17b1291dd1296c9da0dedb7592d8a6b395378ae4101ddaf/report.md) |
| `D2367-rust-seed1050` | [D0000-qa07cdebaa092881d1f6632221e6c5ce37e7ec9283467e563723dd9681b41200d](../docs/deaths/D0000-qa07cdebaa092881d1f6632221e6c5ce37e7ec9283467e563723dd9681b41200d/report.md) |
| `D2368-rust-seed1062` | [D0000-qac5cedd544883e081803da538096028b9c26c8ee8c3b5e0c850fa458c9b395c9](../docs/deaths/D0000-qac5cedd544883e081803da538096028b9c26c8ee8c3b5e0c850fa458c9b395c9/report.md) |
| `D2369-rust-seed1072` | [D0000-q7493851f384566e3d022b536f11e956369015a309f71ffaf0b595ec0b9a60168](../docs/deaths/D0000-q7493851f384566e3d022b536f11e956369015a309f71ffaf0b595ec0b9a60168/report.md) |
| `D2370-rust-seed1084` | [D0000-q78baef7fc4f83408218562118e7cbb1b36055153cac48be47f5375a42b7ba83e](../docs/deaths/D0000-q78baef7fc4f83408218562118e7cbb1b36055153cac48be47f5375a42b7ba83e/report.md) |
| `D2371-rust-seed102` | [D0000-qe527d65394d4d01bab4ef8c3fdc16660d25403c2e0f5beefddac84dadfb7ba10](../docs/deaths/D0000-qe527d65394d4d01bab4ef8c3fdc16660d25403c2e0f5beefddac84dadfb7ba10/report.md) |
| `D2372-rust-seed1020` | [D0000-q340afb74a9865e3a7fe5bf3783d1d1d4d6c4dcf3a14167935a2bc24b67464481](../docs/deaths/D0000-q340afb74a9865e3a7fe5bf3783d1d1d4d6c4dcf3a14167935a2bc24b67464481/report.md) |
| `D2373-rust-seed1011` | [D0000-q2a6627b6f4a6c0f39ed652525dce86eee4b6c979326d1ecb5539e6a79b84bca0](../docs/deaths/D0000-q2a6627b6f4a6c0f39ed652525dce86eee4b6c979326d1ecb5539e6a79b84bca0/report.md) |
| `D2374-rust-seed1020` | [D0000-qca2f6f2392d63db974320fd658a83c7565af8bf338aad7c61a81ede9e913f05e](../docs/deaths/D0000-qca2f6f2392d63db974320fd658a83c7565af8bf338aad7c61a81ede9e913f05e/report.md) |
| `D2375-rust-seed102` | [D0000-qf6bf0823f91b6a0cadb86f4edfa64294ed85526c18f9b17d5dc42baf6bae7fb2](../docs/deaths/D0000-qf6bf0823f91b6a0cadb86f4edfa64294ed85526c18f9b17d5dc42baf6bae7fb2/report.md) |
| `D2376-rust-seed1011` | [D0000-qbb85162fa966dbe9f9e465415ab468b9c4c56a1357e10bdcc57ab1ec246070d7](../docs/deaths/D0000-qbb85162fa966dbe9f9e465415ab468b9c4c56a1357e10bdcc57ab1ec246070d7/report.md) |
| `D2377-rust-seed1011` | [D0000-qc657348815decebe7dd1b7712d46ab640983b1355a661a6ab0fb4c6c20502e99](../docs/deaths/D0000-qc657348815decebe7dd1b7712d46ab640983b1355a661a6ab0fb4c6c20502e99/report.md) |
| `D2378-rust-seed1020` | [D0000-q29d200b5fff2cf197cd31ceb4614471fa80133c4fd9e30915c798b2463dd71b1](../docs/deaths/D0000-q29d200b5fff2cf197cd31ceb4614471fa80133c4fd9e30915c798b2463dd71b1/report.md) |
| `D2379-rust-seed1021` | [D0000-q702b8742ada77ea6445b4dc6a80603f39280d18e553c59ec3e67bb7a9faa7547](../docs/deaths/D0000-q702b8742ada77ea6445b4dc6a80603f39280d18e553c59ec3e67bb7a9faa7547/report.md) |
| `D2380-rust-seed1043` | [D0000-q074a682ba7b91cf4689f38779de16f11a9d3ce4937b3cd8e9ecffd36f9970579](../docs/deaths/D0000-q074a682ba7b91cf4689f38779de16f11a9d3ce4937b3cd8e9ecffd36f9970579/report.md) |
| `D2381-rust-seed1050` | [D0000-qaaa7f80e007d5b79983b2105c1838fad1fb06fbbbba1a636f32eb266becc1b4d](../docs/deaths/D0000-qaaa7f80e007d5b79983b2105c1838fad1fb06fbbbba1a636f32eb266becc1b4d/report.md) |
| `D2382-rust-seed1062` | [D0000-qfd2676cc2b5c59501edee7270f1fa9bed2be6f6c8ff86a821019266f7d99e346](../docs/deaths/D0000-qfd2676cc2b5c59501edee7270f1fa9bed2be6f6c8ff86a821019266f7d99e346/report.md) |
| `D2383-rust-seed1072` | [D0000-q7998021834f45b33c92884746a626903b708bc52cdab4adbdd5b5bf551597e30](../docs/deaths/D0000-q7998021834f45b33c92884746a626903b708bc52cdab4adbdd5b5bf551597e30/report.md) |
| `D2384-rust-seed1084` | [D0000-q1161bef851d62bff74f662ed1bb9e99e9129b45dd4f490877341ef8090fa2815](../docs/deaths/D0000-q1161bef851d62bff74f662ed1bb9e99e9129b45dd4f490877341ef8090fa2815/report.md) |
| `D2385-rust-seed102` | [D0000-qf4d2a72764472fd4c49851fd7317a212e767163c7ba35e8d711781bb464255bd](../docs/deaths/D0000-qf4d2a72764472fd4c49851fd7317a212e767163c7ba35e8d711781bb464255bd/report.md) |
| `D2386-rust-seed1020` | [D0000-qacfe13c86e2ecae836d3afadde1569b4c84ef2049ceb1dc6bbeae0d5c05554ac](../docs/deaths/D0000-qacfe13c86e2ecae836d3afadde1569b4c84ef2049ceb1dc6bbeae0d5c05554ac/report.md) |
| `D2387-rust-seed102` | [D0000-q21b571403eab42cd32088d6205527f906e176c2dddd38af28710fa28db3b23c9](../docs/deaths/D0000-q21b571403eab42cd32088d6205527f906e176c2dddd38af28710fa28db3b23c9/report.md) |
| `D2388-rust-seed1020` | [D0000-qedc6fe8d834438d0ee275b5000b9145e80b5ffa2c6081f72a763764770679613](../docs/deaths/D0000-qedc6fe8d834438d0ee275b5000b9145e80b5ffa2c6081f72a763764770679613/report.md) |
| `D2389-rust-seed1011` | [D0000-qd5bc56dcb295b2720061eaa17258fec0720a8e7637d91c93166c6dcc35fb36bd](../docs/deaths/D0000-qd5bc56dcb295b2720061eaa17258fec0720a8e7637d91c93166c6dcc35fb36bd/report.md) |
| `D2390-rust-seed1020` | [D0000-qabdd6740ee8abe3d3aa8ad2834513d612f0a01005a359f6ef08a983b2b62310c](../docs/deaths/D0000-qabdd6740ee8abe3d3aa8ad2834513d612f0a01005a359f6ef08a983b2b62310c/report.md) |
| `D2391-rust-seed1021` | [D0000-qb2d33de750a735a302a196867e906e08c7bbe46faa32e94fc21d8fd8b2d62278](../docs/deaths/D0000-qb2d33de750a735a302a196867e906e08c7bbe46faa32e94fc21d8fd8b2d62278/report.md) |
| `D2392-rust-seed1043` | [D0000-qc9f44521e8c281a9de6ec0d9be493be1b968980d90cd7c077fa4246d8490eba0](../docs/deaths/D0000-qc9f44521e8c281a9de6ec0d9be493be1b968980d90cd7c077fa4246d8490eba0/report.md) |
| `D2393-rust-seed1050` | [D0000-q9536ef1fe3a00fa175f9a98e48eae8958b273228df678e053b6eb9d197eabf3c](../docs/deaths/D0000-q9536ef1fe3a00fa175f9a98e48eae8958b273228df678e053b6eb9d197eabf3c/report.md) |
| `D2394-rust-seed1062` | [D0000-q42d70a77560c48450d7361bc53360a94baed6d078dd78593b9a9fca8d15ff7e1](../docs/deaths/D0000-q42d70a77560c48450d7361bc53360a94baed6d078dd78593b9a9fca8d15ff7e1/report.md) |
| `D2395-rust-seed1072` | [D0000-qcd2f614edeb0eb9d875268ab0e495db75940120d6f3e752e0c0cd377fd34f450](../docs/deaths/D0000-qcd2f614edeb0eb9d875268ab0e495db75940120d6f3e752e0c0cd377fd34f450/report.md) |
| `D2396-rust-seed1084` | [D0000-q9609154babcbb45f70618bb00f0d3ebb08f0a85681d889ef23c4e84159f6b442](../docs/deaths/D0000-q9609154babcbb45f70618bb00f0d3ebb08f0a85681d889ef23c4e84159f6b442/report.md) |
| `D2397-rust-seed102` | [D0000-q61ba8178fcabab3dd64193793fe8ee1541e9737c479ad07aafbf16d254d352c1](../docs/deaths/D0000-q61ba8178fcabab3dd64193793fe8ee1541e9737c479ad07aafbf16d254d352c1/report.md) |
| `D2398-rust-seed1020` | [D0000-q1300e248af486f4ffc7dfff0ce7815963d68cf062df914aa23ffc0212104c0c1](../docs/deaths/D0000-q1300e248af486f4ffc7dfff0ce7815963d68cf062df914aa23ffc0212104c0c1/report.md) |
| `D2399-rust-seed1004` | [D0000-qa5ff7969a8cb18214205b9ce78fee2552c3a3e3497397965edf0cf5643d0cf09](../docs/deaths/D0000-qa5ff7969a8cb18214205b9ce78fee2552c3a3e3497397965edf0cf5643d0cf09/report.md) |
| `D2400-rust-seed1010` | [D0000-qc4b1d29620d204e3a7be91ef53a3380824578575c52b0f9b011b74e74fdf59aa](../docs/deaths/D0000-qc4b1d29620d204e3a7be91ef53a3380824578575c52b0f9b011b74e74fdf59aa/report.md) |
| `D2401-rust-seed1011` | [D0000-qeeac49821de520297eb330d0ad3b0068719e15136236cc15b9ba78cbda7c4fa9](../docs/deaths/D0000-qeeac49821de520297eb330d0ad3b0068719e15136236cc15b9ba78cbda7c4fa9/report.md) |
| `D2402-rust-seed1012` | [D0000-q3299475f92a73a3aa55619724f210c547547ab9bca774d2935cea6496870f98e](../docs/deaths/D0000-q3299475f92a73a3aa55619724f210c547547ab9bca774d2935cea6496870f98e/report.md) |
| `D2403-rust-seed1013` | [D0000-qa4d429834eb9e1672cef35d4bbe9f02a958d37933872cd4da3617306913fec99](../docs/deaths/D0000-qa4d429834eb9e1672cef35d4bbe9f02a958d37933872cd4da3617306913fec99/report.md) |
| `D2404-rust-seed1014` | [D0000-q885bd16b9c962fb800c4ccd46a11540f912b86a1956e2f06b17b3f9511f4ff0c](../docs/deaths/D0000-q885bd16b9c962fb800c4ccd46a11540f912b86a1956e2f06b17b3f9511f4ff0c/report.md) |
| `D2405-rust-seed1021` | [D0000-q1961d3cbc57529b955b734002289f585dc8bf221f811ad7cbbbec6f873cda323](../docs/deaths/D0000-q1961d3cbc57529b955b734002289f585dc8bf221f811ad7cbbbec6f873cda323/report.md) |
| `D2406-rust-seed1022` | [D0000-qd9b6cef231f4502265ae245e03cb903c48729cc05752c34a7c1469b9382440bc](../docs/deaths/D0000-qd9b6cef231f4502265ae245e03cb903c48729cc05752c34a7c1469b9382440bc/report.md) |
| `D2407-rust-seed1023` | [D0000-qbed82fb56a6de81455ff8146468b4ef31bdd70411b25f41d21faaff773658310](../docs/deaths/D0000-qbed82fb56a6de81455ff8146468b4ef31bdd70411b25f41d21faaff773658310/report.md) |
| `D2408-rust-seed1030` | [D0000-q1cb2a3d4538b77ae78c3975b49019ac8d22c60e0e97f8afdf71589834d32575c](../docs/deaths/D0000-q1cb2a3d4538b77ae78c3975b49019ac8d22c60e0e97f8afdf71589834d32575c/report.md) |
| `D2409-rust-seed1031` | [D0000-q9eee81b96965fc71ffdcb867510de953a479e618500c63d8b3e5220a52a1623b](../docs/deaths/D0000-q9eee81b96965fc71ffdcb867510de953a479e618500c63d8b3e5220a52a1623b/report.md) |
| `D2410-rust-seed1032` | [D0000-q3e958312b947c080eeb15b57ffab75d35aed3d3f051b045839f3f039283f7fd9](../docs/deaths/D0000-q3e958312b947c080eeb15b57ffab75d35aed3d3f051b045839f3f039283f7fd9/report.md) |
| `D2411-rust-seed1033` | [D0000-qb39867c086aaf7a18da6c4a0b25a1ec5d196a76014fb7e1517c86a3bbbfb612f](../docs/deaths/D0000-qb39867c086aaf7a18da6c4a0b25a1ec5d196a76014fb7e1517c86a3bbbfb612f/report.md) |
| `D2412-rust-seed1034` | [D0000-q20aad7df325ba155cf2717f6aa9f362ccb7905391feb5fc245553868c0680de6](../docs/deaths/D0000-q20aad7df325ba155cf2717f6aa9f362ccb7905391feb5fc245553868c0680de6/report.md) |
| `D2413-rust-seed1040` | [D0000-q38379eb48db07e38c6224448467f624bb7d628a87678e99834273775e19baf50](../docs/deaths/D0000-q38379eb48db07e38c6224448467f624bb7d628a87678e99834273775e19baf50/report.md) |
| `D2414-rust-seed1041` | [D0000-q1fd07bd0286ba6580f63f90fa7a5befa43479ba6e938ee7e33cc1adf94fdad03](../docs/deaths/D0000-q1fd07bd0286ba6580f63f90fa7a5befa43479ba6e938ee7e33cc1adf94fdad03/report.md) |
| `D2415-rust-seed1042` | [D0000-qa56f1a0287a725827aa205e439ea3a50cb3a1d3e84a7c2bdfaae8a9a9112d05b](../docs/deaths/D0000-qa56f1a0287a725827aa205e439ea3a50cb3a1d3e84a7c2bdfaae8a9a9112d05b/report.md) |
| `D2416-rust-seed1043` | [D0000-q37f40cfe291049dc676b810b808cf6c5f9f33b3d04cd495ebd612b982072498c](../docs/deaths/D0000-q37f40cfe291049dc676b810b808cf6c5f9f33b3d04cd495ebd612b982072498c/report.md) |
| `D2417-rust-seed1044` | [D0000-q885b28a9119b8da5cbe07c1b56c5e16a709856f27aa1a52cfcbd4bb9ecf5df0d](../docs/deaths/D0000-q885b28a9119b8da5cbe07c1b56c5e16a709856f27aa1a52cfcbd4bb9ecf5df0d/report.md) |
| `D2418-rust-seed1050` | [D0000-qecc81e78d76fe54c32ef97fefd8afeee7f18a224a3a71600645dad2651ade5cc](../docs/deaths/D0000-qecc81e78d76fe54c32ef97fefd8afeee7f18a224a3a71600645dad2651ade5cc/report.md) |
| `D2419-rust-seed1053` | [D0000-q9c24c1122578bfeff0d61040c6d910d55c0a17336b6d00f56a87282c4933c19c](../docs/deaths/D0000-q9c24c1122578bfeff0d61040c6d910d55c0a17336b6d00f56a87282c4933c19c/report.md) |
| `D2420-rust-seed1062` | [D0000-qa009bcce3f9295d0fee57717120e33f11e2b14c12ada0621519a7d83f75abdd8](../docs/deaths/D0000-qa009bcce3f9295d0fee57717120e33f11e2b14c12ada0621519a7d83f75abdd8/report.md) |
| `D2421-rust-seed1063` | [D0000-q957263aeadf44430825f35215f77a90b400a5d78eae21952eaa26d0728b50715](../docs/deaths/D0000-q957263aeadf44430825f35215f77a90b400a5d78eae21952eaa26d0728b50715/report.md) |
| `D2422-rust-seed1064` | [D0000-qa1cd9a3da8bee4572d6e8484e64509f3c8c7a2a5520bc8ec349a71be66a27155](../docs/deaths/D0000-qa1cd9a3da8bee4572d6e8484e64509f3c8c7a2a5520bc8ec349a71be66a27155/report.md) |
| `D2423-rust-seed1070` | [D0000-q0ae1b194e19d5f47881a66b06180618633f9684f644cc562255fb42e14fc5507](../docs/deaths/D0000-q0ae1b194e19d5f47881a66b06180618633f9684f644cc562255fb42e14fc5507/report.md) |
| `D2424-rust-seed1073` | [D0000-qbb3f6c67ee2260ffe039c877dcde521df8e1d90a7f29124025e3696c3fa1e39f](../docs/deaths/D0000-qbb3f6c67ee2260ffe039c877dcde521df8e1d90a7f29124025e3696c3fa1e39f/report.md) |
| `D2425-rust-seed1074` | [D0000-qed9c8ec48574a65fc027af1d8d0393a35c9854f96d46234f67c124b870cfc690](../docs/deaths/D0000-qed9c8ec48574a65fc027af1d8d0393a35c9854f96d46234f67c124b870cfc690/report.md) |
| `D2426-rust-seed1082` | [D0000-qd6e215606053328ebc5414febede759dd5ba0869f485f675983db888f2d09df1](../docs/deaths/D0000-qd6e215606053328ebc5414febede759dd5ba0869f485f675983db888f2d09df1/report.md) |
| `D2427-rust-seed1083` | [D0000-q18501f46f55d3664b5110fc47fdf355cd538b78443d4e9c9deca22646047454d](../docs/deaths/D0000-q18501f46f55d3664b5110fc47fdf355cd538b78443d4e9c9deca22646047454d/report.md) |
| `D2428-rust-seed1084` | [D0000-q89df692ef2858e025f324496a953bbeb34f2911d10e95327402a7d0da5904924](../docs/deaths/D0000-q89df692ef2858e025f324496a953bbeb34f2911d10e95327402a7d0da5904924/report.md) |
| `D2429-rust-seed1011` | [D0000-qf776932d37b9fdc2f6dcfd571c2d404cb9046adc2b3e9f1340985853abe20efa](../docs/deaths/D0000-qf776932d37b9fdc2f6dcfd571c2d404cb9046adc2b3e9f1340985853abe20efa/report.md) |
| `D2430-rust-seed1020` | [D0000-q85254424be35ddcd4f2168d4e6d2d0bd075d96c39aa31fd729965265f795b8eb](../docs/deaths/D0000-q85254424be35ddcd4f2168d4e6d2d0bd075d96c39aa31fd729965265f795b8eb/report.md) |
| `D2431-rust-seed1021` | [D0000-qbd378166b343703f520eaa146fc9823399fd99fceefb67979a61a1fe8207d61c](../docs/deaths/D0000-qbd378166b343703f520eaa146fc9823399fd99fceefb67979a61a1fe8207d61c/report.md) |
| `D2432-rust-seed1043` | [D0000-q37c6fa23f7ed3d3d875da1335cd97575a27e8cd6c59a57bd27e660f6a84fbebe](../docs/deaths/D0000-q37c6fa23f7ed3d3d875da1335cd97575a27e8cd6c59a57bd27e660f6a84fbebe/report.md) |
| `D2433-rust-seed1050` | [D0000-q65a3d0e52ddc014faf19713e6407adaa52455e1656d6c9cb88ab8771abfe225e](../docs/deaths/D0000-q65a3d0e52ddc014faf19713e6407adaa52455e1656d6c9cb88ab8771abfe225e/report.md) |
| `D2434-rust-seed1062` | [D0000-q90364933bdf39efa37599b8e57e4c15107de0a36b9277b11ddb4fefd17ff3bdc](../docs/deaths/D0000-q90364933bdf39efa37599b8e57e4c15107de0a36b9277b11ddb4fefd17ff3bdc/report.md) |
| `D2435-rust-seed1072` | [D0000-q187676759c5f0a8ed34d1b76ce037a06439110c55f4231ae1d533add6a2319c6](../docs/deaths/D0000-q187676759c5f0a8ed34d1b76ce037a06439110c55f4231ae1d533add6a2319c6/report.md) |
| `D2436-rust-seed1084` | [D0000-q890fe054ccce3eb685fd528f85b78e397da5f1c2bee1b676a6d2c18c1bbc8b3b](../docs/deaths/D0000-q890fe054ccce3eb685fd528f85b78e397da5f1c2bee1b676a6d2c18c1bbc8b3b/report.md) |
| `D2437-rust-seed102` | [D0000-qd4050c294104cb57cc529ef750fcfb823f3fd96e8c0d52e90c862f36636c6c99](../docs/deaths/D0000-qd4050c294104cb57cc529ef750fcfb823f3fd96e8c0d52e90c862f36636c6c99/report.md) |
| `D2438-rust-seed1020` | [D0000-q5cd9a30838d2749e46fba3fc4658794b69e740c647cb173b553c68f9edd5b409](../docs/deaths/D0000-q5cd9a30838d2749e46fba3fc4658794b69e740c647cb173b553c68f9edd5b409/report.md) |
| `D2439-rust-seed1011` | [D0000-q2fc8329e6d6a66b237481403e543e863a90483f0b449511f9fcfe2fcf4458ebe](../docs/deaths/D0000-q2fc8329e6d6a66b237481403e543e863a90483f0b449511f9fcfe2fcf4458ebe/report.md) |
| `D2440-rust-seed1020` | [D0000-q4fc609746f43f798a164cd96dfcfc64991b0b597b2dd143686c8df81d218da66](../docs/deaths/D0000-q4fc609746f43f798a164cd96dfcfc64991b0b597b2dd143686c8df81d218da66/report.md) |
| `D2441-rust-seed1021` | [D0000-q0230047df46210c53d1d0765f934a18e7da5913a133414869a85fca732598634](../docs/deaths/D0000-q0230047df46210c53d1d0765f934a18e7da5913a133414869a85fca732598634/report.md) |
| `D2442-rust-seed1043` | [D0000-qac408c284552dd81b0297d68a156add0a7638676b53cfe276ad55e0ab7c58ab1](../docs/deaths/D0000-qac408c284552dd81b0297d68a156add0a7638676b53cfe276ad55e0ab7c58ab1/report.md) |
| `D2443-rust-seed1050` | [D0000-q6664b878248aea67fb19c214e09aad6421a6ed62cc2f9b29bb2b393e93726937](../docs/deaths/D0000-q6664b878248aea67fb19c214e09aad6421a6ed62cc2f9b29bb2b393e93726937/report.md) |
| `D2444-rust-seed1062` | [D0000-q9feca0d03ddd0f6bb43261fdfc2c4bf6423d5326f11dcc25d50be509d0525e74](../docs/deaths/D0000-q9feca0d03ddd0f6bb43261fdfc2c4bf6423d5326f11dcc25d50be509d0525e74/report.md) |
| `D2445-rust-seed1072` | [D0000-q0dcc334a4dee5b389b23be93e20f0a9f02f903660950a62d4bc40436a65261e3](../docs/deaths/D0000-q0dcc334a4dee5b389b23be93e20f0a9f02f903660950a62d4bc40436a65261e3/report.md) |
| `D2446-rust-seed1084` | [D0000-qa6e52720d68e4ce7191859f239fe70c09021545bf8cd26f940321154bc181fe0](../docs/deaths/D0000-qa6e52720d68e4ce7191859f239fe70c09021545bf8cd26f940321154bc181fe0/report.md) |
| `D2447-rust-seed102` | [D0000-q95779ebe33bb34a15cd4a29eb90e5f36869a6313c837ce1785d6ed217a47ea3f](../docs/deaths/D0000-q95779ebe33bb34a15cd4a29eb90e5f36869a6313c837ce1785d6ed217a47ea3f/report.md) |
| `D2448-rust-seed1020` | [D0000-q439a07cf22e4c411cb5477ad43054c1f7e99ed19836403eb330a75f51b0b973b](../docs/deaths/D0000-q439a07cf22e4c411cb5477ad43054c1f7e99ed19836403eb330a75f51b0b973b/report.md) |
| `D2449-rust-seed1011` | [D0000-q881284518ad3b1dad280f317ec0854261364dceb951153df7b3177a8014577ce](../docs/deaths/D0000-q881284518ad3b1dad280f317ec0854261364dceb951153df7b3177a8014577ce/report.md) |
| `D2450-rust-seed102` | [D0000-q93d5af345f728f6d027052be0d7f39de8bb6f21060498abbfd1c2bcfaa2d5c55](../docs/deaths/D0000-q93d5af345f728f6d027052be0d7f39de8bb6f21060498abbfd1c2bcfaa2d5c55/report.md) |
| `D2451-rust-seed1020` | [D0000-qbc41627ec05d9ea3ab91d0b37ee064548949777a88cd91e2546e27ba706377b5](../docs/deaths/D0000-qbc41627ec05d9ea3ab91d0b37ee064548949777a88cd91e2546e27ba706377b5/report.md) |
