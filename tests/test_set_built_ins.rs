use constraint_evaluator::evaluator::mini_evaluator::MiniEvaluator;
use constraint_evaluator::solution_provider::SolutionProvider;
use flatzinc_serde::FlatZinc;
use std::collections::HashSet;
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
pub(crate) fn test_array_set_element() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("array_set_element.json");

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("array_set_element.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut i = 1;
    let mut s = HashSet::from([1, 2]);
    solution_provider.provide_int("i".to_string(), i);
    solution_provider.provide_set("s".to_string(), s.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), None);
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    i = 1;
    s = HashSet::from([1, 3]);
    solution_provider.provide_int("i".to_string(), i);
    solution_provider.provide_set("s".to_string(), s.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_array_var_set_element() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("array_var_set_element.json");

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("array_var_set_element.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let as_arr = vec![HashSet::from([1, 2]), HashSet::from([1])];
    let i = 1;
    let s = HashSet::from([1, 2]);
    solution_provider.provide_array_of_set("as".to_string(), as_arr.clone());
    solution_provider.provide_int("i".to_string(), i);
    solution_provider.provide_set("s".to_string(), s.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), None);
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_set_card() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_card.json");

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_card.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let s = HashSet::from([1, 2]);
    let mut c = 2;
    solution_provider.provide_set("s".to_string(), s.clone());
    solution_provider.provide_int("c".to_string(), c);

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    c = 1;
    solution_provider.provide_set("s".to_string(), s.clone());
    solution_provider.provide_int("c".to_string(), c);

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, (s.len() as f64 - c as f64).abs());

    Ok(())
}

#[test]
pub(crate) fn test_set_eq_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_eq_reif.json");

    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_eq_reif.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2, 3]);
    let mut b = HashSet::from([2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2]);
    b = HashSet::from([1, 3]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_set_diff() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_diff.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_diff.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2, 3]);
    let mut b = HashSet::from([1, 2]);
    let mut c = HashSet::from([3]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());
    solution_provider.provide_set("c".to_string(), c.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2, 3]);
    b = HashSet::from([1, 2, 3]);
    c = HashSet::from([3]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());
    solution_provider.provide_set("c".to_string(), c.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    let diff: HashSet<i64> = a.difference(&b).copied().collect();
    assert_eq!(violation, diff.symmetric_difference(&c).count() as f64);

    Ok(())
}

#[test]
pub(crate) fn test_set_in() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_in.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_in.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut x = 1;
    let mut s = HashSet::from([1, 2, 3]);
    solution_provider.provide_int("x".to_string(), x);
    solution_provider.provide_set("s".to_string(), s.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    x = 5;
    s = HashSet::from([1, 2, 3]);
    solution_provider.provide_int("x".to_string(), x);
    solution_provider.provide_set("s".to_string(), s.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_set_in_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_in_reif.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_in_reif.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut x = 1;
    let mut s = HashSet::from([1, 2, 3]);
    solution_provider.provide_int("x".to_string(), x);
    solution_provider.provide_set("s".to_string(), s.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    x = 5;
    s = HashSet::from([1, 2, 3]);
    solution_provider.provide_int("x".to_string(), x);
    solution_provider.provide_set("s".to_string(), s.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);
    Ok(())
}

#[test]
pub(crate) fn test_set_intersect() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_intersect.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_intersect.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2, 3]);
    let mut b = HashSet::from([1, 2, 4, 5]);
    let mut c = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());
    solution_provider.provide_set("c".to_string(), c.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2, 3]);
    b = HashSet::from([1, 2, 4, 5]);
    c = HashSet::from([4, 5]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());
    solution_provider.provide_set("c".to_string(), c.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    let intersect: HashSet<i64> = a.intersection(&b).copied().collect();
    assert_eq!(violation, intersect.difference(&c).count() as f64);

    Ok(())
}

#[test]
pub(crate) fn test_set_le() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_le.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_le.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2]);
    let mut b = HashSet::from([1, 3]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([2, 3, 4, 5]);
    b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    let mut xv: Vec<i64> = a.iter().cloned().collect();
    let mut yv: Vec<i64> = b.iter().cloned().collect();

    xv.sort();
    yv.sort();
    let mut real_viol = 0.0;
    if !(xv <= yv) {
        for (i, elem) in yv.iter().enumerate() {
            if *elem < xv[i] {
                real_viol += 1.0;
            }
        }
    }
    assert_eq!(violation, real_viol);

    Ok(())
}

#[test]
pub(crate) fn test_set_le_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_le_reif.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_le_reif.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2]);
    let mut b = HashSet::from([4, 5]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2, 4, 5]);
    b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_set_lt() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_lt.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_lt.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2]);
    let mut b = HashSet::from([4, 5]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2, 4, 5]);
    b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    let mut xv: Vec<i64> = a.iter().cloned().collect();
    let mut yv: Vec<i64> = b.iter().cloned().collect();

    xv.sort();
    yv.sort();
    let mut real_viol = 0.0;
    if !(xv < yv) {
        for (i, elem) in yv.iter().enumerate() {
            if *elem <= xv[i] {
                real_viol += 1.0;
            }
        }
    }
    assert_eq!(violation, real_viol);
    Ok(())
}

#[test]
pub(crate) fn test_set_lt_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_lt_reif.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_lt_reif.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2]);
    let mut b = HashSet::from([4, 5]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2, 4, 5]);
    b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_set_ne() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_ne.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_ne.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2]);
    let mut b = HashSet::from([1, 3]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2]);
    b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 1.0);

    Ok(())
}

#[test]
pub(crate) fn test_set_ne_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_ne_reif.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_ne_reif.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2]);
    let mut b = HashSet::from([1, 3]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2]);
    b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_set_subset() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_subset.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_subset.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2]);
    let mut b = HashSet::from([1, 2, 3]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2, 3]);
    b = HashSet::from([1, 4]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, a.difference(&b).count() as f64);

    Ok(())
}

#[test]
pub(crate) fn test_set_subset_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_subset_reif.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_subset_reif.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2]);
    let mut b = HashSet::from([1, 2, 3]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2]);
    b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    Ok(())
}

#[test]
pub(crate) fn test_set_superset() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_superset.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_superset.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2, 3]);
    let mut b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2]);
    b = HashSet::from([1, 2, 4]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, a.difference(&b).count() as f64);
    Ok(())
}

#[test]
pub(crate) fn test_set_superset_reif() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_superset_reif.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_superset_reif.ozn");

    let fzn = load(&path)?;

    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    let mut a = HashSet::from([1, 2, 3]);
    let mut b = HashSet::from([1, 2]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2]);
    b = HashSet::from([1, 2, 4]);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());

    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);
    Ok(())
}

#[test]
pub(crate) fn test_set_symdiff() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_symdiff.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_symdiff.ozn");

    let fzn = load(&path)?;

    let mut a = HashSet::from([1, 2, 3]);
    let mut b = HashSet::from([1, 2]);
    let mut c = HashSet::from([3]);
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());
    solution_provider.provide_set("c".to_string(), c.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2, 3]);
    b = HashSet::from([1, 2, 4, 5]);
    c = HashSet::from([1, 2]);
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());
    solution_provider.provide_set("c".to_string(), c.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    let sym_diff: HashSet<i64> = a.symmetric_difference(&b).copied().collect();
    assert_eq!(violation, sym_diff.difference(&c).count() as f64);

    Ok(())
}

#[test]
pub(crate) fn test_set_union() -> Result<(), Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("flatzinc_json")
        .join("set_union.json");
    let path_ozn = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("minizinc_built_ins")
        .join("set")
        .join("set_union.ozn");

    let fzn = load(&path)?;

    let mut a = HashSet::from([1, 2, 3]);
    let mut b = HashSet::from([1, 2]);
    let mut c = HashSet::from([1, 2, 3]);
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());
    solution_provider.provide_set("c".to_string(), c.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn.clone(), Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    assert_eq!(violation, 0.0);

    a = HashSet::from([1, 2, 3]);
    b = HashSet::from([4, 5]);
    c = HashSet::from([1, 2]);
    let mut solution_provider = SolutionProvider::new(fzn.clone(), &path_ozn);
    solution_provider.provide_set("a".to_string(), a.clone());
    solution_provider.provide_set("b".to_string(), b.clone());
    solution_provider.provide_set("c".to_string(), c.clone());

    let mut invariant_evaluator = MiniEvaluator::new(&*path, fzn, Some("verbose"));
    let (_, violation) = invariant_evaluator.evaluate_invariants_graph(&solution_provider);
    let union: HashSet<i64> = a.union(&b).copied().collect();
    assert_eq!(violation, union.difference(&c).count() as f64);

    Ok(())
}
