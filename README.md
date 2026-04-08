# P

This repository contains the implementation of MiniEvaluator. Given a MiniZinc model and the corresponding FlatZinc representation in JSON format, `MiniEvaluator` is able to determine which constraints of the model have been violated and the magnitude of each violation.

## How to test your model

To test your own model, place the `.mzn` file inside the `minizinc` directory and use the following command to generate the FlatZinc in `.json` format:

```bash
minizinc -c --solver mzn-fzn --fzn-format json --output-fzn-to-stdout ./minizinc/<model_name>.mzn > ./minizinc/flatzinc_json/<model_name>.json
```

After that, to provide a solution and evaluate constraint violations, refer to the examples (in the `examples/` directory)  that demonstrate how to use `MiniEvaluator`.

### Important Notes

`MiniEvaluator` is able to determine the value of variables that in **FlatZinc** are marked with `defined: true`. However, it is not able to infer the value of variables that are only introduced and not defined, i.e. variables that have only the flag `introduced: true`.

```json
{
  "variables": {
    "X_INTRODUCED_0_" : { "type" : "int", "domain" : [[0, 85000]], "defined" : true },
    "X_INTRODUCED_1_" : { "type" : "int", "introduced" : true }
  },
}
```

The use of `Option` types, the `let` keyword, or leaving **domains undefined** may result in the introduction of such undefined variables. Before providing a solution to `MiniEvaluator`, check your flatzinc.json file to ensure that it does not contain these variables, otherwise errors will be returned.
 
Typical workflow to follow:

 - Write a MiniZinc model
 - Compile it to FlatZinc JSON format
 - Check the presence of introduced and not defined variables
 - Write your test in rust using `MiniEvaluator` and `SolutionProvider`

## Project Structure

The project is organized as follows:

- `minizinc/` — Contains MiniZinc models (`.mzn`), output files (`.ozn`), and FlatZinc JSON representations (`flatzinc_json/`).
- `minizinc_built_ins/` — Built-ins MiniZinc and FlatZinc test models, organized by type (e.g., `bool/`, `int/`, `float/`, `set/`).
- `src/` — Main Rust source code, including core modules such as the evaluator, solution provider, ozn_parser, and utilities.
- `examples/` — Example Rust programs demonstrating how to use MiniEvaluator with different models and solutions.
- `tests/` — Test suite for validating built-in constraints and project functionality.

## Build

To build the project:

```bash
cargo build
```

## Run an example
To run an example:

```bash
cargo run --release --example <example_name>
```
## Run tests
To run all the available tests:

```bash
cargo test
```