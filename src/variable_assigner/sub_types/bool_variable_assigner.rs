use crate::args_extractor::sub_types::bool_args_extractor::BoolArgsExtractor;
use crate::evaluator::mini_evaluator::CallWithDefines;
use crate::evaluator::sub_types::bool_functional_evaluator::{
    A_TERM_INDEX, AS_ARRAY_INDEX, B_TERM_INDEX, BS_ARRAY_INDEX, C_TERM_INDEX,
};
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Identifier};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct BoolVariableAssigner {
    args_extractor: BoolArgsExtractor,
    arrays: HashMap<Identifier, Array>,
}

impl BoolVariableAssigner {
    pub fn new(arrays: HashMap<Identifier, Array>) -> Self {
        let args_extractor = BoolArgsExtractor::new();

        Self {
            args_extractor,
            arrays,
        }
    }

    pub fn array_bool_and(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for array_bool_and");
            if vars_identifier.contains(&defined_var) {
                let as_array = args_extractor.extract_bool_array(
                    AS_ARRAY_INDEX,
                    &arrays,
                    &call,
                    solution,
                );
                as_array.iter().all(|&item| item)
            } else {
                let as_array = args_extractor.extract_bool_array(
                    AS_ARRAY_INDEX,
                    &arrays,
                    &call,
                    solution,
                );
                let r = args_extractor.extract_bool_value(
                    B_TERM_INDEX,
                    &call,
                    solution,
                );
                as_array.iter().all(|&item| item) == r
            }
        })
    }

    pub fn array_bool_element(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            args_extractor.extract_bool_element_array(
                AS_ARRAY_INDEX,
                &call,
                &arrays,
                solution,
            )
        })
    }

    pub fn bool_and(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool_and");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a && b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn bool_clause(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let arrays = self.arrays.clone();
        let call = constraint.call.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let as_array = args_extractor.extract_bool_array(
                AS_ARRAY_INDEX,
                &arrays,
                &call,
                solution,
            );
            let bs_array = args_extractor.extract_bool_defined_elements_array(
                BS_ARRAY_INDEX,
                &arrays,
                &call,
                solution,
            );
            as_array.iter().any(|&item| item == true) || !bs_array.iter().any(|&item| item == false)
        })
    }

    pub fn bool_eq(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool_eq");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                a
            } else {
                args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn bool_not(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool_not");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                !a
            } else {
                args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn bool_eq_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool_eq_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a == b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn bool_le_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool_le_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a <= b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn bool_lt_reif(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool_lt_reif");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a < b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn bool_or(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool_or");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a || b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn bool_xor(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> bool + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool_xor");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                let b = args_extractor.extract_bool_value(B_TERM_INDEX, &call, solution);
                a ^ b
            } else {
                args_extractor.extract_bool_value(C_TERM_INDEX, &call, solution)
            }
        })
    }

    pub fn bool2int(
        &self,
        constraint: &CallWithDefines,
        _solution: &HashMap<String, VariableValue>,
    ) -> Box<dyn Fn(&HashMap<String, VariableValue>) -> i64 + Send + Sync> {
        let args_extractor = self.args_extractor.clone();
        let call = constraint.call.clone();
        let defines = constraint.defines.clone();
        Box::new(move |solution: &HashMap<String, VariableValue>| {
            let vars_identifier = args_extractor.extract_literal_identifiers(&call.args);
            let defined_var = defines
                .as_ref()
                .expect("Expected a defined variable for bool2int");
            if vars_identifier.contains(&defined_var) {
                let a = args_extractor.extract_bool_value(A_TERM_INDEX, &call, solution);
                a as i64
            } else {
                args_extractor.extract_int_value(B_TERM_INDEX, &call, solution)
            }
        })
    }
}
