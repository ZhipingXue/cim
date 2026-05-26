"""
Organize split SEMI standard PDFs into subfolders by their class prefix.

Usage:
    python organize_standards.py [source_dir]

Default:
    source_dir = ../standards_split
"""

import re
import sys
import shutil
from pathlib import Path


def organize(source_dir):
    source_dir = Path(source_dir)
    if not source_dir.exists():
        print(f"Error: source directory not found: {source_dir}")
        sys.exit(1)

    pdf_files = sorted(source_dir.glob("SEMI_*.pdf"))
    print(f"Found {len(pdf_files)} PDF(s) in {source_dir}")

    moved = 0
    for pdf in pdf_files:
        # Extract prefix (leading letters) from standard ID
        # Filename looks like SEMI_E1-0697.pdf or SEMI_PR8-0703.pdf
        m = re.match(r"SEMI_([A-Z]+)", pdf.stem)
        if not m:
            print(f"  Skipping unrecognized file: {pdf.name}")
            continue

        prefix = m.group(1)
        target_dir = source_dir / prefix
        target_dir.mkdir(exist_ok=True)

        target_path = target_dir / pdf.name
        shutil.move(str(pdf), str(target_path))
        moved += 1

    print(f"Done. Moved {moved} file(s) into class folders under {source_dir}")


def main():
    script_dir = Path(__file__).parent.resolve()
    default_source = script_dir.parent / "standards_split"
    source = sys.argv[1] if len(sys.argv) > 1 else str(default_source)
    organize(source)


if __name__ == "__main__":
    main()
