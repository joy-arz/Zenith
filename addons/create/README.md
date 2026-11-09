# Create on Zenith 1142b362f

This source patch adapts the native Create 0.8.0-alpha.16 development bundle
for the Zenith engine pinned by the ATM10 8.5 preview.

| Component | Git commit |
| --- | --- |
| Zenith engine | `1142b362f364c7ee5dea43dc1600905e2a3fe138` |
| Create Horizon | `c92212b043d542b3d35f25587000ab5af37e6a28` |
| Horizon Mixins | `e8cd0aa38cffef49cb2e088ef716baa107148c02` |

Clone Create Horizon from its original repository. Check out the exact commit
above and apply the patch in that checkout.

```sh
git clone https://github.com/Delete228/create-horizon.git
cd create-horizon
git switch --detach c92212b043d542b3d35f25587000ab5af37e6a28
git apply --check /absolute/path/to/zenith-1142b362f.patch
git apply /absolute/path/to/zenith-1142b362f.patch
```

Use the resulting `mixins/create` source with the pinned Zenith engine and
Mixins. For direct Mixins builds, stage the engine from the pinned source
archive, write the engine commit to `.horizon-revision`, and remove the
`CLAUDE.md` documentation symlink from the staging copy. Mixins rejects
symlinks. Preserve the upstream Create LICENSE and third-party notices.

In the combined preview, the motor and Mechanical Press made two iron sheets.
The depot retained both on save reopen. The patch also includes the pure
Rust Mechanical Crafter pattern matcher. Its world traversal, in-game recipe
registration and output transfer are separate work.

## Zenith Rust Edition 0.2.0-preview.1

Keep the first patch for the original ATM10 preview. To target the newer
Zenith engine at `a5609804433041da96cd8fde34b322dd6593b905`, apply
[`zenith-0.2.0-preview.1.patch`](zenith-0.2.0-preview.1.patch) after it.
The follow-up retargets the manifests and uses the engine's item-spawn
function for armor dropped during horse haunting. The old motion helper was
removed when the engine moved spawn motion to the item entity's own random
number generator.

## Mechanical Crafter chain and recipe input

Apply [the chain patch](zenith-mechanical-crafter-chain.patch) after the base
adapter. It adds a Rust routine that follows crafter arrows to the output,
collects all connected inputs on the same face, and assembles the recipe
grid. The tests cover branches, gaps, slot covers, redstone starts, cycles,
and unloaded neighbors. Its reference is
[Create's 1.21.1 recipe grid](https://github.com/Creators-of-Create/Create/blob/4148fd8d5b05c85d88947edc00fb495a5074f57d/src/main/java/com/simibubi/create/content/kinetics/crafter/RecipeGridHandler.java).

The world adapter still needs to register the machine, supply the cells,
run the crafting phases, and send the result to a belt or inventory.

## Zenith Rust Edition 0.2.0-preview.5

Apply [the engine source patch](zenith-engine-0.2.0-preview.5.patch) to a
staging copy of the signed engine source at `b73a23d331`. Apply
[the Create adapter](zenith-0.2.0-preview.5.patch) after the earlier three
Create patches. Use Mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.

This restores the asset overlay on pack reload and reconnects the transmission
tick after Horizon moved it out of `game.rs`. All 95 injections planned and
applied, and the transformed client built on macOS ARM64. A motor drove the
press to produce two iron sheets, and the depot retained both on save reopen.
Use the player's own Create JAR for the original assets.

## Zenith Rust Edition 0.2.0-preview.6

On the signed engine source at `b9f3bd4502`, the resource-root change is already
present. Apply [the Preview 6 adapter](zenith-0.2.0-preview.6.patch) after the
base, Preview 1 and Mechanical Crafter chain patches. Mixins is unchanged.
All 95 injections planned and applied, and the transformed client type-checked.
