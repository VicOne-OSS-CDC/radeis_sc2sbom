#!/usr/bin/env python3
"""
Convert Juliet Test Suite manifest.xml to .benchmark_truth.tsv.

Usage:
    python3 scripts/juliet_manifest_to_truth.py <juliet_fixture_dir>

Writes <juliet_fixture_dir>/.benchmark_truth.tsv with one row per flaw:
    relative_file_path<TAB>line_number<TAB>cwe_id<TAB>TP

Only CWEs in the scanner's coverage set are emitted (configurable via
COVERED_CWES below). Flaws for CWEs outside coverage are skipped — they
can't be TPs or FPs for our scanner.

The manifest contains bare filenames; this script resolves them to their
relative path under testcases/ by scanning the fixture directory once and
building a basename→relpath index. Files with duplicate basenames (rare)
get all matches emitted.
"""

import sys
import re
import xml.etree.ElementTree as ET
from pathlib import Path
from collections import defaultdict

# CWEs our scanner covers (AST + lexical union, Phase 18)
# All CWEs present in the full Juliet C/C++ corpus (union of all scanner findings)
COVERED_CWES = {
    20, 22, 78, 119, 120, 122, 125, 126, 134, 190, 195, 242, 295, 319, 327,
    362, 367, 369, 377, 398, 401, 404, 415, 416, 457, 467, 476, 477, 561,
    562, 563, 570, 571, 590, 628, 664, 665, 672, 676, 685, 686, 732, 758,
    762, 775, 780, 785, 786, 788, 789, 807, 910,
}

def build_basename_index(fixture: Path) -> dict[str, list[str]]:
    """Map basename → list of relative paths under fixture."""
    index: dict[str, list[str]] = defaultdict(list)
    for p in fixture.rglob("*"):
        if p.suffix in (".c", ".cpp", ".h", ".hpp") and p.is_file():
            index[p.name].append(str(p.relative_to(fixture)))
    return index

def parse_cwe_id(name: str) -> int | None:
    """Extract integer CWE ID from 'CWE-120: Buffer Copy...' string."""
    m = re.match(r"CWE-(\d+)", name)
    return int(m.group(1)) if m else None

def main(fixture_dir: str) -> None:
    fixture = Path(fixture_dir).resolve()
    manifest = fixture / "manifest.xml"
    if not manifest.exists():
        print(f"ERROR: manifest.xml not found at {manifest}", file=sys.stderr)
        sys.exit(1)

    print("Building file index...", file=sys.stderr)
    index = build_basename_index(fixture)

    print("Parsing manifest...", file=sys.stderr)
    tree = ET.parse(manifest)
    root = tree.getroot()

    rows: list[tuple[str, int, int]] = []  # (relpath, line, cwe_id)
    skipped_no_file = 0
    skipped_cwe = 0

    for testcase in root.findall("testcase"):
        for file_el in testcase.findall("file"):
            basename = Path(file_el.get("path", "")).name
            relpaths = index.get(basename, [])
            if not relpaths:
                skipped_no_file += 1
                continue

            for flaw in file_el.findall("flaw"):
                line = int(flaw.get("line", 0))
                cwe_id = parse_cwe_id(flaw.get("name", ""))
                if cwe_id is None or cwe_id not in COVERED_CWES:
                    skipped_cwe += 1
                    continue
                for relpath in relpaths:
                    rows.append((relpath, line, cwe_id))

    out = fixture / ".benchmark_truth.tsv"
    with open(out, "w", encoding="utf-8") as f:
        f.write("# Juliet Test Suite ground truth — generated from manifest.xml\n")
        f.write("# Only CWEs covered by sc2sbom Phase 18 scanner are included.\n")
        f.write("# relative_file_path\tline_number\tcwe_id\tlabel\n")
        for relpath, line, cwe_id in sorted(rows):
            f.write(f"{relpath}\t{line}\t{cwe_id}\tTP\n")

    print(f"Wrote {len(rows):,} TP rows to {out}", file=sys.stderr)
    print(f"Skipped {skipped_no_file} flaws (file not found on disk)", file=sys.stderr)
    print(f"Skipped {skipped_cwe} flaws (CWE not in coverage set)", file=sys.stderr)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <juliet_fixture_dir>", file=sys.stderr)
        sys.exit(1)
    main(sys.argv[1])
