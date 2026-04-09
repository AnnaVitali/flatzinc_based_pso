#!/usr/bin/env python3
"""
Wilcoxon signed-rank test and best result comparison for two algorithms on a given problem.
Usage:
  python python/wilcoxon_compare.py g01 --alg1 flatzinc_pso --alg2 pso
"""
import argparse
import re
from pathlib import Path
from typing import List, Optional
from scipy.stats import wilcoxon

# Manual known optima (same as in compare_summaries.py)
_MANUAL_OPTIMA = {
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
    "1": 1.393,
    "2": -6961.813,
    "6": -213.0,
}

def manual_optimum_for_model(model: str) -> Optional[float]:
    key = model.upper()
    if key in _MANUAL_OPTIMA:
        return _MANUAL_OPTIMA[key]
    m = re.match(r'(?i)g0*([0-9]+)$', model)
    if m:
        idx = int(m.group(1))
        k2 = f"G{idx}"
        return _MANUAL_OPTIMA.get(k2)
    return None

def extract_objectives(filename: Path) -> List[float]:
    objectives = []
    with filename.open("r", encoding="utf-8") as f:
        for line in f:
            match = re.match(r'\s*\d+\s*----\s*([\-\d.eE]+)', line)
            if match:
                objectives.append(float(match.group(1)))
    return objectives

def best_and_mean_std(objs: List[float]):
    import numpy as np
    if not objs:
        return None, None, None
    arr = np.array(objs)
    return arr.min(), arr.mean(), arr.std()

def main():

    parser = argparse.ArgumentParser(description="Wilcoxon test and best result comparison for two algorithms on multiple problems")
    parser.add_argument("problems", nargs='+', help="Problem names, e.g. g01 1 2 6")
    parser.add_argument("--alg1", default="flatzinc_pso", help="First algorithm name (default: flatzinc_pso)")
    parser.add_argument("--alg2", default="pso", help="Second algorithm name (default: pso)")
    args = parser.parse_args()

    reports = Path("reports")

    # Table header
    header = [
        "Problem", "Wilcoxon p", "Wilcoxon stat"
    ]
    print(" ".join(f"{h:>15}" for h in header))

    for problem in args.problems:
        # File naming logic as in compare_summaries.py
        if problem in {"1", "2", "6"}:
            f1 = reports / f"{problem}_flatzinc_pso.txt"
            f2 = reports / f"{problem}_pso.txt"
        else:
            f1 = reports / f"{problem}_{args.alg1}.txt"
            f2 = reports / f"{problem}_{args.alg2}.txt"

        if not f1.exists() or not f2.exists():
            print(f"{problem:>15} {'MISSING':>15} {'':>95}")
            continue

        alg1_obj = extract_objectives(f1)
        alg2_obj = extract_objectives(f2)

        if len(alg1_obj) != len(alg2_obj) or not alg1_obj or not alg2_obj:
            print(f"{problem:>15} {'INVALID':>15} {'':>95}")
            continue

        try:
            stat, p = wilcoxon(alg1_obj, alg2_obj)
        except Exception:
            stat, p = float('nan'), float('nan')

        print(f"{problem:>15} "
              f"{(f'{p:.4g}' if p == p else ''):>15} "
              f"{(f'{stat:.4g}' if stat == stat else ''):>15}")

if __name__ == "__main__":
    main()
