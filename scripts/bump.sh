#!/usr/bin/env sh
# Bump the version of every workspace crate and refresh Cargo.lock.
# The project version lives only in the Cargo workspace; npm packages are
# version-less (all private). Usage: bump.sh <version>
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: bump.sh <version>" >&2
    exit 1
fi

version="$1"
printf '%s' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$' || {
    echo "error: '$version' is not a plain semver (x.y.z[-pre])" >&2
    exit 1
}

for toml in crates/*/Cargo.toml; do
    grep -q '^version = ' "$toml" || {
        echo "error: no '^version = ' line in $toml" >&2
        exit 1
    }
    # Line-anchored on purpose: package version only; dependency versions are
    # never line-anchored (inline tables or `workspace = true` here).
    sed -i.bak "s/^version = \".*\"/version = \"$version\"/" "$toml"
    rm "$toml.bak"
done

# Refresh workspace member versions in Cargo.lock without touching third-party deps.
cargo update --workspace --quiet
