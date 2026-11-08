#!/usr/bin/env bash
# Sign the exact checksum list verified by the Zenith engine workflow.
set -euo pipefail
tag=$1
directory=$2
expected_key=$3
openssl=${OPENSSL:-openssl}
umask 077
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

printf '%s' "${ZENITH_ENGINE_SIGNING_KEY:?missing Zenith engine signing key}" |
  "$openssl" base64 -d -A > "$work/key"
actual_key=$("$openssl" pkey -inform DER -in "$work/key" -pubout -outform DER |
  tail -c 32 | "$openssl" base64 -A)
[[ "$actual_key" == "$(tr -d '\r\n' < "$expected_key")" ]]
{ printf 'horizon release %s\n' "$tag"; cat "$directory/SHA256SUMS"; } > "$work/message"
"$openssl" pkeyutl -sign -rawin -inkey "$work/key" -keyform DER \
  -in "$work/message" -out "$work/signature"
"$openssl" base64 -A -in "$work/signature" > "$directory/SHA256SUMS.zenith.sig"
printf '\n' >> "$directory/SHA256SUMS.zenith.sig"
