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
