# Bump version, update changelog, commit and tag
# Usage: just bump [version]  (version optional — auto-detected by git-cliff)
[group('release')]
bump version="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{ version }}" ]; then
        NEXT="{{ version }}"
    else
        NEXT=$(git-cliff --bump --unreleased -o /dev/stdout 2>/dev/null | grep -m1 '^## ' | sed -E 's/^## \[([^]]+)\].*/\1/')
    fi
    CURRENT=$(cargo metadata --no-deps --format-version 1 | jq -r '.workspace_packages[0].version' 2>/dev/null \
        || grep -m1 '^version' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
    if [ "$NEXT" = "$CURRENT" ]; then
        echo "Version is already $CURRENT — nothing to bump" >&2
        exit 1
    fi
    echo "Bumping $CURRENT -> $NEXT"
    git-cliff --bump -o CHANGELOG.md
    sed -i '' -E "s/^version = \"$CURRENT\"/version = \"$NEXT\"/" Cargo.toml
    cargo update -w --offline
    git add CHANGELOG.md Cargo.toml Cargo.lock
    git commit -m "chore(release): prepare for v$NEXT"
    git tag "v$NEXT"
    echo "Done: v$NEXT (not pushed)"

# Dry-run publish checks for all publishable crates
[group('release')]
dry:
    cargo publish --dry-run -p baza_core
    cargo publish --dry-run -p baza

# Publish to crates.io: core first, then baza
[group('release')]
publish:
    cargo publish -p baza_core
    @echo "Waiting for baza_core to be indexed..."
    @for i in $(seq 1 12); do sleep 10; cargo search baza_core --limit 1 | grep -q "^baza_core " && break; done
    cargo publish -p baza

# Full release cycle: bump (or verify tag) -> dry -> publish
[group('release')]
release version="": (bump version) dry publish
