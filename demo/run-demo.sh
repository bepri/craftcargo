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

p "# Step 2: Let's package 'bon' — a popular builder-pattern macro crate"
echo ""
p "# First, what would the Debian source package be called?"
demo_pause 1
pe "$DEBCARGO deb-src-name bon"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 3: Generate debcraft.yaml
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 3: Generate the debcraft.yaml"
demo_pause 1
pe "$DEBCARGO package-debcraft bon --directory $DEMO_WORKDIR/bon"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 4: Inspect the output
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 4: Let's look at what was generated"
demo_pause 1
pe "ls $DEMO_WORKDIR/bon/"

demo_pause 1

p "# Here's the debcraft.yaml:"
demo_pause 1
pe "cat $DEMO_WORKDIR/bon/debcraft.yaml"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 5: Pack it into .deb files with debcraft
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 5: Now let's build the .deb packages — one command!"
demo_pause 2
pe "cd $DEMO_WORKDIR/bon && debcraft pack"

demo_pause 2

p "# Look at all those .deb files!"
demo_pause 1
pe "ls -lh $DEMO_WORKDIR/bon/*.deb"

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 6: Inspect the package
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 6: Let's inspect what we built"
demo_pause 2
pe "dpkg-deb --info $DEMO_WORKDIR/bon/librust-bon-dev_*.deb"

demo_pause 2

p "# Ready to install with: sudo dpkg -i librust-bon-dev_*.deb"
demo_pause 2

# ═══════════════════════════════════════════════════════════════════════════════
# STEP 7: Compare with manual approach
# ═══════════════════════════════════════════════════════════════════════════════

p "# Step 7: Compare — the traditional debcargo approach generates many more files"
demo_pause 2
pe "DEBFULLNAME='Demo User' DEBEMAIL='demo@example.com' $DEBCARGO package bon --directory $DEMO_WORKDIR/bon-traditional 2>&1 | tail -5"

pe "find $DEMO_WORKDIR/bon-traditional -type f | wc -l"
p "# vs our single debcraft.yaml — from crate to installed .deb in seconds!"
demo_pause 3

# ═══════════════════════════════════════════════════════════════════════════════
# WRAP UP
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${COLOR_RESET}"
echo -e "${BOLD}${WHITE}        Demo Complete!${COLOR_RESET}"
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${COLOR_RESET}"
echo ""
echo -e "${WHITE}craftcargo turns Rust crates into installable .deb packages"
echo -e "via a single, reviewable debcraft.yaml file.${COLOR_RESET}"
echo ""
echo -e "${GREY}Generated output is in: $DEMO_WORKDIR/bon/${COLOR_RESET}"
echo ""
