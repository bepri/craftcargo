#!/usr/bin/env bash

###############################################################################
# Record an asciinema cast of the craftcargo demo
#
# Usage:
#   ./record-cast.sh              # Records demo/craftcargo-demo.cast
#   ./record-cast.sh output.cast  # Records to custom path
###############################################################################

DEMO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT="${1:-$DEMO_DIR/craftcargo-demo.cast}"

# Ensure binary is built
REPO_ROOT="$(cd "$DEMO_DIR/.." && pwd)"
if [ ! -f "$REPO_ROOT/target/release/debcargo" ] && [ ! -f "$REPO_ROOT/target/debug/debcargo" ]; then
  echo "Building craftcargo..."
  (cd "$REPO_ROOT" && cargo build --release --quiet)
fi

echo "Recording asciinema cast to: $OUTPUT"
echo "Running demo in non-interactive mode with simulated timing..."
echo ""

# Record the demo in non-interactive mode (-n = no wait for ENTER)
# We keep -d off so the typing simulation shows, but if pv is missing use -d
if command -v pv &>/dev/null; then
  asciinema rec "$OUTPUT" \
    --cols 100 \
    --rows 30 \
    --title "craftcargo: Rust → Ubuntu Packaging Made Easy" \
    --command "bash $DEMO_DIR/run-demo.sh -n"
else
  echo "Note: 'pv' not installed, running without typing simulation"
  asciinema rec "$OUTPUT" \
    --cols 100 \
    --rows 30 \
    --title "craftcargo: Rust → Ubuntu Packaging Made Easy" \
    --command "bash $DEMO_DIR/run-demo.sh -d -n"
fi

echo ""
echo "Cast recorded: $OUTPUT"
echo ""
echo "To play back:  asciinema play $OUTPUT"
echo "To upload:     asciinema upload $OUTPUT"
echo "To embed:      use <script> tag from asciinema.org after upload"
