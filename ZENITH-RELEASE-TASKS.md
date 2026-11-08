# Zenith release tasks

The Zenith launcher and its downloadable Rust Edition game will have their own release channel. Mods and packs will target a specific Zenith game release.

## Repositories

- [ ] Create `joy-arz/zenith-modpacks` for pack manifests, conversion tools, compatibility tables, and modpack releases. Start with ATM10 8.5 and keep its local conversion backlog alongside the pack work.
- [ ] Create `joy-arz/zenith-rust-edition` for the game source and releases, native and content mods, shaders, resource packs, and datapacks. Keep the upstream history and license notices with the game source.
- [ ] Audit redistribution rights per component. Keep user-supplied Minecraft and third-party assets that require local import outside the published repositories and archives.

## First pinned game release

- [x] Merge Horizon through `6c2498236` on the isolated `zenith/upstream-sync` branch. Preserve Zenith launcher translations and both projects' license notices.
- [x] Carry the local ATM10 converter and gameplay changes onto `zenith/atm10-integration`. Retain upstream stateful block entity handling and reconcile the embedded world pack loader, dimension commands, and server autosaves.
- [ ] Verify the integrated port in-game with prepared resources. Confirm that existing saves reopen and the embedded world data loads without a separate datapack.
- [ ] Use the newest merged Horizon revision that passes Zenith's release checks. Record the upstream commit, Zenith source commit, native hooks API, and Mixins revision in the release.
- [ ] Build and publish signed game archives for supported Windows, macOS, and Linux targets. Include client and dedicated server smoke tests.
- [ ] Add a Zenith game channel to the launcher. Install the game version declared by a pack or profile and keep save locations scoped to their profile.
- [ ] Fetch native build sources and the source checksum from the selected Zenith release. Keep the upstream registry available alongside Zenith's catalog.
- [ ] Test first install, in-app updates, and rollback against the Zenith channel. Preserve the previous game version while a new build is installing.

## Content compatibility

- [x] Survey the [Horizon registry](https://playhorizon.dev/mods). The November 8 snapshot contains 101 native mods, 5 content mods, 8 shader packs, and 1 resource pack. It lists no modpacks or datapacks, so test those formats with separate fixtures.
- [x] Verify checksums and plan all six Horizon-authored native bundles together on the upstream merge. Horizon Mixins accepted 28 injections and 7 modules after retargeting scratch copies.
- [ ] Inventory the original Horizon registry and official bundles at the selected upstream revision. Record project, artifact version, format, checksum, dependencies, and license. Include native and content mods, modpacks, shaders, resource packs, and datapacks.
- [ ] Verify Horizon registry installs and updates against the pinned Zenith game. Test content registration, shader loading, resource reload, and datapack activation on world creation and reopen. Keep compatible artifacts installable directly from their original source.
- [ ] Verify the upstream native suite (Coordinates HUD, FPS Counter, Keystrokes, Minimap, Recipes, and Zoom) using the pinned Zenith source and Mixins revision. Port incompatible hooks and publish a separately versioned Zenith build where the license permits.
- [ ] Validate Create and Allthemodium against the pinned engine on single-player and dedicated servers, including offline client joins and world reopen.
- [ ] Pin compatible mod, shader, resource pack, and datapack versions and checksums in each pack manifest.
- [ ] Track compatibility per artifact with verified, port required, and blocked states. Show the reason in the launcher when an artifact cannot be installed for the selected Zenith release.
- [ ] Publish new game and content versions together after compatibility checks. Keep released packs and their required game versions available for existing profiles.
