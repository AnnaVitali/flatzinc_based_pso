use crate::invariant_evaluator::CallWithDefines;
use crate::solution_provider::VariableValue;
use flatzinc_serde::{Array, Call, Identifier};
use std::collections::HashMap;
use crate::args_extractor_sub_types::bool_args_extractor::BoolArgsExtractor;
use crate::invariant_evaluator_sub_types::bool_invariant_evaluator::{A_TERM_INDEX, AS_ARRAY_INDEX, B_TERM_INDEX, BS_ARRAY_INDEX, C_TERM_INDEX};

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
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
         let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        
        if vars_identifier.contains(&defined_var) {
            let as_array = self.args_extractor.extract_bool_array(AS_ARRAY_INDEX, &self.arrays, &constraint.call, complete_solution);

        as_array.iter().all(|&item| item)
        } else {
            let as_array = self.args_extractor.extract_bool_array(AS_ARRAY_INDEX, &self.arrays, &constraint.call, complete_solution);
            let r = self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution);
            
            as_array.iter().all(|&item| item) == r
        }
        
    }

    pub fn array_bool_element(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        self.args_extractor.extract_bool_element_array(
            AS_ARRAY_INDEX,
            &constraint.call,
            &self.arrays,
           complete_solution
        )
    }

    pub fn bool_and(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for bool_and");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a && b
        } else {
            self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, complete_solution)
        }
    }

    pub fn bool_clause(&self,
                       constraint: &CallWithDefines,
                       complete_solution: &HashMap<String, VariableValue>,
    ) -> bool{
        let as_array = self.args_extractor.extract_bool_array(
            AS_ARRAY_INDEX,
            &self.arrays,
            &constraint.call,
            complete_solution
        );
        let bs_array = self.args_extractor.extract_bool_defined_elements_array(
            BS_ARRAY_INDEX,
            &self.arrays,
            &constraint.call,
            complete_solution
        );

        as_array.iter().any(|&item| item==true) || !bs_array.iter().any(|&item| item==false)

    }

    pub fn bool_eq(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
         
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for float_le_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self
            .args_extractor
               .extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
        a
        } else {
            self
            .args_extractor
               .extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution)
        }
        
    }

    pub fn bool_not(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for bool_not");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
            !a
        } else {
            self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution)
        }
    }

    pub fn bool_eq_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for bool_eq_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a == b
        } else {
            self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, complete_solution)
        }
    }

    pub fn bool_le_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for bool_le_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a <= b
        } else {
            self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, complete_solution)
        }
    }

    pub fn bool_lt_reif(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for bool_lt_reif");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a < b
        } else {
            self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, complete_solution)
        }
    }

    pub fn bool_or(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for bool_or");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a || b
        } else {
            self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, complete_solution)
        }
    }

    pub fn bool_xor(
        &self,
        constraint: &CallWithDefines,
        complete_solution: &HashMap<String, VariableValue>,
    ) -> bool {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for bool_xor");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
            let b = self.args_extractor.extract_bool_value(B_TERM_INDEX, &constraint.call, complete_solution);
            a ^ b
        } else {
            self.args_extractor.extract_bool_value(C_TERM_INDEX, &constraint.call, complete_solution)
        }
    }

    pub fn bool2int(&self, constraint: &CallWithDefines,
                    complete_solution: &HashMap<String, VariableValue>,) -> i64 {
        let vars_identifier = self.args_extractor.extract_literal_identifiers(&constraint.call.args);
        let defined_var = constraint.defines.as_ref().expect("Expected a defined variable for bool2int");
        if vars_identifier.contains(&defined_var) {
            let a = self.args_extractor.extract_bool_value(A_TERM_INDEX, &constraint.call, complete_solution);
            a as i64
        } else {
            self.args_extractor.extract_int_value(B_TERM_INDEX, &constraint.call, complete_solution)
        }
    }
}
