#!/bin/bash

# Quick pantryman development script - calls the main script from project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$SCRIPT_DIR/../dev.sh" android "$@"
