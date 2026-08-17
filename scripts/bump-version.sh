#!/usr/bin/env bash
# Bump the release version everywhere it is recorded, then tag.
#
# The version lives in three files that must agree — Cargo.toml, pyproject.toml
# and uv.lock — and .github/workflows/release.yml refuses to publish when they
# drift from the tag. Editing them by hand is how that drift happens, so do it
# here instead.
#
#   scripts/bump-version.sh 0.3.1        # rewrite, commit and tag
#   scripts/bump-version.sh 0.3.1 --dry  # show what would change
#
# Pushing the tag is left to you, and is what actually starts the release:
#   git push origin main --follow-tags

set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"

version="${1:-}"
dry_run="${2:-}"

if [[ -z "$version" ]]; then
  echo "usage: scripts/bump-version.sh <version> [--dry]" >&2
  exit 2
fi
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.]+)?$ ]]; then
  echo "error: '$version' is not a version PyPI will accept" >&2
  exit 2
fi

current=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
if [[ "$version" == "$current" ]]; then
  echo "error: already at $current" >&2
  exit 2
fi
if git rev-parse -q --verify "refs/tags/v${version}" >/dev/null; then
  echo "error: tag v${version} already exists" >&2
  exit 2
fi
if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree is dirty — commit or stash first" >&2
  exit 2
fi

echo "$current -> $version"

if [[ "$dry_run" == "--dry" ]]; then
  echo "(dry run, nothing written)"
  exit 0
fi

# Only the first `version = ` of each manifest is ours; the rest belong to
# dependencies, hence the line-range anchoring rather than a blanket -i.
sed -i "0,/^version = \"${current}\"/s//version = \"${version}\"/" Cargo.toml
sed -i "0,/^version = \"${current}\"/s//version = \"${version}\"/" pyproject.toml

# Both lockfiles pin this package, and CI builds with --locked, so leaving
# them behind turns every release into a lockfile-out-of-date failure.
#
# Patched in place rather than through `uv lock` / `cargo update`: the only
# thing changing is our own version, no dependency needs re-resolving. Those
# commands would reach for the network (uv lock hangs without it) and cargo
# would additionally need a toolchain new enough to parse the v4 lockfile.
python3 - "$version" <<'PY'
import re
import sys

version = sys.argv[1]
for path in ("Cargo.lock", "uv.lock"):
    text = open(path, encoding="utf-8").read()
    patched, count = re.subn(
        r'(name = "donglao-g2p"\nversion = ")[^"]+(")',
        lambda m: m.group(1) + version + m.group(2),
        text,
        count=1,
    )
    if count != 1:
        raise SystemExit(f"{path}: could not find the donglao-g2p stanza")
    open(path, "w", encoding="utf-8").write(patched)
PY

for file in Cargo.toml pyproject.toml; do
  found=$(grep -m1 '^version = ' "$file" | cut -d'"' -f2)
  if [[ "$found" != "$version" ]]; then
    echo "error: $file still reads $found" >&2
    exit 1
  fi
done
for file in Cargo.lock uv.lock; do
  found=$(grep -A1 'name = "donglao-g2p"' "$file" | grep -m1 '^version = ' | cut -d'"' -f2)
  if [[ "$found" != "$version" ]]; then
    echo "error: $file still reads $found" >&2
    exit 1
  fi
done

git add -A
git commit -m "release ${version}"
git tag -a "v${version}" -m "v${version}"

echo
echo "committed and tagged v${version}. To release:"
echo "  git push origin $(git branch --show-current) --follow-tags"
