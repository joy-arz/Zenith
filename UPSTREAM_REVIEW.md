# Zenith upstream review

Reviewed 6 November 2026 for joy-arz. Priorities: data integrity, reliable updates, offline multiplayer and local hosting. Source licenses were checked before considering code for reuse.

## Upstream synchronization

The five commits through `a1ff75a4a525b6a9e1ed68f74cf9d54161596370` merged without conflicts. They preserve caller-selected harness work directories and consolidate mass nightly-audit failures into one report. All eight affected Python tests passed. Upstream commit IDs and contributor attribution are preserved.

## Follow-up synchronization for Zenith 0.3.2

The follow-up request reported 13 missing commits. Fetching upstream found 21 incoming commits through `41f822b65`, including merge commits. This synchronization includes multiplayer player-list and name-tag performance fixes, slow LAN event attribution, 26.2 client registry adapters, the native transparent-screen hook, and refreshed upstream documentation.

The merge had one README conflict. The resolution retains the upstream layout and screenshots while preserving Zenith branding, joy-arz attribution, fork downloads, and offline account instructions. No source-code conflicts occurred. Zenith’s launcher changes and the two selected backports remain intact.

The first-run offline Play regression is fixed in this rebuild: both launcher themes now allow preparation from the player’s local Minecraft install when offline access is enabled. Busy launches still wait; strict account mode and authenticated downloads retain their checks. Account settings now explain local offline access instead of incorrectly claiming that missing Microsoft sign-in prevents all gameplay.

## Selected fixes

| Source | Problem | Validation |
| --- | --- | --- |
| [PR #10173](https://github.com/horizon-rust/horizon/pull/10173), addresses [#10071](https://github.com/horizon-rust/horizon/issues/10071) | Automatic content-mod updates re-enable disabled mods. Explicit installs now enable a mod; automatic updates retain its existing state. | Eight registry auto-update tests pass, including real fixture packages, disabled/enabled transitions and concurrent removal. The appended test was adapted to the current test file. |
| [PR #9974](https://github.com/horizon-rust/horizon/pull/9974), addresses [#9969](https://github.com/horizon-rust/horizon/issues/9969) | GUI game updates accumulate obsolete Nightly installations. Automatically installed Nightlies are marked and pruned with separate retention limits. | Five pruning tests and four Nightly tests pass. Selected, pinned, running, manual and legacy installations retain their existing protections. |

The baseline Zenith 0.3.0 release passed all five platform jobs and signature verification. These two backports are included in the 0.3.1 update.

## Reviewed but not imported

- [Issue #9217](https://github.com/horizon-rust/horizon/issues/9217): the path-traversal half is already fixed by merged [PR #9254](https://github.com/horizon-rust/horizon/pull/9254) and present in this fork. The remaining digest check needs the cosmetic service's hash contract; the report states that some live hashes do not match truncated SHA-256. No guessed digest rule was added.
- [Gateway prototype #6668](https://github.com/horizon-rust/horizon/pull/6668): adds a hostname-routing gateway. Zenith already supports owner-managed port forwarding; this prototype adds infrastructure beyond that request.
- Mass nightly parity reports were checked against their reported compiler/runner failures. Those reports alone do not establish separate gameplay regressions; the five-commit sync includes the upstream reporting fix.
- Mobile, web, large native-hook and older-protocol proposals expand platform or protocol scope and need dedicated compatibility validation.

## Fork inventory

The public fork inventory included the leading Android, iOS, offline, Switch and web forks. The relevant desktop/offline branches were compared with upstream; no entire fork was merged.

| Fork | Exact reviewed head | License | Decision |
| --- | --- | --- | --- |
| [0CYBER/horizon-free](https://github.com/0CYBER/horizon-free) | `f2e70e9630e0c3457ce2d8437a9c857950963dce` | MIT | Its main focus overlaps Zenith's offline access and fork updating. Its branch is substantially behind current upstream; no code imported. |
| [bunchy7s/horizon-offline](https://github.com/bunchy7s/horizon-offline) | `fb28b692962b7690e69bab41f6984f022f4333df` | MIT | Offline access, packaging and synchronization changes overlap the implemented features; no code imported. |
| [arismoko/horizon](https://github.com/arismoko/horizon) | `36210b2b50ed05202dc47699eded0359d7c9173a` | MIT | Compared branch metadata and changed-file inventory. No independently necessary desktop fix selected. |

This review made no comments, claims, issue closures or merges in the upstream repository.

## Local validation

The complete launcher suite passes: 752 tests, with eight existing opt-in tests ignored. Clippy passes with warnings denied. All eight Python tests affected by the five-commit upstream sync pass. Tests and smoke runs use isolated temporary data directories.

Baseline release: [Zenith 0.3.0](https://github.com/joy-arz/horizon/releases/tag/launcher%400.3.0), built from `830207112c5edf6c042a34d7bb4675e598d2168c`.

## Offline download follow-up

The missing-files report showed a second gap after the first-run Play fix: offline profiles could prepare existing files, but the downloader still required a Microsoft owner. Zenith now selects an explicit offline download path and uses the existing pinned-manifest, hash-checked, resumable downloader. Offline profiles do not fabricate Microsoft accounts, tokens, or ownership. Microsoft profiles retain actual sign-in checks.

Reviewed the MIT-licensed Horizon forks [Domaman202/horizon-crack at ad038ae2d7d91000b95a18f77ae439ed3eab44f3](https://github.com/Domaman202/horizon-crack/tree/ad038ae2d7d91000b95a18f77ae439ed3eab44f3) and [sirruserror/horizon-crack at eb181246292e5b3f4aef29b5495293be825cdbfc](https://github.com/sirruserror/horizon-crack/tree/eb181246292e5b3f4aef29b5495293be825cdbfc). The former replaces ownership checks and removes account tests; the latter adds synthetic authentication responses. No code was imported from either fork.

Reviewed documentation only for [Mecso2/prism-launcher-crack at aa353ff513c4d07fcc7795893b5179a194f2a08e](https://github.com/Mecso2/prism-launcher-crack/tree/aa353ff513c4d07fcc7795893b5179a194f2a08e) (no license declared) and [Diegiwg/PrismLauncher-Cracked at 4916ceeb72022b275c9bae8b94518e7c2236027a](https://github.com/Diegiwg/PrismLauncher-Cracked/tree/4916ceeb72022b275c9bae8b94518e7c2236027a) (GPL-3.0). Their implementation was not read or copied under this repository's contribution rules.

The release runner for Apple Silicon is now `macos-26`, a supported standard ARM64 runner, after repeated macOS 15 capacity cancellations prevented publication. The app's architecture and package names remain unchanged.

The live offline bootstrap also exposed first-use Java detection on macOS: Mojang downloads a `jre.bundle`, while preparation tried the outer component directory. Both Java installation paths now share runtime-home normalization, with a regression test for macOS bundles and flat Windows/Linux layouts.

## Release runner recovery

GitHub repeatedly canceled both macOS 15 and macOS 26 ARM64 jobs before allocating a runner. The Apple Silicon package now cross-compiles on the available `macos-15-intel` runners using Rust’s `aarch64-apple-darwin` target and the Apple SDK. Its job runs native host tests and the offline-profile smoke test, type-checks target-specific tests, and checks the packaged binary’s ARM64 architecture and code signature. Runtime verification of that release binary runs on Apple Silicon separately. No platform package is omitted from the signed release.
