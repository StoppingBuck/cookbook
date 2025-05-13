#!/bin/bash
# Script to run the cookbook GTK application with debug output
cd /home/mpr/code/cookbook
RUST_BACKTRACE=1 cargo run -p cookbook-gtk
