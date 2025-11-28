#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Use the first argument as the project directory, or default to the current directory
PROJECT_DIR="${1:-.}"

# Define the output file with a timestamp
OUTPUT_FILE="voronoi_vivarium_primer_$(date +%Y%m%d_%H%M%S).txt"

# --- Header ---
echo "--- VORONOI VIVARIUM PRIMER ---" > "$OUTPUT_FILE"
echo "Generated on: $(date)" >> "$OUTPUT_FILE"
echo "Project Directory: $(realpath "$PROJECT_DIR")" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# --- Git Status ---
echo "--- START OF GIT STATUS ---" >> "$OUTPUT_FILE"
(cd "$PROJECT_DIR" && git status) >> "$OUTPUT_FILE"
echo "--- END OF GIT STATUS ---" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# --- Git Log (Last 5 Commits) ---
echo "--- START OF GIT LOG ---" >> "$OUTPUT_FILE"
(cd "$PROJECT_DIR" && git log -n 5 --oneline --graph) >> "$OUTPUT_FILE"
echo "--- END OF GIT LOG ---" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# --- Directory Tree ---
echo "--- START OF DIRECTORY TREE ---" >> "$OUTPUT_FILE"
(cd "$PROJECT_DIR" && tree -L 3 -I 'target') >> "$OUTPUT_FILE"
echo "--- END OF DIRECTORY TREE ---" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# --- Key File Contents ---
# Array of key files to include in the primer for the Voronoi Vivarium project
KEY_FILES=(
    "README.md"
    ".gitignore"
    "Cargo.toml"
    "src/main.rs"
    "src/state.rs"
    "src/ui.rs"
    "src/chemistry.rs"
    "src/voronoi.rs"
    "index.html"
    ".github/workflows/deploy_pages.yml"
    "voronoi-vivarium-primer.sh"
)

for file in "${KEY_FILES[@]}"; do
    FILE_PATH="$PROJECT_DIR/$file"
    if [ -f "$FILE_PATH" ]; then
        echo "--- START OF FILE: $file ---" >> "$OUTPUT_FILE"
        cat "$FILE_PATH" >> "$OUTPUT_FILE"
        echo "--- END OF FILE: $file ---" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    else
        echo "--- WARNING: FILE NOT FOUND: $file ---" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    fi
done

echo "Voronoi Vivarium primer created successfully at: $OUTPUT_FILE"
