# Zenith

Maintained by [joy-arz](https://github.com/joy-arz).

# Offline accounts and self-hosted servers

This fork adds ordinary offline access to the Horizon launcher and invitations for servers hosted on your own computer. Microsoft sign-in is optional for offline play. Game resources still come from your own Minecraft installation. Authenticated downloads and online-mode servers keep their existing authentication requirements.

## Use the launcher

1. Install and prepare a Horizon game version using your existing Minecraft files.
2. Open **Settings → Account**. Add an offline name (3–16 letters, digits or underscores) and select **Play as**. **Offline access** is visible in Settings and enabled by default.
3. Select **Custom skin… → Choose a skin…** beside that name. Import a 64×64 or legacy 64×32 PNG and choose Classic or Slim arms. The launcher stores a copy per profile and uses it at the next game launch. This existing skin feature renders your own player locally; it does not distribute the PNG to other players or vanilla clients.
4. Open **Servers → Create a server**. New servers allow offline accounts by default. List your friends' offline usernames. The first listed player is the operator. Accept the EULA and create the server.
5. Change the local game port in the server's **Settings** if needed, then start the server. Existing servers retain their settings: disable **account checks** to let offline accounts join.
6. Give the host computer a stable LAN address. Forward your chosen **external TCP port** on your router to this computer's **local game port**. Allow that local game port through the host firewall. Keep management, panel and RCON ports private.
7. Under **Own port forwarding**, save your public IP or DNS name with its external port. Use **Copy server link**. Friends paste that link into this fork's **Multiplayer → Add Server** address field. Other clients can use the ordinary `host:port` address.

The share link is `zenith://server/HOST:PORT`. It is a pasteable invitation, not a hosted webpage or a registered browser protocol. The launcher does not configure your router, discover your public IP, or claim that the port is reachable. CGNAT requires an ISP-provided public IP or your own TCP tunnel. Update the saved address when your public IP changes, or use dynamic DNS.

Offline names are not authenticated. A whitelist limits names, but cannot prove who owns one. Share the endpoint with trusted friends and keep backups.

## Command line

After building the launcher, use its executable as `horizon-launcher` below:

```sh
horizon-launcher account offline Builder_1
horizon-launcher account skin Builder_1 /absolute/path/to/skin.png --model slim
horizon-launcher play

horizon-launcher servers create --name "Friends" --port 25565 --offline \
  --player Builder_1 --player Friend_2 --accept-eula
horizon-launcher servers run "Friends" -- start
horizon-launcher servers share "Friends" --address play.example.com:25570
horizon-launcher play --server zenith://server/play.example.com:25570
```

`play.example.com:25570` is an example: replace it with your actual public address and external port. This example forwards external TCP 25570 to local TCP 25565. `servers share` without `--address` prints the saved invitation; `--json` returns the address and link. No relay, Tailscale login or Microsoft account is required for this connection.

`servers create --check-accounts` explicitly creates an online-mode server. `settings offline-access off` restores the launcher's account requirement for offline profiles; `settings offline-access on` enables offline access again. Existing server authentication settings are never migrated automatically.

## Build and verification

The launcher is a separate Cargo workspace:

```sh
cargo build --release --manifest-path launcher/Cargo.toml
```

On a Mac without the Metal compiler, append `--features gpui_platform/runtime_shaders`.

Release builds check this fork's launcher releases from inside the app. Use the app's Update action to install and restart; the existing updater verifies the archive checksum and this fork's Ed25519 signature before replacement. Development builds keep self-update disabled. Game installation and updates retain upstream releases and their separate signing key. No Minecraft resources are included in the repository.

Focused checks:

```sh
cargo test --manifest-path launcher/Cargo.toml --lib server_share::tests
cargo test --manifest-path launcher/Cargo.toml --lib skins::tests
python3 tools/pr_ready.py --base upstream/main --quick
```

A live game and an external connection still require your prepared resources, router mapping, public address and a second network. Unit tests cannot establish internet reachability.

## Release through GitHub Actions

The **Zenith release** workflow builds Linux x86-64, Windows x64 and ARM64, macOS Apple Silicon and macOS Intel on GitHub-hosted runners. It runs launcher tests, packages the app, signs `SHA256SUMS`, verifies the signature against `launcher/release-keys.txt`, and publishes the matching archives. The signing key is stored only in the fork's `FORK_RELEASE_SIGNING_KEY` Actions secret. The upstream launcher release workflow is gated to the upstream repository.

Run **Actions → Zenith release → Run workflow**, select the implementation branch, and enter a new three-part version such as `0.3.1`. Alternatively, push a tag such as `launcher@0.3.1` on the desired commit. Use increasing versions. The workflow refuses to overwrite an existing release. Once installed initially, release builds obtain later launcher versions through the in-app Update action.

Automatic registry updates retain a content mod's enabled or disabled state. Automatic game updates mark their Nightly installations for retention cleanup; manual and legacy installs, selected and instance-pinned versions, and running versions keep their protections. Source review and tests: [UPSTREAM_REVIEW.md](UPSTREAM_REVIEW.md).
