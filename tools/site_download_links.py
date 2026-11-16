"""Moves the published site's download links onto a release tag.

    python tools/site_download_links.py <site-folder> <tag>

Every `releases/download/<tag>/…` and `releases/tag/<tag>` in the site's HTML is moved to the new
tag, and so is the same tag written in the page text, so a new release leaves the buttons pointing
at its own files. Prints the files it changed. Exits 1 when no page references a tag at all, which
means the pages changed shape and this script needs updating.
"""
import pathlib
import re
import sys

TAG = re.compile(r"v\d[^/\"'<>\s]*")


def retag(text, tag):
    """The text with every older tag replaced by `tag`; nothing to do leaves it untouched."""
    for old in sorted(set(TAG.findall(text)), key=len, reverse=True):
        if old != tag:
            text = text.replace(old, tag)
            # The same version without its `v`, as the terms page writes it.
            text = text.replace(old[1:], tag[1:])
    return text


def main(folder, tag):
    root = pathlib.Path(folder)
    files = sorted(root.glob("*.html")) + sorted(root.glob("pages/*.html"))
    changed = 0
    referencing = 0
    for path in files:
        text = path.read_text(encoding="utf-8")
        if TAG.search(text):
            referencing += 1
        new = retag(text, tag)
        if new != text:
            path.write_text(new, encoding="utf-8")
            print(f"retagged {path}")
            changed += 1
    print(f"{changed} file(s) changed, {referencing} of {len(files)} reference a tag")
    if not referencing:
        print("no download tag found: the pages no longer carry one", file=sys.stderr)
    return 0 if referencing else 1


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    sys.exit(main(sys.argv[1], sys.argv[2]))
