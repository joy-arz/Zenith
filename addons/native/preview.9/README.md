# Native mods for Zenith Rust Edition Preview 9

These adapters target engine commit `b10a5c68e39b3452690a2c8aeb94f187f5dfc5df`
with Horizon Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
They are specific to the registry archives below. Keep a separate unpacked
folder for each mod.

Verify the original archive against its SHA-256, unpack it, and apply the
matching patch with `git apply --check` followed by `git apply` from inside
that folder. Each folder should contain `horizon-native.json` and
`mixins/<id>/`. Use those folders as input to the launcher's native build
for Preview 9 after the signed engine is available. Keep the original mod
sources and their MIT notices.

| Original project | Registry archive | SHA-256 |
| --- | --- | --- |
| [The Basic HUD](https://github.com/repositorykreml1n/the-basic-hud) | [0.5.0](https://github.com/horizon-rust/horizon-mods/releases/download/basic_hud-0.5.0/basic_hud-0.5.0.hnmod) | `97011cc7374354c6293678c7ab593240d0512bffde1d7b642dad521e1df3f8ac` |
| [BiomeCompass](https://github.com/d3grando/biomecompass) | [0.2.2](https://github.com/horizon-rust/horizon-mods/releases/download/biomecompass-0.2.2/biomecompass-0.2.2.hnmod) | `d9a4aedcf22318774e1ac6bed6f1e00aca13a38c8dc3426d5a7800e4d8c61bf5` |
| [Coordinates HUD](https://github.com/horizon-rust/horizon-mods/tree/main/native/coords-hud) | [2.0.0](https://github.com/horizon-rust/horizon-mods/releases/download/coords_hud-2.0.0/coords_hud-2.0.0.hnmod) | `432376fad3062b3517e274d3b8c0d0b52fc5d6de2aee20a0b340c8cab5a8cf08` |
| [Fast Items for Horizon](https://github.com/Lyvt-Dev/FastItems-for-Horizon) | [0.1.0](https://github.com/horizon-rust/horizon-mods/releases/download/fast_items-0.1.0/fast_items-0.1.0.hnmod) | `c3acd8adfeac6450c3a13b5172c8abb298cb3ceb355dfc539b4bdd653ef15502` |
| [FPS Counter](https://github.com/horizon-rust/horizon-mods/tree/main/native/fps-overlay) | [2.0.0](https://github.com/horizon-rust/horizon-mods/releases/download/fps_overlay-2.0.0/fps_overlay-2.0.0.hnmod) | `bedb670e2490ce8239906c86eabd3a0c131ef5037b2937607231bdbdba38c171` |
| [3D Skins](https://github.com/umbx-dot/horizon-3d-skins) | [1.1.0](https://github.com/horizon-rust/horizon-mods/releases/download/skins_3d-1.1.0/skins_3d-1.1.0.hnmod) | `ac9f20ea771f2abb25976c06d320883016b13a984075522ecc07323b48994602` |
| [SmartF3](https://github.com/d3grando/smartf3) | [0.1.0](https://github.com/horizon-rust/horizon-mods/releases/download/smartf3-0.1.0/smartf3-0.1.0.hnmod) | `666fc77a12411c8b525dca3d3d478595e760b73c48b62b936fb6026419547929` |

Use the [Create adapter](../../create/README.md#zenith-rust-edition-020-preview9)
for Create 0.8.0-alpha.16. It follows the earlier Zenith patch series.

All eight adapted bundles applied together and the transformed game passed
`cargo check --locked -p horizon-official --bin horizon-official` on macOS
ARM64. Native gameplay and builds on other platforms still need validation.
