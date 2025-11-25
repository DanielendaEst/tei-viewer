#!/usr/bin/env python3
"""
Fix Tractatus TEI coordinates and image references.

This script:
1. Changes image URL from images/p004.jpg to images/p1.jpg
2. Updates graphic dimensions from 2479×3508 to 960×1358 (actual image size)
3. Scales all zone coordinates from old dimensions to new dimensions
"""

import re
import sys
from pathlib import Path

# Old dimensions (declared in XML)
OLD_WIDTH = 2479
OLD_HEIGHT = 3508

# New dimensions (actual image)
NEW_WIDTH = 960
NEW_HEIGHT = 1358

# Scale factors
SCALE_X = NEW_WIDTH / OLD_WIDTH
SCALE_Y = NEW_HEIGHT / OLD_HEIGHT

print(f"Scaling coordinates from {OLD_WIDTH}×{OLD_HEIGHT} to {NEW_WIDTH}×{NEW_HEIGHT}")
print(f"Scale factors: X={SCALE_X:.4f}, Y={SCALE_Y:.4f}")
print()


def scale_point(x, y):
    """Scale a single coordinate point."""
    new_x = int(round(x * SCALE_X))
    new_y = int(round(y * SCALE_Y))
    return new_x, new_y


def scale_points_string(points_str):
    """Scale a points attribute value."""
    coords = []
    pairs = points_str.strip().split()

    for pair in pairs:
        if "," not in pair:
            continue
        x_str, y_str = pair.split(",")
        try:
            x = float(x_str)
            y = float(y_str)
            new_x, new_y = scale_point(x, y)
            coords.append(f"{new_x},{new_y}")
        except ValueError:
            print(f"Warning: Could not parse point '{pair}'")
            coords.append(pair)

    return " ".join(coords)


def fix_graphic_element(line):
    """Fix the <graphic> element."""
    # Change image URL
    line = line.replace("images/p004.jpg", "images/p1.jpg")

    # Update dimensions
    line = re.sub(r'width="2479px"', f'width="{NEW_WIDTH}px"', line)
    line = re.sub(r'height="3508px"', f'height="{NEW_HEIGHT}px"', line)

    return line


def fix_zone_element(line):
    """Fix zone coordinates."""
    match = re.search(r'points="([^"]+)"', line)
    if match:
        old_points = match.group(1)
        new_points = scale_points_string(old_points)
        line = line.replace(f'points="{old_points}"', f'points="{new_points}"')
    return line


def process_file(input_path, output_path):
    """Process a TEI file."""
    print(f"Processing: {input_path}")

    with open(input_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    zones_fixed = 0
    graphic_fixed = False

    for i, line in enumerate(lines):
        # Fix graphic element
        if "<graphic" in line and "p004.jpg" in line:
            lines[i] = fix_graphic_element(line)
            graphic_fixed = True

        # Fix zone elements
        if "<zone" in line and "points=" in line:
            lines[i] = fix_zone_element(line)
            zones_fixed += 1

    # Write output
    with open(output_path, "w", encoding="utf-8") as f:
        f.writelines(lines)

    print(f"  ✓ Graphic element updated: {graphic_fixed}")
    print(f"  ✓ Zones rescaled: {zones_fixed}")
    print(f"  ✓ Saved to: {output_path}")
    print()


def main():
    # Process both diplomatic and translation files
    base_dir = Path(__file__).parent / "projects" / "Tractatus-Fascinatione"

    if not base_dir.exists():
        print(f"Error: Directory not found: {base_dir}")
        sys.exit(1)

    files_to_process = [
        ("p1_dip.xml", "p1_dip.xml"),
        ("p1_trad.xml", "p1_trad.xml"),
    ]

    for input_name, output_name in files_to_process:
        input_path = base_dir / input_name
        output_path = base_dir / output_name

        if not input_path.exists():
            print(f"Warning: File not found: {input_path}")
            continue

        process_file(input_path, output_path)

    print("=" * 60)
    print("✅ All files processed successfully!")
    print()
    print("Next steps:")
    print("  1. Run: ./sync_projects.sh")
    print("  2. Run: trunk build")
    print("  3. Test the Tractatus project in the viewer")
    print()


if __name__ == "__main__":
    main()
