use constraint_evaluator::evaluator::mini_evaluator::MiniEvaluator;
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
pub(crate) fn test_array_bool_and() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("array_bool_and.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("array_bool_and.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut as_array = vec![true, true, true];
    solution_provider.provide_array_of_bool("as".to_string(), as_array);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    as_array = vec![true, false, true];
    solution_provider.provide_array_of_bool("as".to_string(), as_array);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_array_bool_element() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("array_bool_element.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("array_bool_element.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let b = 1;
    let mut c = false;

    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_bool("c".to_string(), c);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = true;
    solution_provider.provide_bool("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_array_var_bool_element() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("array_var_bool_element.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("array_var_bool_element.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let as_array = vec![true, true, true];
    let b = 2;
    let mut c = true;
    solution_provider.provide_array_of_bool("as".to_string(), as_array);
    solution_provider.provide_int("b".to_string(), b);
    solution_provider.provide_bool("c".to_string(), c);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = false;
    solution_provider.provide_bool("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_array_bool_xor() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("array_bool_xor.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("array_bool_xor.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut as_array = vec![true, true, true];
    solution_provider.provide_array_of_bool("as".to_string(), as_array);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    as_array = vec![true, false, true];
    solution_provider.provide_array_of_bool("as".to_string(), as_array);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_and() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_and.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_and.ozn");
    eprintln!("Looking for: {}", path_ozn.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path_ozn.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let  a = true;
    let  b = true;
    let mut c = true;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);
    solution_provider.provide_bool("c".to_string(), c);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = false;
    solution_provider.provide_bool("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_clause() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_clause.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_clause.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut pos = vec![true, false];
    let neg = vec![true];
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_bool("pos".to_string(), pos);
    solution_provider.provide_array_of_bool("neg".to_string(), neg.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    pos = vec![false, false];
    solution_provider.provide_array_of_bool("pos".to_string(), pos);
    solution_provider.provide_array_of_bool("neg".to_string(), neg.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_eq_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_eq_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_eq_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let a = true;
    let mut b = true;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    b = false;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_le() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_le.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_le.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut a = false;
    let mut b = true;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = true;
    b = false;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_le_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_le_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_le_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut a = false;
    let mut b = true;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = true;
    b = false;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_lin_eq() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_lin_eq.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_lin_eq.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut bs = vec![true, false];
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_bool("bs".to_string(), bs);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    bs = vec![false, false];
    solution_provider.provide_array_of_bool("bs".to_string(), bs);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_lin_le() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_lin_le.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_lin_le.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_bool("bs".to_string(), vec![true, false]);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_array_of_bool("bs".to_string(), vec![true, true]);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_lt_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_lt_reif.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_lt_reif.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut a = false;
    let mut b = true;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = true;
    b = false;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_lt() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_lt.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_lt.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut a = false;
    let mut b = true;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = true;
    b = false;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_not() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_not.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_not.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut a = false;
    let b = true;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = true;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_or() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_or.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_or.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut a = true;
    let mut b = false;
    let mut c = true;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);
    solution_provider.provide_bool("c".to_string(), c);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = false;
    b = true;
    c = false;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);
    solution_provider.provide_bool("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_bool_xor() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("flatzinc_json")
        .join("bool_xor.json");
    eprintln!("Looking for: {}", path.display());

    if !path.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("bool")
        .join("bool_xor.ozn");
    eprintln!("Looking for: {}", path.display());

    if !path_ozn.exists() {
        return Err(format!("file not found: {}", path.display()).into());
    }

    let fzn = load(&path)?;

    let mut a = true;
    let mut b = true;
    let mut c = false;
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);
    solution_provider.provide_bool("c".to_string(), c);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = false;
    b = true;
    c = false;
    solution_provider.provide_bool("a".to_string(), a);
    solution_provider.provide_bool("b".to_string(), b);
    solution_provider.provide_bool("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}
