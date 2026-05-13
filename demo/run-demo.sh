#!/usr/bin/env bash

###############################################################################
# craftcargo end-to-end demo
#
# Demonstrates: crate → debcraft.yaml generation using craftcargo
# Requires: pv (for simulated typing), cargo, craftcargo built
#
# Usage:
#   ./run-demo.sh        # Interactive (press ENTER to advance)
#   ./run-demo.sh -d -n  # Non-interactive, no typing (for CI/testing)
###############################################################################

DEMO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$DEMO_DIR/.." && pwd)"

# Configure BEFORE sourcing demo-magic so TYPE_SPEED is set for pv check
TYPE_SPEED=40

# Source demo-magic (it parses -n, -d, -h flags from $@)
source "$DEMO_DIR/demo-magic.sh"

# Configure prompt
DEMO_PROMPT="${GREEN}craftcargo-demo ${CYAN}\$ ${COLOR_RESET}"

# Ensure we have a built binary
if [ ! -f "$REPO_ROOT/target/release/debcargo" ] && [ ! -f "$REPO_ROOT/target/debug/debcargo" ]; then
  echo "Building craftcargo first..."
  (cd "$REPO_ROOT" && cargo build --release --quiet)
fi

DEBCARGO="$REPO_ROOT/target/release/debcargo"
if [ ! -f "$DEBCARGO" ]; then
  DEBCARGO="$REPO_ROOT/target/debug/debcargo"
fi

# Working directory for demo output (same filesystem as repo to avoid cross-device link errors)
DEMO_WORKDIR=$(mktemp -d "${REPO_ROOT}/.demo-output.XXXXXX")
trap "rm -rf $DEMO_WORKDIR" EXIT

# Helper: only wait if not in no-wait mode
demo_wait() {
  if [ "$NO_WAIT" = false ]; then
    wait
  fi
}

# Helper: pause for readability in recordings (respects NO_WAIT)
demo_pause() {
  local seconds="${1:-1}"
  if [ "$NO_WAIT" = false ]; then
    wait
  else
    sleep "$seconds"
  fi
}

clear

# ═══════════════════════════════════════════════════════════════════════════════
# TITLE
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${COLOR_RESET}"
echo -e "${BOLD}${WHITE}        craftcargo: Rust → Ubuntu Packaging Made Easy${COLOR_RESET}"
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${COLOR_RESET}"
echo ""
echo -e "${GREY}This demo shows how craftcargo generates a complete debcraft.yaml"
echo -e "from a Rust crate, ready for Ubuntu package building.${COLOR_RESET}"
echo ""

demo_pause 2

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 1: Show the tool
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 1: Let's see what craftcargo can do"
demo_pause 2
pe "$DEBCARGO --help"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 2: Pick a crate
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 2: Let's package 'tokio-util' — it has features, multiple deps, and licence complexity"
echo ""
p "# First, what would the Debian source package be called?"
demo_pause 1
pe "$DEBCARGO deb-src-name tokio-util"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 3: Generate debcraft.yaml
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 3: Generate the debcraft.yaml"
demo_pause 1
pe "$DEBCARGO package-debcraft tokio-util --directory $DEMO_WORKDIR/tokio-util"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 4: Inspect the output
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 4: Let's look at what was generated"
demo_pause 1
pe "ls -la $DEMO_WORKDIR/tokio-util/"

demo_pause 2

p "# Here's the debcraft.yaml:"
demo_pause 1
pe "cat $DEMO_WORKDIR/tokio-util/debcraft.yaml"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 5: Show the structure
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 5: Let's look at key sections"
echo ""
p "# The metadata section maps crate info → Debian packaging fields"
demo_pause 2
pe "head -20 $DEMO_WORKDIR/tokio-util/debcraft.yaml"

p "# The parts section defines build steps"
demo_pause 2
pe "grep -A 10 'parts:' $DEMO_WORKDIR/tokio-util/debcraft.yaml"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 6: Show the debcraft integration
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 6: The generated debcraft.yaml is designed for 'debcraft pack'"
p "# debcraft reads the YAML and builds the .deb — one command!"
echo ""
demo_pause 2
pe "cd $DEMO_WORKDIR/tokio-util && debcraft pack 2>&1 || true"

p "# (The cargo plugin is still being implemented upstream in debcraft)"
p "# Once landed, it's: craftcargo generate → debcraft pack → .deb"
demo_pause 2

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 7: Compare with manual approach
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 7: Compare — the traditional debcargo approach generates 100+ files"
demo_pause 2
pe "DEBFULLNAME='Demo User' DEBEMAIL='demo@example.com' $DEBCARGO package tokio-util --directory $DEMO_WORKDIR/tokio-util-traditional 2>&1 | tail -5"

pe "find $DEMO_WORKDIR/tokio-util-traditional -type f | wc -l"
p "# vs our single debcraft.yaml — much simpler to review and maintain!"
demo_pause 3

# ═══════════════════════════════════════════════════════════════════════════════
# WRAP UP
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${COLOR_RESET}"
echo -e "${BOLD}${WHITE}        Demo Complete!${COLOR_RESET}"
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${COLOR_RESET}"
echo ""
echo -e "${WHITE}craftcargo turns complex Rust crates into clean debcraft.yaml files"
echo -e "that are easy to review, version-control, and maintain.${COLOR_RESET}"
echo ""
echo -e "${GREY}Generated output is in: $DEMO_WORKDIR/tokio-util/${COLOR_RESET}"
echo ""
