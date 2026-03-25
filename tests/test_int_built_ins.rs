use constraint_evaluator::functional_evaluator::functional_evaluator::FunctionalEvaluator;
use constraint_evaluator::solution_provider::SolutionProvider;
use flatzinc_serde::FlatZinc;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn load(path: &Path) -> Result<FlatZinc, Box<dyn Error>> {
    let mut s = String::new();
    BufReader::new(File::open(path)?).read_to_string(&mut s)?;
    Ok(serde_json::from_str(s.trim_start_matches('\u{feff}'))?)
}

#[test]
pub(crate) fn test_array_int_element() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("array_int_element.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("array_int_element.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let b: i64 = 1;
    let mut c: i64 = 10;
    let array_value: i64 = 10;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 30;
    solution_provider.provide_int("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((array_value - c) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_array_var_int_element() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("array_var_int_element.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("array_var_int_element.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let b: i64 = 1;
    let mut c: i64 = 1;
    let as_array = vec![1_i64, 2, 3];

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);
    solution_provider.provide_array_of_int("as".to_string(), as_array.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 20;
    solution_provider.provide_int("c".to_string(), c);
    solution_provider.provide_array_of_int("as".to_string(), as_array.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((as_array[b as usize - 1] - c) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_int_abs() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_abs.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_abs.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let a: i64 = -1;
    let mut b: i64 = 1;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 2;
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((a.abs() - b.abs()) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_int_div() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_div.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_div.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let a: i64 = 10;
    let mut b: i64 = 2;
    let mut c: i64 = 5;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 0;
    c = 6;
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((2_i64) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_int_eq_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_eq_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_eq_reif.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

       let fzn = load(&path)?;

    let a: i64 = 1;
    let b: i64 = 1;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_le() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_le.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_le.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

       let fzn = load(&path)?;

    let mut a: i64 = 1;
    let mut b: i64 = 1;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 1;
    a = 2;
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((a - b) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_int_le_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_le_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_le_reif.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

       let fzn = load(&path)?;

    let b: i64 = 3;
    let a: i64 = 1;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("a".to_string(), a);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_lin_eq_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_lin_eq_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_lin_eq_reif.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_int("bs".to_string(), vec![2, 2]);
    let bs = vec![2_i64, 2];
    solution_provider.provide_array_of_int("bs".to_string(), bs.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_lin_le() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_lin_le.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_lin_le.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

        let fzn = load(&path)?;

    let mut bs = vec![2_i64, 2];
    let const_term = 5;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_int("bs".to_string(), bs.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![3_i64, 3];
    solution_provider.provide_array_of_int("bs".to_string(), bs.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (bs[0] + bs[1] - const_term) as f64);

    Ok(())
}

#[test]
pub(crate) fn test_int_lin_le_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_lin_le_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_lin_le_reif.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let bs = vec![2_i64, 1];
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_int("bs".to_string(), bs.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_lin_ne() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_lin_ne.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_lin_ne.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

       let fzn = load(&path)?;

    let bs = vec![2_i64, 1];
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_int("bs".to_string(), bs.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    let bs2 = vec![2_i64, 2];
    solution_provider.provide_array_of_int("bs".to_string(), bs2.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_lin_ne_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_lin_ne_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_lin_ne_reif.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

       let fzn = load(&path)?;

    let mut bs = vec![2_i64, 1];

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_int("bs".to_string(), bs.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![2_i64, -1];
    solution_provider.provide_array_of_int("bs".to_string(), bs.clone());

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_lt() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_lt.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_lt.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let a: i64 = 1;
    let mut b: i64 = 2;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 0;
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((a - b + 1) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_int_lt_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_lt_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_lt_reif.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

        let fzn = load(&path)?;

    let a: i64 = 0;
    let mut b: i64 = 1;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 1;
    solution_provider.provide_int("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_max() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_max.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_max.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let a: i64 = 1;
    let b: i64 = 2;
    let mut c: i64 = 2;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 1;
    solution_provider.provide_int("c".to_string(), c);
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((b - c) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_int_min() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_min.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_min.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let a: i64 = 1;
    let b: i64 = 2;
    let mut c: i64 = 1;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 2;
    solution_provider.provide_int("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((c - a) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_int_mod() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_mod.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_mod.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let a = 3;
    let b = 2;
    let mut c = 1;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 2;
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, ((a % b - c) as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_int_ne() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_ne.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_ne.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1;
    let mut b = 0;
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 1;
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_ne_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_ne_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_ne_reif.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let mut a = 0;
    let b = 0;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = 1;
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_int_pow() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_pow.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_pow.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let a = 1;
    let mut b = 2;
    let mut c = 1;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 3;
    c = 9;
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a.pow(b as u32) - c).abs() as f64);

    Ok(())
}

#[test]
pub(crate) fn test_int_times() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("flatzinc_json")
        .join("int_times.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("int")
        .join("int_times.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let a = 2;
    let b = 3;
    let mut c = 6;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = FunctionalEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 3;
    solution_provider.provide_int("a".to_string(), a);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_int("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a * b - c).abs() as f64);

    Ok(())
}
