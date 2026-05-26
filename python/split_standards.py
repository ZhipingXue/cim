"""
Split a compiled SEMI standards PDF into individual standard PDFs.

Requires: pip install pypdf

Usage:
    python split_standards.py [input_pdf] [output_dir]

Defaults:
    input_pdf = ../standard.pdf  (relative to this script's directory)
    output_dir = ../standards_split

Note: For a ~8000 page PDF this script may take 15-20 minutes to complete
because it extracts text from every page to detect standard boundaries.
"""

import re
import sys
import os
from pathlib import Path

try:
    from pypdf import PdfReader, PdfWriter
except ImportError:
    print("Error: pypdf is not installed. Run: pip install pypdf")
    sys.exit(1)


# Regex to match standard identifiers like:
#   SEMI E1-0697
#   SEMI E1.1-0697
#   SEMI F3-94
#   SEMI PR8-0703
#   SEMI HB1-0305
#   SEMI C3.22-1000
STD_PATTERN = re.compile(
    r"SEMI\s+([A-Z]+\d+(?:\.\d+)?-\d+)",
    re.MULTILINE,
)


def detect_standard_id(page):
    """Extract standard ID from a PDF page."""
    text = page.extract_text()
    if not text:
        return None
    # Only search first 800 chars — header is always at top of page
    m = STD_PATTERN.search(text[:800])
    return m.group(1) if m else None


def split_pdf(input_path, output_dir):
    input_path = Path(input_path)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Reading {input_path} ...", flush=True)
    reader = PdfReader(str(input_path))
    total = len(reader.pages)
    print(f"Total pages: {total}", flush=True)

    # First pass: detect raw standard ID for every page
    print("Scanning pages for standard identifiers ...", flush=True)
    raw_ids = []
    for i in range(total):
        sid = detect_standard_id(reader.pages[i])
        raw_ids.append(sid)
        if (i + 1) % 500 == 0 or i == total - 1:
            print(f"  scanned {i + 1}/{total} pages", flush=True)

    # Second pass: resolve unknown pages
    # A page with no ID inherits the nearest known neighbour.
    # We do a simple forward-fill then backward-fill.
    resolved_ids = list(raw_ids)

    # Forward fill
    current = None
    for i in range(total):
        if resolved_ids[i] is not None:
            current = resolved_ids[i]
        else:
            resolved_ids[i] = current

    # Backward fill for leading unknown pages
    current = None
    for i in range(total - 1, -1, -1):
        if raw_ids[i] is not None:
            current = raw_ids[i]
        elif resolved_ids[i] is None:
            resolved_ids[i] = current

    # Group consecutive pages by resolved standard ID
    groups = []
    start = 0
    cur_id = resolved_ids[0]
    for i in range(1, total):
        if resolved_ids[i] != cur_id:
            groups.append((cur_id, start, i - 1))
            cur_id = resolved_ids[i]
            start = i
    groups.append((cur_id, start, total - 1))

    print(f"\nDetected {len(groups)} standard segment(s).", flush=True)

    # Write each group
    written = 0
    for std_id, pg_start, pg_end in groups:
        if std_id is None:
            print(f"  Skipping unknown pages {pg_start + 1}-{pg_end + 1}", flush=True)
            continue

        writer = PdfWriter()
        for i in range(pg_start, pg_end + 1):
            writer.add_page(reader.pages[i])

        # Sanitize filename
        safe_name = re.sub(r"[^A-Za-z0-9.\-]", "_", std_id)
        out_file = output_dir / f"SEMI_{safe_name}.pdf"

        with open(out_file, "wb") as f:
            writer.write(f)

        page_count = pg_end - pg_start + 1
        print(f"  SEMI {std_id}: pages {pg_start + 1}-{pg_end + 1} ({page_count} pp) -> {out_file.name}", flush=True)
        written += 1

    print(f"\nDone. Wrote {written} standard PDF(s) to {output_dir}", flush=True)


def main():
    script_dir = Path(__file__).parent.resolve()
    default_input = script_dir.parent / "standard.pdf"
    default_output = script_dir.parent / "standards_split"

    input_pdf = sys.argv[1] if len(sys.argv) > 1 else str(default_input)
    output_dir = sys.argv[2] if len(sys.argv) > 2 else str(default_output)

    if not os.path.exists(input_pdf):
        print(f"Error: input file not found: {input_pdf}")
        sys.exit(1)

    split_pdf(input_pdf, output_dir)


if __name__ == "__main__":
    main()
