#!/usr/bin/env bash
# Project quality gates: format, clippy with warnings as errors, the test
# suite, and the vendored palette's own check. Called by
# .githooks/pre-commit for every commit and by .claude/hooks/check-commit.sh
# before Claude's commits; non-zero exit blocks the commit.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Git exports GIT_DIR and friends to a hook; a child that inherits them acts
# on this repository instead of the one it means to. Drop them, as kyu does.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_PREFIX GIT_COMMON_DIR

# Standing rule 7: a gate that does not predict the build is not a gate.
# cargo rewrites Cargo.lock; anything rewritten AFTER `git add` is green
# here and absent from the commit, so the tree is fingerprinted either side.
gate_tree_fingerprint() {
  { git status --porcelain; git diff; } | sha256sum | cut -d' ' -f1
}
gate_tree_before=$(gate_tree_fingerprint)

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet

# The palette is generated from the kp-themes tokens, never authored here
# [kp-themes scope-128, scope-139]. vendor/kp-themes/ is the tokens.tar of
# the release vendor/PIN names: the first check rebuilds that tar from the
# copy and compares its sha256 with the pin, the second refuses a palette
# that drifted from the tokens.
node gates/check-vendor.mjs
node gates/generate-tui-palette.mjs --check

if [ "$(gate_tree_fingerprint)" != "$gate_tree_before" ]; then
  echo "GATES FAILED — the working tree changed while the gates ran."
  echo "Something rewrote files after they were staged. Re-add and retry."
  exit 1
fi
echo "gates green"
