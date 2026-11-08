#!/usr/bin/env bash
# Promote a verified public engine workflow run without exposing the release signing key to CI.
set -euo pipefail

if [[ $# != 3 || ! "$1" =~ ^[1-9][0-9]*$ || ! "$2" =~ ^[0-9a-f]{40}$ ||
      ! "$3" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)-preview\.[1-9][0-9]*$ ]]; then
  echo "Usage: $0 RUN_ID SOURCE_COMMIT X.Y.Z-preview.N" >&2
  exit 2
fi

run=$1
commit=$2
version=$3
tag=zenith-game@$version
repo=joy-arz/Zenith
root=$(cd "$(dirname "$0")/../.." && pwd -P)
zenith_promote_key=${ZENITH_ENGINE_SIGNING_KEY:?missing Zenith engine signing key}
export -n zenith_promote_key
unset ZENITH_ENGINE_SIGNING_KEY
umask 077
stage=$(mktemp -d)
echo "Staging run $run in $stage"
cd "$root"

gh api "repos/$repo/actions/runs/$run" |
  jq -e --arg repo "$repo" '
    .path == ".github/workflows/zenith-engine-preview.yml" and .head_branch == "main"
    and .head_repository.full_name == $repo
  ' >/dev/null
gh run view "$run" --repo "$repo" --json workflowName,conclusion,event,jobs |
  jq -e '
    .workflowName == "Zenith engine preview" and .conclusion == "success"
    and .event == "workflow_dispatch"
    and ([.jobs[] | select(.name | startswith("build ("))] |
      length == 5 and all(.conclusion == "success"))
    and any(.jobs[]; .name == "verify" and .conclusion == "success")
  ' >/dev/null

[[ "$(gh api "repos/$repo/commits/$commit" --jq .sha)" == "$commit" ]]
gh run download "$run" --repo "$repo" --pattern 'zenith-*' --dir "$stage/artifacts"
dist=$stage/dist
mkdir "$dist"
binaries=(
  zenith-linux-x86_64.tar.gz zenith-server-linux-x86_64.tar.gz
  zenith-macos-arm64.tar.gz zenith-macos-x64.tar.gz
  zenith-windows-arm64.tar.gz zenith-windows-x64.tar.gz
)
for file in "${binaries[@]}"; do
  platform=${file%.tar.gz}
  [[ "$platform" != zenith-server-linux-x86_64 ]] || platform=zenith-linux-x86_64
  mv "$stage/artifacts/$platform/$file" "$dist/"
done
verification=(preview.json zenith-source.tar.gz SHA256SUMS SHA256SUMS.sig ephemeral.pub)
for file in "${verification[@]}"; do
  mv "$stage/artifacts/zenith-preview-verification/$file" "$dist/"
done

jq -e --arg tag "$tag" --arg version "$version" --arg commit "$commit" \
  '.tag == $tag and .version == $version and .source_commit == $commit and .channel == "preview"' \
  "$dist/preview.json" >/dev/null
printf '%s\n' "${binaries[@]}" preview.json zenith-source.tar.gz | LC_ALL=C sort > "$stage/expected"
awk '{print $2}' "$dist/SHA256SUMS" | LC_ALL=C sort > "$stage/actual"
cmp "$stage/expected" "$stage/actual"

verifier=(cargo run --locked --quiet -p horizon-release-sign -- verify --tag "$tag")
"${verifier[@]}" --key "$(cat "$dist/ephemeral.pub")" "$dist"
ZENITH_ENGINE_SIGNING_KEY="$zenith_promote_key" \
  bash tools/ci/sign_zenith_checksums.sh "$tag" "$dist" launcher/zenith-engine-release-keys.txt
"${verifier[@]}" --signature SHA256SUMS.zenith.sig \
  --key "$(cat launcher/zenith-engine-release-keys.txt)" "$dist"

ref="refs/tags/$tag"
ref_sha=$(gh api "repos/$repo/git/matching-refs/tags/$tag" --jq ".[] | select(.ref == \"$ref\") | .object.sha")
if [[ -z "$ref_sha" ]]; then
  gh api --method POST "repos/$repo/git/refs" -f "ref=$ref" -f "sha=$commit" --silent
else
  [[ "$ref_sha" == "$commit" ]]
fi
cat > "$stage/notes.md" <<EOF
Install Zenith Rust Edition $version from the Zenith launcher's Versions tab.
No GitHub account is needed. The launcher verifies the signature before installing.
Minecraft resources are prepared locally and are not bundled with these builds.

The signed release manifest records the source commit, upstream revision and
Minecraft protocol. The Linux dedicated server and the corresponding source
archive are included. [Build and verification](https://github.com/$repo/actions/runs/$run).
EOF
gh release create "$tag" --repo "$repo" --verify-tag --prerelease --latest=false \
  --title "Zenith Rust Edition $version" --notes-file "$stage/notes.md" \
  "$dist"/*.tar.gz "$dist"/preview.json "$dist"/SHA256SUMS "$dist"/SHA256SUMS.sig \
  "$dist"/SHA256SUMS.zenith.sig "$dist"/ephemeral.pub
