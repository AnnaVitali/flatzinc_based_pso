import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent
MODELS_DIR = ROOT / "minizinc"
JSON_DIR = ROOT / MODELS_DIR / "json_flatzinc"

def main() -> None:
    for index in range(1, 13):
        problem = f"g{index:02d}"

        model_path = MODELS_DIR / f"{problem}.mzn"
        output_path = JSON_DIR / f"{problem}.json"

        cmd = [
            "minizinc",
            "-c",
            "--solver",
            "mzn-fzn",
            "--fzn-format",
            "json",
            "--output-fzn-to-stdout",
            str(model_path),
        ]

        print(f"Compiling {problem}...")
        result = subprocess.run(cmd, capture_output=True, text=True, check=False)
        if result.returncode != 0:
            raise RuntimeError(
                f"Failed to compile {problem}: {result.stderr.strip() or result.stdout.strip()}"
            )

        output_path.write_text(result.stdout, encoding="utf-8")
        print(f"Wrote {output_path}")


if __name__ == "__main__":
    main()