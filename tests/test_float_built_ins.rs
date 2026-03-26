use constraint_evaluator::evaluator::mini_evaluator::MiniEvaluator;
use constraint_evaluator::solution_provider::SolutionProvider;
use flatzinc_serde::FlatZinc;
use rand_distr::num_traits::Float;
use std::error::Error;
use std::f64;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn load(path: &Path) -> Result<FlatZinc, Box<dyn Error>> {
    let mut s = String::new();
    BufReader::new(File::open(path)?).read_to_string(&mut s)?;
    Ok(serde_json::from_str(s.trim_start_matches('\u{feff}'))?)
}

#[test]
pub(crate) fn test_array_float_element() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("array_float_element.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("array_float_element.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let b = 1;
    let mut c = 1.0;
    let array_value = 1.0;
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_float("c".to_string(), c);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 20.0;
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_float("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (array_value - c).abs());

    Ok(())
}

#[test]
pub(crate) fn test_array_var_float_element() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("array_var_float_element.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("array_var_float_element.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let b = 1;
    let mut c = 1.0;
    let as_array = vec![1.0, 2.0, 3.0];
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_float("c".to_string(), c);
    solution_provider.provide_array_of_float("as".to_string(), as_array.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 20.0;
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_float("c".to_string(), c);
    solution_provider.provide_array_of_float("as".to_string(), as_array.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (as_array[b as usize - 1] - c).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_abs() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_abs.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_abs.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = -1.0;
    let mut b = 1.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a.abs() - b).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_acos() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_acos.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_acos.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let mut b = 0.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a.acos() - b).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_acosh() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_acosh.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_acosh.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let mut b = 0.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a.acosh() - b).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_asin() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_asin.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_asin.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let mut b = a.asin();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a.asin() - b).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_asinh() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_asinh.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_asinh.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let mut b = a.asinh();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a.asinh() - b).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_atan() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_atan.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_atan.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let mut b = a.atan();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a.atan() - b).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_atanh() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_atanh.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_atanh.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 0.5;
    let b = a.atanh();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_cos() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_cos.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_cos.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let b = a.cos();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_cosh() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_cosh.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_cosh.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let b = a.cosh();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_div() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_div.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_div.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 10.0;
    let b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_eq_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_eq_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_eq_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.5;
    let b = 1.5;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_exp() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_exp.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_exp.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.5;
    let b = a.exp();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_le() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_le.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_le.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = 1.0;
    let b = 1.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a - b).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_le_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_le_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_le_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 3.0;
    let b = 3.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_lin_eq_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lin_eq_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lin_eq_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut bs = vec![2.0, 2.0];
    solution_provider.provide_array_of_float("bs".to_string(), bs.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![3.5, 2.2];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_lin_le() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lin_le.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lin_le.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut bs = vec![2.0, 2.0];
    let const_term = 5.0;

    solution_provider.provide_array_of_float("bs".to_string(), bs.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![3.5, 3.5];
    solution_provider.provide_array_of_float("bs".to_string(), bs.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (bs[0] + bs[1] - const_term).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_lin_le_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lin_le_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lin_le_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut bs = vec![1.5, 2.5];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![3.0, -3.0];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_lin_lt() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lin_lt.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lin_lt.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut bs = vec![2.0, 2.0];
    let const_term = 5.0;
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![3.5, 3.5];
    solution_provider.provide_array_of_float("bs".to_string(), bs.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (bs[0] + bs[1] - const_term + 1.0).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_lin_lt_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lin_lt_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lin_lt_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut bs = vec![1.0, -2.5];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![3.0, 3.0];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_lin_ne() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lin_ne.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lin_ne.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut bs = vec![1.5, 2.5];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![2.5, 2.5];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_lin_ne_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lin_ne_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lin_ne_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut bs = vec![2.0, 1.0];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![2.0, 2.0];
    solution_provider.provide_array_of_float("bs".to_string(), bs);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_ln() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_ln.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_ln.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 4.4816890703380645;
    let b = a.ln();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_log10() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_log10.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_log10.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let b = a.log10();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_log2() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_log2.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_log2.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let b = a.log2();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_lt() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lt.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lt.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = 1.4;
    let mut b = 1.5;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = 1.8;
    b = 1.7;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (a - b + 1.0).abs());

    Ok(())
}

#[test]
pub(crate) fn test_float_lt_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_lt_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_lt_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = 1.4;
    let mut b = 1.5;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = 1.4;
    b = 1.5;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_max() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_max.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_max.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.2;
    let mut b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 0.5;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_min() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_min.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_min.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let mut b = 2.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 0.5;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_ne() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_ne.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_ne.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let mut b = 2.6;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 1.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_ne_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_ne_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_ne_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let mut b = 2.5;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = 1.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_plus() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_plus.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_plus.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 2.0;
    let b = 3.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    assert_eq!(0, 0);

    Ok(())
}

#[test]
pub(crate) fn test_float_pow() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_pow.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_pow.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 2.0;
    let b = 3.0;
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_sin() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_sin.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_sin.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let b = a.sin();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_sinh() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_sinh.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_sinh.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let b = a.sinh();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_sqrt() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_sqrt.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_sqrt.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 4.0;
    let b = a.sqrt();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_tan() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_tan.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_tan.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let b = a.tan();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_tanh() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_tanh.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_tanh.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 1.0;
    let b = a.tanh();
    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_float_times() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("float_times.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("float_times.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 3.0;
    let b = 2.0;

    solution_provider.provide_float("a".to_string(), a);
    solution_provider.provide_float("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_int2float() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("flatzinc_json")
        .join("int2float.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("float")
        .join("int2float.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = 2;
    let b = a as f64;
    solution_provider.provide_int("x".to_string(), a);
    solution_provider.provide_float("y".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}
