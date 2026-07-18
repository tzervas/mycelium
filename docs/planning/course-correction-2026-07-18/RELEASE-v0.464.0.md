# RELEASE — v0.464.0 (course-correction train, staged 2026-07-18)

| Field | Value |
|---|---|
| **Status** | **Staged, not yet cut** — the annotated tags were prepared in-session but every tag push was refused by the session's scoped git relay (HTTP 403 on `refs/tags/*`; branch pushes are permitted, tag pushes are not — a policy boundary, recorded per the blocked-op protocol, not retried or circumvented). Cutting the release = running §3 below from any environment with tag-push permission. `Declared` until the tags exist on the remotes. |
| **Train** | v0.464.0 lockstep (recorded default D-5); no crates.io publishing (D-6/ADR-018 — source-only). |
| **Verification basis** | Every Rust rev: fmt/clippy `-D warnings`/test green standalone (Phase B/C5, `Empirical`). Every `*-myc` rev: delivered with a `DELIVERY.md` honesty record; 26/46 `myc-check` CLEAN, 20/46 FINDINGS on `Declared` seed drafts. Dual-side metrics: `docs/measurements/course-correction-dual-side-2026-07-18.md`. |

## 1. Release revisions (the manifest)

| Repo | Kind | Rev to tag |
|---|---|---|
| `mycelium-bench` | Rust component | `1958e0f6144c927928bc262f78b526f929c9e9bd` |
| `mycelium-build` | Rust component | `bd8087cf1cee6b2755b0512a747ee04889074c43` |
| `mycelium-check` | Rust component | `6d5f99d87157d3285d5ba53765b616136f281a4d` |
| `mycelium-cli` | Rust component | `d6af45148e48f1701c4a6ba470cae0c27c6b07fd` |
| `mycelium-cli-common` | Rust component | `c673fafdd1bcecb713e3560689af6ab152033867` |
| `mycelium-codegen` | Rust component | `505448cbfb5553a34aca726f0d1b884981a83631` |
| `mycelium-core` | Rust component | `46d2515cbd86d2ae4d1365f4adcd2796737e9f0b` |
| `mycelium-doc` | Rust component | `22346d3c1e4bb17b14c4275e60efb04704c22319` |
| `mycelium-fmt` | Rust component | `8dcb0f8081ff8d0f1344d9b4017a52c77b599c1b` |
| `mycelium-l1` | Rust component | `2b92f54349eb0d4f67e32e983874df76908b9ab6` |
| `mycelium-lint` | Rust component | `2c6c86bbdd3e2c7abf9bfc0b81e7924279e9e712` |
| `mycelium-lsp` | Rust component | `ba86f11ef1b33bdd091e897f6549fb87058445fc` |
| `mycelium-proj` | Rust component | `20b8a6d264ac728e81cfe8cd90cec8d2a91370be` |
| `mycelium-runtime` | Rust component | `487b1e7049ff521b1a6fa33f376245089e7dc1e1` |
| `mycelium-sec` | Rust component | `17ce44cf26abffa3212a634262b48aee9618baea` |
| `mycelium-spore` | Rust component | `283f9fd901607841d5302d5935d15d873a32eef7` |
| `mycelium-std-cmp` | Rust component | `e398a5d2ef981933a080736e366f49c9bfdb285c` |
| `mycelium-std-collections` | Rust component | `101e8d6644d6130c9ea364d6bcaf7d8974ad1aa1` |
| `mycelium-std-conformance` | Rust component | `5bde5f86ddce042388b45ea982e79e796168c4cf` |
| `mycelium-std-content` | Rust component | `a6059bae85256fed1272f38a8cbbd2dec2e8c56c` |
| `mycelium-std-core` | Rust component | `376762cc17853e1582684ececf9e760426bcfb0c` |
| `mycelium-std-dense` | Rust component | `632b6d4955cf34224e8c66b936286026ece21954` |
| `mycelium-std-diag` | Rust component | `0ce2e431a4786ec5f974fc66e774cb0a9b77def4` |
| `mycelium-std-error` | Rust component | `dece7d3b1ce12df65cfba0131d151689f1e42a5e` |
| `mycelium-std-fmt` | Rust component | `8e5af9fd4d61472248b6c31634888294d6249c6c` |
| `mycelium-std-fs` | Rust component | `c7fb6dcbce6c37567dcd244c44cd7aa2a76ac11a` |
| `mycelium-std-io` | Rust component | `d2bf3d585f5d2e9d08f21df723547b7c35a26d68` |
| `mycelium-std-iter` | Rust component | `736bfca8675945c6107fcf8eecb3abf0ae6183f9` |
| `mycelium-std-math` | Rust component | `7b0e02d96177754ef8311422abf478abc6e93e12` |
| `mycelium-std-numerics` | Rust component | `2676276b1559f0c1c0b1c0c39ad48ba6ff89d639` |
| `mycelium-std-rand` | Rust component | `088256c5f29031e487cb7a335cbc9ff29794b58d` |
| `mycelium-std-recover` | Rust component | `ad8787428c0d8c1eb2bf3a8cd6504cc39bca00bf` |
| `mycelium-std-runtime` | Rust component | `297d69e42a468adfeb582ea425a10887674ef4cb` |
| `mycelium-std-select` | Rust component | `a800a36061be6177ed4885707bf5d0f5e125ed73` |
| `mycelium-std-spore` | Rust component | `29cbddd0a1480c6b784805bb8d7e5141497d7dc0` |
| `mycelium-std-swap` | Rust component | `55bb071af6b428c933c17c7cd8045f8c8663e5ea` |
| `mycelium-std-sys` | Rust component | `95957a5a91e42f003709d584e47783777e4a4618` |
| `mycelium-std-sys-host` | Rust component | `fb4f3cb71e54b1bab2552060f19322007921f437` |
| `mycelium-std-ternary` | Rust component | `bcc63ee0fcd9e07ae1d9cc85241e251767a6d8bf` |
| `mycelium-std-testing` | Rust component | `6f3a51aa61a60b508fa6a8a15cb7b12e4b736413` |
| `mycelium-std-text` | Rust component | `8ff7317e05f37294a4654f0eb2ed34124cfea34f` |
| `mycelium-std-time` | Rust component | `47ef9e7ec4143c97878083ca5c15930a21eeed83` |
| `mycelium-std-vsa` | Rust component | `3936b492b99ba204e9c156822f70041383a37056` |
| `mycelium-transpile` | Rust component | `4af3ed070bda31fd433d1fd632e5a2c5d881bb06` |
| `mycelium-value` | Rust component | `6d230ad2023a716704c697ac6812a2062624b4eb` |
| `mycelium-bench-myc` | `*-myc` twin | `9ccdd62f2fb7f216ebdc9e867c297236ff4507b1` |
| `mycelium-build-myc` | `*-myc` twin | `2aa2f39bf023f7943266960fc2716dfcf65eefeb` |
| `mycelium-check-myc` | `*-myc` twin | `874f180e752ddb7a8b665265989d13cec8101c36` |
| `mycelium-cli-common-myc` | `*-myc` twin | `e74c0c1ab1c13906c079ef422638668e17022c1a` |
| `mycelium-cli-myc` | `*-myc` twin | `f0e7f0e76d68a11c754434070a3afa13514b4c23` |
| `mycelium-codegen-myc` | `*-myc` twin | `104a7cdb9f2b9372c42d0dcc5fc1b86c5ef65b15` |
| `mycelium-compiler-myc` | `*-myc` twin | `f0d7c9659e74161482dd2aeacdd45eaae60cbdf4` |
| `mycelium-core-myc` | `*-myc` twin | `28ca8a8743a96be905cb0494657081ff361004d5` |
| `mycelium-doc-myc` | `*-myc` twin | `c4c77a0a87a81c336519bacc2df83f741fe60676` |
| `mycelium-fmt-myc` | `*-myc` twin | `6553e3494156371d29fb8abc2fd7ef6e5ae00737` |
| `mycelium-l1-myc` | `*-myc` twin | `123c45f9c4ebcb797c90585bac1c224a795cbf0c` |
| `mycelium-lint-myc` | `*-myc` twin | `65222d2e8ef09978b5b366dda17573b5c7b91b46` |
| `mycelium-lsp-myc` | `*-myc` twin | `11a3b6317f7722163fe6105abe68a8c4110f0713` |
| `mycelium-proj-myc` | `*-myc` twin | `b9b8737101d77f1561e30ba74f73d524b8533377` |
| `mycelium-runtime-myc` | `*-myc` twin | `e6119973ec8ded6345597a220b20af49246a6d59` |
| `mycelium-sec-myc` | `*-myc` twin | `b4bc17a4a1b8cf07468a5ab89c88606c11151049` |
| `mycelium-spore-myc` | `*-myc` twin | `53702198f8603f286d3e4f9b3e7eca6c274354a8` |
| `mycelium-std-cmp-myc` | `*-myc` twin | `09ba84a28d04404f9fd44fc5ad778f7df7b502f1` |
| `mycelium-std-collections-myc` | `*-myc` twin | `8d12c3fc6138afe31461a2dc2f874592edb5ce52` |
| `mycelium-std-conformance-myc` | `*-myc` twin | `61760bebe338277f36b369fd5cd8dc3a7c8510df` |
| `mycelium-std-content-myc` | `*-myc` twin | `a764f5aac5c6194d839b5c8b8b796c54daab6654` |
| `mycelium-std-core-myc` | `*-myc` twin | `0b5a8ad46efd3f79d2546c1de4a776e5f3181e7b` |
| `mycelium-std-dense-myc` | `*-myc` twin | `ba3d4627bb85d9e047e61891f5c1f28d31fb8968` |
| `mycelium-std-diag-myc` | `*-myc` twin | `ffdfabd2b3a0ca576d5137cda70b3bf8c1883ac5` |
| `mycelium-std-error-myc` | `*-myc` twin | `1a833fef0caf14f75ac45d797c3843df0da87a91` |
| `mycelium-std-fmt-myc` | `*-myc` twin | `e731978cd3829f8ba335f3d1911632b978da5da3` |
| `mycelium-std-fs-myc` | `*-myc` twin | `bea9cd10a1b771d174dc75b1c020bad3426f67da` |
| `mycelium-std-io-myc` | `*-myc` twin | `7b61a536f0d9fe64cc2afee9468ea28a1d4c2fc7` |
| `mycelium-std-iter-myc` | `*-myc` twin | `da21ced5672b8fb15d2ba3ee18c76e27b73e1b37` |
| `mycelium-std-math-myc` | `*-myc` twin | `9e6741057dc6857fd275b4b174eb47b48ec0ee7a` |
| `mycelium-std-numerics-myc` | `*-myc` twin | `281163b4e600aa5aa6c43d304367d2529f61af47` |
| `mycelium-std-rand-myc` | `*-myc` twin | `9b2f45c4d92e98df6c7d6cdee3082842daa8c72f` |
| `mycelium-std-recover-myc` | `*-myc` twin | `d53af1a381b6bc5a6a661980970f152fc1350bdb` |
| `mycelium-std-runtime-myc` | `*-myc` twin | `eeeb2a76584ece93b3e8d3cc4970557a8eb2e4f7` |
| `mycelium-std-select-myc` | `*-myc` twin | `ac776eefce5e8de3882dc172c09a80af108c78d1` |
| `mycelium-std-spore-myc` | `*-myc` twin | `b3869f6e3ff09b545498ed9f31f8e4d4dc290791` |
| `mycelium-std-swap-myc` | `*-myc` twin | `de961320747cdb4dacbce06565932805d9140cef` |
| `mycelium-std-sys-host-myc` | `*-myc` twin | `0a4d0f8aab442b3dfa13a8fb88c0c2d50d38095b` |
| `mycelium-std-sys-myc` | `*-myc` twin | `a84a9c468aa165e0031309ec32c82750083a4775` |
| `mycelium-std-ternary-myc` | `*-myc` twin | `494ca210ff5d72325964fe598fab4b25c362afa7` |
| `mycelium-std-testing-myc` | `*-myc` twin | `37734631ae9157b2f05a7ef088a37dd458cf2404` |
| `mycelium-std-text-myc` | `*-myc` twin | `a1e5a1ef16910831fb85b7b218cd63423eae8a54` |
| `mycelium-std-time-myc` | `*-myc` twin | `a7d73be8704dc03639d7e4c73236fefca2da9a5d` |
| `mycelium-std-vsa-myc` | `*-myc` twin | `abae4e97704e141197edf258e035b8daed081ebb` |
| `mycelium-transpile-myc` | `*-myc` twin | `ec2fddbb38afbfabac197baed4f0c1e11e8e4da6` |
| `mycelium-value-myc` | `*-myc` twin | `22c349d1d18289202c6498418d0455d4793b1aab` |
| `mycelium-lang` | umbrella | `7304b1170a6200b400d4f979754866245f5d4c54` |

## 2. Release notes text

Rust components: standalone-buildable at the tagged rev; carries the AX-core delta (first-fault
envelope, `policy: ambient`, grade catalog, meet-boundary, ternary `i64`→`i128` ceiling lift) at
frozen lockstep pins. `*-myc` twins: graduated monorepo nodules refreshed where they exist; seed
drafts remain `Declared` (see each repo's `DELIVERY.md`). Umbrella: lock v2 pins all 91 at the
revs above. Reproducible tarball per repo: `git archive --format=tar.gz v0.464.0`.

## 3. Cutting the tags (one command per repo, from an authorized environment)

For each row above:
`git -C <clone> tag -a v0.464.0 -m "<notes §2>" <rev> && git -C <clone> push origin refs/tags/v0.464.0`
(fresh tags only — never overwrite an existing `v0.464.0`). GitHub Release objects can then be
created from the tags (also unavailable to this session — no release-creation tool in its set).

## 4. Monorepo promotion (held)

The monorepo's own release path is the tiered promotion: PR #1705 (this branch → the working
tier) is held for maintainer review; the staging-tier and terminal squash promotions follow it
(CC-B7). No monorepo tag is staged here — the monorepo releases via the squash, not this train.
