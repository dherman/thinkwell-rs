#!/usr/bin/env python3
"""Compile complete Rust blocks from the tutorials without running inference."""
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]
SITE = ROOT / "website"
OUT = ROOT / "target" / "website-examples"


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    config = json.loads((SITE / "docs.json").read_text())
    for group in config["navigation"]["groups"]:
        for page in group["pages"]:
            assert (SITE / f"{page}.mdx").is_file(), f"Missing page: {page}"

    first = (SITE / "get-started/first-answer.mdx").read_text()
    manifest = re.search(r"```toml\n(.*?)\n```", first, re.S).group(1)
    # Use precisely the tutorial dependencies, replacing only its checkout path.
    dependencies = manifest.split("[dependencies]\n", 1)[1]
    dependencies = dependencies.replace(
        '"../thinkwell-rs/crates/thinkwell"',
        json.dumps(str(ROOT / "crates/thinkwell")),
    )
    bins = []
    excerpts = 0
    for page in sorted(SITE.rglob("*.mdx")):
        source = page.read_text()
        for link in re.findall(r"\]\((/[^)]+)\)", source):
            route = link.split("#", 1)[0].lstrip("/")
            assert (SITE / f"{route}.mdx").is_file(), f"{page}: broken link {link}"
        excerpt_pattern = r'\{/\* source: ([^\n]+) \*/\}\s*```rust title="[^"\n]+"\n(.*?)\n```'
        found_excerpts = re.findall(excerpt_pattern, source, re.S)
        assert len(found_excerpts) == len(re.findall(r'^```rust title=', source, re.M)), (
            f"{page}: each titled Rust excerpt needs a source marker"
        )
        for original, block in found_excerpts:
            # Excerpts may be dedented for display; all other text must match.
            normalize = lambda text: "\n".join(line.lstrip() for line in text.strip().splitlines())
            assert normalize(block) in normalize((ROOT / original).read_text()), (
                f"{page}: excerpt no longer matches {original}"
            )
            excerpts += 1
        for index, block in enumerate(re.findall(r"```rust\n(.*?)\n```", source, re.S)):
            name = "-".join(page.relative_to(SITE).with_suffix("").parts) + f"-{index}"
            path = OUT / f"{name}.rs"
            path.write_text(block + "\n")
            bins.append(f'[[bin]]\nname = "{name}"\npath = "{name}.rs"\n')
    assert bins, "No Rust examples found"
    manifest = (
        '[package]\nname = "website-examples"\nversion = "0.0.0"\n'
        'edition = "2024"\npublish = false\n\n[workspace]\n\n'
        '[dependencies]\n' + dependencies + "\n" + "\n".join(bins)
    )
    path = OUT / "Cargo.toml"
    path.write_text(manifest)
    env = dict(os.environ)
    env.setdefault("CARGO_TARGET_DIR", str(ROOT / "target"))
    for command in [
        ["cargo", "check", "--manifest-path", str(path), "--bins"],
        ["cargo", "fmt", "--manifest-path", str(path), "--", "--check"],
    ]:
        subprocess.run(command, cwd=ROOT, env=env, check=True)
    print(f"Checked {len(bins)} complete Rust examples, {excerpts} source excerpts, and local page links.")


if __name__ == "__main__":
    main()
