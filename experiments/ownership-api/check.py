#!/usr/bin/env python3
"""Compile/run positive fixtures; require specific diagnostics from negative cases."""
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "experiments/ownership-api"
OUT = ROOT / "target/ownership-api"
OUT.mkdir(parents=True, exist_ok=True)


def run(args, *, success=True):
    result = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, timeout=60)
    if success and result.returncode:
        raise RuntimeError(f"{args}\n{result.stdout}\n{result.stderr}")
    return result


print(run(["rustc", "--version"]).stdout.strip())
common = ["rustc", "--edition=2024", "-Dwarnings"]
lib = OUT / "libownership_probe.rlib"
run(common + ["--crate-name", "ownership_probe", "--crate-type", "rlib", str(SOURCE / "prototype.rs"), "-o", str(lib)])
external = ["--extern", f"ownership_probe={lib}"]
tests = OUT / "scenarios"
run(common + ["--test", str(SOURCE / "scenarios.rs"), *external, "-o", str(tests)])
print(run([str(tests)]).stdout.strip())

for name in ["send_adapter", "generic_helper"]:
    output = OUT / name
    run(common + [str(SOURCE / "cases" / f"{name}.rs"), *external, "-o", str(output)])
    run([str(output)])
    print(f"PASS: {name}")

negative = {
    "overlapping_access": "E0499",
    "two_mutable_tools": "E0499",
    "escape": "E0515",
    "clone_scoped": "E0599",
    "generic_conditional": "E0308",
    "send_without_adapter": "future cannot be sent between threads safely",
}
for name, expected in negative.items():
    result = run(["rustc", "--edition=2024", "--error-format=json", str(SOURCE / "cases" / f"{name}.rs"), *external, "-o", str(OUT / name)], success=False)
    (OUT / f"{name}.stderr.json").write_text(result.stderr)
    errors = [json.loads(line) for line in result.stderr.splitlines() if line.startswith("{")]
    errors = [entry for entry in errors if entry.get("level") == "error"]
    codes = [entry.get("code", {}).get("code") for entry in errors if entry.get("code")]
    messages = "\n".join(entry.get("message", "") for entry in errors)
    if result.returncode == 0 or not (expected in codes or expected in messages):
        raise RuntimeError(f"{name}: expected {expected}, got\n{result.stderr}")
    print(f"PASS: {name} rejected as expected ({expected})")
print("Ownership experiment passed; no ACP/model/serialization behavior tested.")
