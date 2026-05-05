#!/usr/bin/env python3
"""
Collect runtime for PSO and FlatzincBasedPSO from Rust example outputs.

Assumption:
- Each example prints exactly two elapsed lines in order:
  1) "Elapsed time: ..." for PSO
  2) "Elapsed time: ..." for FlatzincBasedPSO

Usage examples:
  python python/collect_runtimes.py
  python python/collect_runtimes.py --release
  python python/collect_runtimes.py --models g01 g02 g03
  python python/collect_runtimes.py --examples-dir examples --out-dir report
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Optional, Tuple

ELAPSED_RE = re.compile(r"Elapsed time:\s*([0-9]*\.?[0-9]+)\s*([a-zA-Z\u00b5]+)")

# Convert to milliseconds.
UNIT_TO_MS = {
    "ns": 1e-6,
    "us": 1e-3,
    "\u00b5s": 1e-3,
    "ms": 1.0,
    "s": 1000.0,
    "m": 60_000.0,
    "h": 3_600_000.0,
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Collect PSO and Flatzinc-PSO runtimes from all example problems"
    )
    parser.add_argument(
        "--examples-dir",
        default="examples",
        help="Directory containing *_problem.rs files (default: examples)",
    )
    parser.add_argument(
        "--out-dir",
        default="report",
        help="Directory for generated table report (default: report)",
    )
    parser.add_argument(
        "--out-file",
        default="runtime_table.txt",
        help="Output table filename (default: runtime_table.txt)",
    )
    parser.add_argument(
        "--models",
        nargs="+",
        default=None,
        help="Optional subset of models, e.g. g01 g02 1 2 6",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Run cargo in release mode (default: false)",
    )
    parser.add_argument(
        "--cargo-quiet",
        action="store_true",
        help="Pass --quiet to cargo run",
    )
    parser.add_argument(
        "--continue-on-error",
        action="store_true",
        help="Continue with remaining problems if a run fails",
    )
    return parser.parse_args()


def discover_models(examples_dir: Path) -> List[str]:
    models = []
    for path in sorted(examples_dir.glob("*_problem.rs")):
        stem = path.stem
        if stem.endswith("_problem"):
            models.append(stem[: -len("_problem")])
    return models


def parse_elapsed_ms(stdout: str) -> Tuple[Optional[float], Optional[float], List[str]]:
    durations_ms: List[float] = []
    raw_matches: List[str] = []

    for line in stdout.splitlines():
        m = ELAPSED_RE.search(line)
        if not m:
            continue
        value = float(m.group(1))
        unit = m.group(2)
        raw_matches.append(m.group(0).strip())
        if unit not in UNIT_TO_MS:
            continue
        durations_ms.append(value * UNIT_TO_MS[unit])

    if len(durations_ms) < 2:
        return None, None, raw_matches

    return durations_ms[0], durations_ms[1], raw_matches


def run_example(example_name: str, release: bool, cargo_quiet: bool) -> subprocess.CompletedProcess:
    cmd = ["cargo", "run"]
    if release:
        cmd.append("--release")
    cmd.extend(["--example", f"{example_name}_problem"])
    if cargo_quiet:
        cmd.append("--quiet")

    return subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )


def fmt_ms(value: Optional[float]) -> str:
    if value is None:
        return ""
    return f"{value:.3f}"


def write_table(path: Path, rows: List[List[str]]) -> None:
    if not rows:
        return

    widths = [0] * len(rows[0])
    for row in rows:
        for i, cell in enumerate(row):
            widths[i] = max(widths[i], len(cell))

    with path.open("w", encoding="utf-8") as f:
        for ridx, row in enumerate(rows):
            parts = []
            for i, cell in enumerate(row):
                if i == 0:
                    parts.append(cell.ljust(widths[i]))
                else:
                    parts.append(cell.rjust(widths[i]))
            f.write("  ".join(parts) + "\n")
            if ridx == 0:
                f.write("  ".join("-" * w for w in widths) + "\n")


def main() -> int:
    args = parse_args()

    examples_dir = Path(args.examples_dir)
    if not examples_dir.exists():
        print(f"Examples directory not found: {examples_dir}", file=sys.stderr)
        return 2

    available_models = discover_models(examples_dir)
    if not available_models:
        print(f"No *_problem.rs files found in {examples_dir}", file=sys.stderr)
        return 2

    if args.models:
        model_set = set(args.models)
        models = [m for m in available_models if m in model_set]
        missing = sorted(model_set.difference(set(available_models)))
        if missing:
            print(f"Warning: unknown model(s) ignored: {', '.join(missing)}", file=sys.stderr)
    else:
        models = available_models

    if not models:
        print("No models selected", file=sys.stderr)
        return 2

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / args.out_file

    headers = [
        "Problem",
        "PSO (ms)",
        "Flatzinc-PSO (ms)",
    ]
    rows: List[List[str]] = [headers]

    print(f"Collecting runtimes for models: {', '.join(models)}")
    for model in models:
        print(f"Running {model}_problem...")
        cp = run_example(model, release=args.release, cargo_quiet=args.cargo_quiet)
        if cp.returncode != 0:
            print(
                f"  FAILED (exit {cp.returncode})\n{cp.stderr}",
                file=sys.stderr,
            )
            if not args.continue_on_error:
                return cp.returncode
            rows.append([model, "", ""])
            continue

        pso_ms, flatzinc_ms, matches = parse_elapsed_ms(cp.stdout)
        if pso_ms is None or flatzinc_ms is None:
            print(
                f"  Could not find two parseable elapsed times. "
                f"Matched lines: {matches if matches else 'none'}",
                file=sys.stderr,
            )
            if not args.continue_on_error:
                return 1
            rows.append([model, "", ""])
            continue

        rows.append([model, fmt_ms(pso_ms), fmt_ms(flatzinc_ms)])

    write_table(out_path, rows)

    print(f"\nSaved runtime table to: {out_path}")
    with out_path.open("r", encoding="utf-8") as f:
        print(f.read().rstrip())

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
