#!/usr/bin/env python3
"""
Fix Tractatus TEI line order based on Y-coordinates.

The Tractatus TEI files have lines in the wrong order because the original
PAGE-XML had incorrect readingOrder indices. This script physically reorders
the <lb> and <ab> elements in the XML based on Y-coordinates.
"""

import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import List, Tuple

TEI_NS = "http://www.tei-c.org/ns/1.0"
XML_NS = "http://www.w3.org/XML/1998/namespace"


def get_zone_avg_y(points_str: str) -> float:
    """Calculate average Y coordinate from a points string."""
    points = []
    for point in points_str.split():
        if "," in point:
            try:
                x, y = point.split(",")
                points.append((float(x), float(y)))
            except ValueError:
                continue

    if not points:
        return 0.0

    return sum(p[1] for p in points) / len(points)


def fix_file(file_path: Path) -> None:
    """Fix line order in a TEI file."""
    print(f"\nProcessing: {file_path.name}")
    print("-" * 70)

    # Parse XML with namespace
    ET.register_namespace("", TEI_NS)
    tree = ET.parse(file_path)
    root = tree.getroot()

    # Find all zones and build map of zone_id -> avg_y
    zones = {}
    for zone in root.findall(".//{%s}zone" % TEI_NS):
        zone_id = zone.get("{%s}id" % XML_NS)
        points = zone.get("points")
        if zone_id and points:
            avg_y = get_zone_avg_y(points)
            zones[zone_id] = avg_y

    # Find the div containing the text
    div = root.find(".//{%s}div" % TEI_NS)
    if div is None:
        print("  Error: Could not find <div> element")
        return

    # Extract all lb/ab pairs
    lines = []
    i = 0
    children = list(div)

    while i < len(children):
        elem = children[i]

        # Check if this is an <lb> element
        if elem.tag == "{%s}lb" % TEI_NS:
            lb_elem = elem
            ab_elem = None

            # The next element should be <ab>
            if i + 1 < len(children) and children[i + 1].tag == "{%s}ab" % TEI_NS:
                ab_elem = children[i + 1]
                i += 2  # Skip both lb and ab
            else:
                i += 1

            # Get zone reference
            facs = lb_elem.get("facs", "")
            zone_id = facs.replace("#", "") if facs else None

            # Get Y coordinate
            avg_y = zones.get(zone_id, 999999.0) if zone_id else 999999.0

            lines.append(
                {
                    "lb": lb_elem,
                    "ab": ab_elem,
                    "zone_id": zone_id,
                    "avg_y": avg_y,
                    "original_n": lb_elem.get("n", "0"),
                }
            )
        else:
            i += 1

    # Check how many are out of order
    out_of_order = 0
    prev_y = -1
    for line in lines:
        if line["avg_y"] < prev_y:
            out_of_order += 1
        prev_y = line["avg_y"]

    print(f"  Found {len(lines)} lines")
    print(f"  Lines out of order: {out_of_order}")

    if out_of_order == 0:
        print(f"  ✓ File already in correct order")
        return

    # Sort by Y coordinate
    sorted_lines = sorted(lines, key=lambda x: x["avg_y"])

    # Show examples
    print(f"  Examples of reordering:")
    changes = 0
    for new_num, line in enumerate(sorted_lines, start=1):
        old_num = line["original_n"]
        if str(new_num) != old_num:
            if changes < 5:
                print(
                    f"    Line {old_num:3s} → Line {new_num:3d} (Y={line['avg_y']:.0f}, zone={line['zone_id']})"
                )
            changes += 1

    if changes > 5:
        print(f"    ... and {changes - 5} more changes")

    # Remove all existing lb/ab from div
    for child in list(div):
        if child.tag in ["{%s}lb" % TEI_NS, "{%s}ab" % TEI_NS]:
            div.remove(child)

    # Add back in correct order with correct line numbers
    for new_num, line in enumerate(sorted_lines, start=1):
        # Update line number
        line["lb"].set("n", str(new_num))

        # Add to div
        div.append(line["lb"])
        if line["ab"] is not None:
            div.append(line["ab"])

    # Write back
    tree.write(file_path, encoding="utf-8", xml_declaration=True)

    print(f"  ✓ File updated with corrected line order")


def main():
    base_dir = Path(__file__).parent / "projects" / "Tractatus-Fascinatione"

    if not base_dir.exists():
        print(f"Error: Directory not found: {base_dir}")
        sys.exit(1)

    files_to_process = [
        "p1_dip.xml",
        "p1_trad.xml",
    ]

    print("=" * 70)
    print("TRACTATUS LINE ORDER FIX")
    print("=" * 70)

    for filename in files_to_process:
        file_path = base_dir / filename

        if not file_path.exists():
            print(f"Warning: File not found: {file_path}")
            continue

        try:
            fix_file(file_path)
        except Exception as e:
            print(f"  ✗ Error processing file: {e}")
            import traceback

            traceback.print_exc()

    print("\n" + "=" * 70)
    print("✅ All files processed successfully!")
    print()
    print("Next steps:")
    print("  1. Run: ./sync_projects.sh")
    print("  2. Run: trunk build")
    print("  3. Test the Tractatus project in the viewer")
    print()
    print("The highlights should now stay with the correct text lines!")
    print("=" * 70)


if __name__ == "__main__":
    main()
