#!/usr/bin/env python3
"""
Run an example problem multiple times and collect PSO and FlatzincBasedPSO results.

Outputs (in ./reports):
  - {model}_pso.txt
  - {model}_flatzinc_pso.txt
  - {model}_pso_summary.json
  - {model}_flatzinc_pso_summary.json

This variant writes tabular `.txt` reports with columns separated by ` ---- `.
"""
import argparse
import json
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path
from statistics import mean, stdev
import math


def parse_args():
    p = argparse.ArgumentParser(description="Run an example problem multiple times and collect results")
    p.add_argument("model", help="Model name, e.g. g01, g02, ...")
    p.add_argument("--runs", type=int, default=100, help="Number of runs (default 100)")
    p.add_argument("--build-first", action="store_true", dest="build_first", help="Run `cargo build --examples --release` before executing runs (faster per-run)")
    p.add_argument("--release", action="store_true", help="Use release build when building first")
    p.add_argument("--cargo-quiet", action="store_true", help="Pass --quiet to cargo run/build to reduce noise")
    return p.parse_args()


def run_command(cmd, timeout=None):
    try:
        cp = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=timeout)
        return cp.returncode, cp.stdout, cp.stderr
    except subprocess.TimeoutExpired:
        return 124, "", "timeout"


def safe_float(x):
    try:
        return float(x)
    except Exception:
        return float("nan")


def summarize(values):
    # values: list[float]
    clean = [v for v in values if not (v is None)]
    # remove NaNs for stats
    nums = [v for v in clean if not (isinstance(v, float) and (v != v))]
    if not nums:
        return {
            "count": 0,
            "best": None,
            "mean": None,
            "std": None,
            "worst": None,
        }
    s = {
        "count": len(nums),
        "best": min(nums),
        "mean": mean(nums),
        "std": stdev(nums) if len(nums) > 1 else 0.0,
        "worst": max(nums),
    }
    return s


def format_value(v):
    # Represent numbers with up to 20 significant digits, keep 'nan' for invalids
    if v is None:
        return "None"
    if isinstance(v, float):
        if math.isnan(v):
            return "nan"
        return "{:.20g}".format(v)
    return str(v)


def write_tabular_txt(path: Path, headers, rows, sep_between=" ---- "):
    # headers: list[str]
    # rows: list[list[str]]
    # compute column widths
    col_widths = []
    cols = len(headers)
    for c in range(cols):
        max_w = len(headers[c])
        for r in rows:
            max_w = max(max_w, len(r[c]))
        col_widths.append(max_w)

    # build formats: header centered, data right-aligned
    header_cells = []
    for c, h in enumerate(headers):
        header_cells.append(h.center(col_widths[c]))
    header_line = sep_between.join(header_cells)

    sep_cells = []
    for w in col_widths:
        sep_cells.append("-" * w)
    sep_line = sep_between.join(sep_cells)

    with path.open("w", encoding="utf-8") as fh:
        fh.write(header_line + "\n")
        fh.write(sep_line + "\n")
        for r in rows:
            cells = []
            for c, cell in enumerate(r):
                # right-align numeric-looking columns (digits, dot, minus), else left-align
                if all(ch.isdigit() or ch in ".-+eE" for ch in cell) and any(ch.isdigit() for ch in cell):
                    cells.append(cell.rjust(col_widths[c]))
                else:
                    cells.append(cell.ljust(col_widths[c]))
            fh.write(sep_between.join(cells) + "\n")


def main():
    args = parse_args()
    model = args.model
    runs = args.runs
    build_first = args.build_first

    example_name = f"{model}_problem"
    cargo_quiet_flag = ["--quiet"] if args.cargo_quiet else []

    reports_dir = Path("reports")
    reports_dir.mkdir(exist_ok=True)

    if build_first:
        mode = "--release" if args.release else ""
        build_cmd = ["cargo", "build", "--examples"]
        if args.release:
            build_cmd.append("--release")
        if args.cargo_quiet:
            build_cmd.append("--quiet")
        print("Building examples (this runs once)...")
        code, out, err = run_command(build_cmd)
        if code != 0:
            print("cargo build failed:", err)
            sys.exit(1)
        print("Build finished")

    alg_data = defaultdict(list)  # algorithm -> list of (objective, violation)

    print(f"Running {example_name} {runs} times (this may take a while)...")
    for i in range(1, runs + 1):
        cmd = ["cargo", "run", "--release", "--example", example_name]
        if args.cargo_quiet:
            cmd.append("--quiet")
        # run the example
        start = time.time()
        code, out, err = run_command(cmd)
        elapsed = time.time() - start
        if code != 0:
            print(f"Run {i}: cargo run failed (exit {code})\n{err}")
            # still continue to next run
            continue

        # parse lines looking for JSON objects printed by example
        lines = out.splitlines()
        parsed_count = 0
        for line in lines:
            line = line.strip()
            if not line:
                continue
            if not (line.startswith('{') and '"algorithm"' in line):
                continue
            try:
                j = json.loads(line)
            except Exception:
                # try a quick heuristic to convert single quotes or trailing commas
                try:
                    j = json.loads(line.replace("'", '"'))
                except Exception:
                    print(f"Run {i}: failed to parse JSON line: {line}")
                    continue
            alg = j.get("algorithm")
            obj = safe_float(j.get("objective"))
            viol = safe_float(j.get("violation"))
            alg_data[alg].append({"run": i, "objective": obj, "violation": viol})
            parsed_count += 1

        if parsed_count == 0:
            print(f"Run {i}: no machine-readable result lines found in stdout. Full stdout:\n{out}\n--- stderr:\n{err}")
        else:
            print(f"Run {i} finished in {elapsed:.2f}s, parsed {parsed_count} result line(s)")

    # For each algorithm, compute stats and write TXT and summary JSON
    for alg, entries in alg_data.items():
        txt_path = reports_dir / f"{model}_{alg}.txt"
        headers = ["run", "objective", "violation"]
        rows = []
        for e in entries:
            rows.append([str(e["run"]), format_value(e["objective"]), format_value(e["violation"])])
        write_tabular_txt(txt_path, headers, rows, sep_between=" ---- ")

        objs = [e['objective'] for e in entries]
        viols = [e['violation'] for e in entries]
        summary = {
            "model": model,
            "algorithm": alg,
            "runs": len(entries),
            "objective": summarize(objs),
            "violation": summarize(viols),
        }
        json_path = reports_dir / f"{model}_{alg}_summary.json"
        with json_path.open("w", encoding="utf-8") as fh:
            json.dump(summary, fh, indent=2)
        print(f"Wrote TXT: {txt_path} and summary: {json_path}")

    if not alg_data:
        print("No results collected. Make sure the example prints machine-readable JSON lines like:\n{\"algorithm\":\"pso\", ... }")


if __name__ == '__main__':
    main()