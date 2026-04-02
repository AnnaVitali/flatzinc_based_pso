# python
#!/usr/bin/env python3
# python/compare_summaries.py
"""
Compare summary JSONs for multiple models produced in `reports/` and output an aligned table.
Usage:
  python python/compare_summaries.py g01 g02 --alg1 pso --alg2 flatzinc_based_pso
"""
import argparse
import json
import re
from pathlib import Path
from typing import Any, Dict, Optional, List

# manual known optima (use model keys like 'G1', 'G2', ...)
_MANUAL_OPTIMA: Dict[str, float] = {
    "G1": -15.0,
    "G2": -0.8036,
    "G3": -1.0,
    "G4": -30665.386,
    "G5": 5126.496,
    "G6": -6961.813,
    "G7": 24.306,
    "G8": 0.095,
    "G9": 680.63,
    "G10": 7049.248,
    "G11": 0.75,
    "G12": -1.0,
    "1": -1.393,
    "2": -6961.813,
    "6": -213.0,
}

def fmt(v: Any) -> str:
    if v is None:
        return ""
    if isinstance(v, bool):
        return str(v)
    if isinstance(v, (int,)):
        return str(v)
    if isinstance(v, float):
        if abs(v - round(v)) < 1e-9:
            return str(int(round(v)))
        return "{:.3f}".format(v)
    return str(v)

def load_summary(path: Path) -> Optional[Dict[str, Any]]:
    if not path.exists():
        return None
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)

def get_field(d: Optional[Dict], *path: str) -> Optional[Any]:
    if not d:
        return None
    cur = d
    for p in path:
        if not isinstance(cur, dict):
            return None
        cur = cur.get(p)
        if cur is None:
            return None
    return cur

def get_optimum(d: Optional[Dict]) -> Optional[Any]:
    if not d:
        return None
    for key_path in (("optimum",), ("meta", "optimum"), ("problem", "optimum")):
        v = get_field(d, *key_path)
        if v is not None:
            return v
    return None

def manual_optimum_for_model(model: str) -> Optional[float]:
    # try direct lookup with canonical key
    key = model.upper()
    if key in _MANUAL_OPTIMA:
        return _MANUAL_OPTIMA[key]
    # try pattern like g01, g1 -> G1
    m = re.match(r'(?i)g0*([0-9]+)$', model)
    if m:
        idx = int(m.group(1))
        k2 = f"G{idx}"
        return _MANUAL_OPTIMA.get(k2)
    # TODO: Add manual optimum for p1, p2, p6 if known
    return None

def build_rows(models: List[str], s_map: Dict[str, Dict[str, Any]], alg1: str, alg2: str):
    # columns: Problem, Optimum, Best alg1, Best alg2, Mean alg1, Mean alg2, Std alg1, Std alg2
    headers = [
        "Problem",
        "Optimum",
        f"Best {alg1}",
        f"Best {alg2}",
        f"Mean {alg1}",
        f"Mean {alg2}",
        f"Std {alg1}",
        f"Std {alg2}",
    ]
    rows: List[List[str]] = [headers]
    for model in models:
        # Map 1,2,6 to p1,p2,p6 for display
        if model == "1":
            display_model = "p1"
        elif model == "2":
            display_model = "p2"
        elif model == "6":
            display_model = "p6"
        else:
            display_model = model

        s_alg1 = s_map.get(model, {}).get(alg1)
        s_alg2 = s_map.get(model, {}).get(alg2)

        # prefer manual known optimum for the model when available (now includes p1, p2, p6)
        manual_opt = manual_optimum_for_model(model)
        if manual_opt is not None:
            optimum = manual_opt
        else:
            optimum = get_optimum(s_alg2) or get_optimum(s_alg1)

        best_alg1 = get_field(s_alg1, "objective", "best")
        best_alg2 = get_field(s_alg2, "objective", "best")
        mean_alg1 = get_field(s_alg1, "objective", "mean")
        mean_alg2 = get_field(s_alg2, "objective", "mean")
        std_alg1 = get_field(s_alg1, "objective", "std")
        std_alg2 = get_field(s_alg2, "objective", "std")

        row = [
            display_model,
            fmt(optimum),
            fmt(best_alg1),
            fmt(best_alg2),
            fmt(mean_alg1),
            fmt(mean_alg2),
            fmt(std_alg1),
            fmt(std_alg2),
        ]
        rows.append(row)
    return rows

def write_aligned(path: Path, rows: List[List[str]]):
    if not rows:
        return
    cols = len(rows[0])
    widths = [0] * cols
    for r in rows:
        for i, cell in enumerate(r):
            widths[i] = max(widths[i], len(cell))
    with path.open("w", encoding="utf-8") as fh:
        for r in rows:
            parts = []
            for i, cell in enumerate(r):
                if i == 0:
                    parts.append(cell.ljust(widths[i]))   # left align problem name
                else:
                    parts.append(cell.rjust(widths[i]))   # right align numeric columns
            fh.write("  ".join(parts) + "\n")


def main():
    p = argparse.ArgumentParser(description="Compare reports summary JSONs for multiple models")
    p.add_argument("models", nargs="+", help="One or more model names, e.g. g01 g02 g03 or 1 2 6")
    p.add_argument("--alg1", default="pso", help="First algorithm name (default: pso)")
    p.add_argument("--alg2", default="flatzinc_based_pso", help="Second algorithm name (default: flatzinc_based_pso)")
    p.add_argument("--out", help="Output file path (default reports/{alg1}_vs_{alg2}_models.txt)")
    args = p.parse_args()

    reports = Path("reports")
    reports.mkdir(exist_ok=True)

    s_map: Dict[str, Dict[str, Any]] = {}
    for model in args.models:
        s_map[model] = {}
        # Special handling for problems 1, 2, 6
        if model in {"1", "2", "6"}:
            f1 = reports / f"{model}_pso_summary.json"
            f2 = reports / f"{model}_flatzinc_pso_summary.json"
        else:
            f1 = reports / f"{model}_{args.alg1}_summary.json"
            f2 = reports / f"{model}_{args.alg2}_summary.json"
        s_map[model][args.alg1] = load_summary(f1)
        s_map[model][args.alg2] = load_summary(f2)

    out_path = Path(args.out) if args.out else reports / f"{args.alg1}_vs_{args.alg2}_models.txt"
    rows = build_rows(args.models, s_map, args.alg1, args.alg2)
    write_aligned(out_path, rows)

    with out_path.open("r", encoding="utf-8") as fh:
        print(fh.read().rstrip())

if __name__ == "__main__":
    main()