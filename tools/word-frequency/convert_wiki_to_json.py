#!/usr/bin/env python3
"""Convert plain-text Wikipedia dump files to JSON and delete originals."""

import json
import os
import sys

SRC = os.path.join(os.path.dirname(__file__), "full-wikipedia-english", "fullEnglish")
DST = os.path.join(os.path.dirname(__file__), "corpus", "wikipedia")


def main():
    # Collect all source files
    files = []
    for dirpath, _, filenames in os.walk(SRC):
        for fname in sorted(filenames):
            files.append(os.path.join(dirpath, fname))

    total = len(files)
    print(f"Converting {total} files...")

    for i, src_path in enumerate(files, 1):
        rel = os.path.relpath(src_path, SRC)
        dst_dir = os.path.join(DST, os.path.dirname(rel))
        dst_path = os.path.join(dst_dir, os.path.basename(rel) + ".json")

        # Read source
        with open(src_path, "r", encoding="utf-8", errors="replace") as f:
            text = f.read()

        # Write JSON
        os.makedirs(dst_dir, exist_ok=True)
        with open(dst_path, "w", encoding="utf-8") as f:
            json.dump({"content": text}, f, ensure_ascii=False)
            f.flush()
            os.fsync(f.fileno())

        # Delete source
        os.remove(src_path)

        if i % 100 == 0 or i == total:
            print(f"  {i}/{total} ({100 * i // total}%)")

    # Remove empty source directories
    for dirpath, dirnames, filenames in os.walk(SRC, topdown=False):
        if not filenames and not dirnames:
            try:
                os.rmdir(dirpath)
            except OSError:
                pass

    print("Done.")


if __name__ == "__main__":
    main()
