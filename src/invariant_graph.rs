use flatzinc_serde::{Array, Identifier, Literal};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::algo::toposort;
use petgraph::Directed;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use crate::args_extractor::ArgsExtractor;
use crate::invariant_evaluator::CallWithDefines;
use crate::data_utility::data_utility::ConstraintEvaluation;

#[derive(Clone, Debug, Default)]
pub struct InvariantGraph {
    graph: Graph<(), i32, Directed>,
    arrays: HashMap<Identifier, Array>,
    index_map: HashMap<NodeIndex, usize>,            
    constraint_to_node: HashMap<usize, NodeIndex>,   
    variables_map: HashMap<String, NodeIndex>,      
    evaluations: HashMap<NodeIndex, ConstraintEvaluation>,
    constraints: Vec<CallWithDefines>,
}

impl InvariantGraph {

    pub fn build(constraints: &[CallWithDefines], arrays: &HashMap<Identifier, Array>) -> Self {
        let mut graph = Graph::<(), i32, Directed>::new();
        let mut constraint_nodes: Vec<NodeIndex> = Vec::with_capacity(constraints.len());
        let mut variables_map: HashMap<String, NodeIndex> = HashMap::new();
        let mut index_map: HashMap<NodeIndex, usize> = HashMap::new();
        let mut constraint_to_node: HashMap<usize, NodeIndex> = HashMap::new();
        let extractor = ArgsExtractor::new();

        let mut argument_variable_ids_per_constraint: Vec<Vec<String>> = Vec::with_capacity(constraints.len());
        
        for (constraint_index, constraint) in constraints.iter().enumerate() {
            // Create constraint node
            let constraint_node_index = graph.add_node(());
            constraint_nodes.push(constraint_node_index);
            index_map.insert(constraint_node_index, constraint_index);
            constraint_to_node.insert(constraint.id, constraint_node_index);
            
            // Extract argument identifiers
            let argument_variable_ids = extractor.extract_literal_identifiers(&constraint.call.args);
            
            // Create variable nodes as we discover them
            if let Some(defined_var) = &constraint.defines {
                variables_map.entry(defined_var.clone()).or_insert_with(|| graph.add_node(()));
            }
            for arg_var in &argument_variable_ids {
                let identifier = Identifier::from(arg_var.as_str());
                
                // If this is an array, create nodes for its contents instead of the array itself
                if let Some(array) = arrays.get(&identifier) {
                    for arg in &array.contents {
                        if let Literal::Identifier(id) = arg {
                            variables_map.entry(id.to_string()).or_insert_with(|| graph.add_node(()));
                        }
                    }
                } else {
                    // Not an array, create node for this variable
                    variables_map.entry(arg_var.clone()).or_insert_with(|| graph.add_node(()));
                }
            }
            
            argument_variable_ids_per_constraint.push(argument_variable_ids);
        }

        // Second pass: create edges in the graph
        for (constraint_index, constraint) in constraints.iter().enumerate() {
            let constraint_node_index = constraint_nodes[constraint_index];
            let mut seen_input_nodes: HashSet<NodeIndex> = HashSet::new();

            for variable_name in &argument_variable_ids_per_constraint[constraint_index] {
                let identifier = Identifier::from(variable_name.as_str());
                
                // If this is an array, create edges to its contents instead
                if let Some(array) = arrays.get(&identifier) {
                    for arg in &array.contents {
                        if let Literal::Identifier(id) = arg {
                            if constraint.defines.as_deref() == Some(id.as_str()) {
                                continue;
                            }
                            if let Some(&variable_node_index) = variables_map.get(&id.to_string()) {
                                if variable_node_index != constraint_node_index && seen_input_nodes.insert(variable_node_index) {
                                    graph.add_edge(variable_node_index, constraint_node_index, 1);
                                }
                            }
                        }
                    }
                } else {
                    // Not an array, create edge normally
                    if let Some(&variable_node_index) = variables_map.get(variable_name) {
                        if variable_node_index != constraint_node_index && seen_input_nodes.insert(variable_node_index) {
                            if constraint.defines.as_deref() == Some(variable_name.as_str()) {
                                continue;
                            }
                            graph.add_edge(variable_node_index, constraint_node_index, 2);
                        }
                    }
                }
            }
            if let Some(defined_var) = constraint.defines.as_ref() {
                if let Some(&variable_node_index) = variables_map.get(defined_var) {
                    if variable_node_index != constraint_node_index {
                        graph.add_edge(constraint_node_index, variable_node_index, 3);
                    }
                }
            }
        }

        let invariant_graph = Self {
            graph,
            arrays: arrays.clone(),
            index_map,
            constraint_to_node,
            variables_map,
            evaluations: HashMap::new(),
            constraints: constraints.to_vec(),
        };

        //invariant_graph.export_dot("graph.dot");
        invariant_graph
    }


    pub fn topological_order_indices(&self) -> Vec<usize> {
        let order = toposort(&self.graph, None).expect("Invariant graph has a cycle");
        order
            .into_iter()
            .filter_map(|node| self.index_map.get(&node).copied())
            .collect()
    }

    pub fn topologically_sorted_constraints(&self, constraints: &[CallWithDefines]) -> Vec<CallWithDefines> {
        let indices = self.topological_order_indices();
        indices.iter().map(|&i| constraints[i].clone()).collect()
    }

    pub fn clear_evaluations(&mut self) {
        self.evaluations.clear();
    }

    pub fn attach_evaluation_by_constraint_index(&mut self, constraint_idx: usize, evaluation: ConstraintEvaluation) {
        self.evaluations.remove(self.constraint_to_node.get(&constraint_idx).unwrap_or_else(|| panic!("No node found for constraint index {}", constraint_idx)));
        if let Some(&constraint_node_index) = self.constraint_to_node.get(&constraint_idx) {
            self.evaluations.insert(constraint_node_index, evaluation);
        }
    }

    pub fn get_variable_constraint_evaluation_nodes(&self, var_name: &str) -> Vec<ConstraintEvaluation> {
        use std::collections::{VecDeque, HashSet};
        let mut result = Vec::new();
        let mut visited_vars = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(&start_var_node) = self.variables_map.get(var_name) {
            queue.push_back(start_var_node);
            visited_vars.insert(start_var_node);
        } else {
            return Vec::new();
        }

        while let Some(var_node) = queue.pop_front() {
            // For each outgoing neighbor (constraint node)
            for constraint_node in self.graph.neighbors_directed(var_node, petgraph::Direction::Outgoing) {
                // If this constraint has an evaluation, add it
                if let Some(eval) = self.evaluations.get(&constraint_node) {
                    result.push(eval.clone());
                }
                // If this constraint defines a variable, add that variable node to the queue if not visited
                if let Some(&constraint_idx) = self.index_map.get(&constraint_node) {
                    if let Some(constraint) = self.constraints.get(constraint_idx) {
                        if let Some(defined_var) = &constraint.defines {
                            if let Some(&defined_var_node) = self.variables_map.get(defined_var) {
                                if !visited_vars.contains(&defined_var_node) {
                                    queue.push_back(defined_var_node);
                                    visited_vars.insert(defined_var_node);
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn get_constraint_evaluation_nodes(&self) -> Vec<ConstraintEvaluation>{
        let mut out: Vec<ConstraintEvaluation> = Vec::with_capacity(self.evaluations.len());
        for (node, eval) in &self.evaluations {
            out.push(eval.clone());
        }
        out
    }

    pub fn get_variable_constraint_nodes(&self, var_name: &str) -> Vec<CallWithDefines> {
        use std::collections::{VecDeque, HashSet};
        let mut result = Vec::new();
        let mut visited_vars = HashSet::new();
        let mut queue = VecDeque::new();

        let start_var_node = match self.variables_map.get(var_name) {
            Some(&node) => node,
            None => panic!("No node in the graph defines the variable '{}'", var_name),
        };
        queue.push_back(start_var_node);
        visited_vars.insert(start_var_node);

        while let Some(var_node) = queue.pop_front() {
            for constraint_node in self.graph.neighbors_directed(var_node, petgraph::Direction::Outgoing) {
                if let Some(&constraint_idx) = self.index_map.get(&constraint_node) {
                    if let Some(constraint) = self.constraints.get(constraint_idx) {
                        result.push(constraint.clone());
                        // If this constraint defines a variable, add that variable node to the queue if not visited
                        if let Some(defined_var) = &constraint.defines {
                            if let Some(&defined_var_node) = self.variables_map.get(defined_var) {
                                if !visited_vars.contains(&defined_var_node) {
                                    queue.push_back(defined_var_node);
                                    visited_vars.insert(defined_var_node);
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn get_constraints_without_defines(&self) -> Vec<CallWithDefines> {
        self.constraints.iter().filter(|constraint| constraint.defines.is_none()).cloned().collect()
    }


    pub fn get_constraint_variables(&self, constraint: &CallWithDefines) -> Vec<String> {
        if let Some(&constraint_node) = self.constraint_to_node.get(&constraint.id) {
            let mut variables = Vec::new();
            
            // Collect neighbor node indices (both incoming and outgoing)
            let mut neighbor_indices: HashSet<NodeIndex> = HashSet::new();
            neighbor_indices.extend(self.graph.neighbors_directed(constraint_node, petgraph::Direction::Incoming));
            neighbor_indices.extend(self.graph.neighbors_directed(constraint_node, petgraph::Direction::Outgoing));
            
            // Find variable names for these nodes
            for (var_name, &var_node_idx) in &self.variables_map {
                if neighbor_indices.contains(&var_node_idx) {
                    variables.push(var_name.clone());
                }
            }
            
            variables
        } else {
            Vec::new()
        }
    }

    pub fn get_constraint_variables_by_index(&self, constraint_idx: usize) -> Vec<String> {
        if let Some(&constraint_node) = self.constraint_to_node.get(&constraint_idx) {
            let mut variables = Vec::new();
            
            // Collect neighbor node indices (both incoming and outgoing)
            let mut neighbor_indices: HashSet<NodeIndex> = HashSet::new();
            neighbor_indices.extend(self.graph.neighbors_directed(constraint_node, petgraph::Direction::Incoming));
            neighbor_indices.extend(self.graph.neighbors_directed(constraint_node, petgraph::Direction::Outgoing));
            
            // Find variable names for these nodes
            for (var_name, &var_node_idx) in &self.variables_map {
                if neighbor_indices.contains(&var_node_idx) {
                    variables.push(var_name.clone());
                }
            }
            
            variables
        } else {
            Vec::new()
        }
    }

    pub fn get_root_variables(&self, var_name: &str) -> Vec<String> {

        let mut result: HashSet<String> = HashSet::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut stack: VecDeque<String> = VecDeque::new();

        stack.push_back(var_name.to_string());

        while let Some(current_var) = stack.pop_back() {
            if !visited.insert(current_var.clone()) {
                continue;
            }

            // Check if this variable is defined by a constraint
            if let Some(constraint) = self.get_defining_constraint(&current_var) {
                let input_vars = self.get_constraint_variables(&constraint);

                for var in input_vars {
                    // Avoid self-loop
                    if var != current_var {
                        stack.push_back(var);
                    }
                }
            } else {
                // No defining constraint → this is a REAL variable
                result.insert(current_var);
            }
    }

    let mut out: Vec<String> = result.into_iter().collect();
    out.sort();
    out
}

    pub fn get_defining_constraint(&self, var_name: &str) -> Option<CallWithDefines> {
            self.constraints
                .iter()
                .find(|c| c.defines.as_deref() == Some(var_name))
                .cloned()
    }

    pub fn get_correlated_variables(&self, var_name: &str) -> Vec<(String, usize)> {
        let Some(&variable_node) = self.variables_map.get(var_name) else {
            return Vec::new();
        };

        let mut correlated_counts: HashMap<String, usize> = HashMap::new();

        // Gather every constraint node touching this variable (as input or as defined output).
        let mut related_constraints: HashSet<NodeIndex> = HashSet::new();
        related_constraints.extend(
            self.graph
                .neighbors_directed(variable_node, petgraph::Direction::Outgoing),
        );
        related_constraints.extend(
            self.graph
                .neighbors_directed(variable_node, petgraph::Direction::Incoming),
        );

        // Build a reverse map once to recover variable names from node indices.
        let node_to_var: HashMap<NodeIndex, &String> = self
            .variables_map
            .iter()
            .map(|(name, &node)| (node, name))
            .collect();

        for constraint_node in related_constraints {
            let mut variable_nodes: HashSet<NodeIndex> = HashSet::new();
            variable_nodes.extend(
                self.graph
                    .neighbors_directed(constraint_node, petgraph::Direction::Incoming),
            );
            variable_nodes.extend(
                self.graph
                    .neighbors_directed(constraint_node, petgraph::Direction::Outgoing),
            );

            for var_node in variable_nodes {
                if let Some(name) = node_to_var.get(&var_node) {
                    if name.as_str() != var_name {
                        *correlated_counts.entry((*name).clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut result: Vec<(String, usize)> = correlated_counts.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn get_normalized_correlated_variables(&self, var_name: &str) -> Vec<(String, f64)> {
        let correlated_variables = self.get_correlated_variables(var_name);
        if correlated_variables.is_empty() {
            return Vec::new();
        }

        let total_constraints = self.get_variable_constraint_nodes(var_name).len();
        if total_constraints == 0 {
            return Vec::new();
        }

        correlated_variables
            .into_iter()
            .map(|(name, shared_constraints)| {
                (name, shared_constraints as f64 / total_constraints as f64)
            })
            .collect()
    }

     fn node_label(&self, node: NodeIndex) -> String {
        if let Some(&constraint_idx) = self.index_map.get(&node) {
            if let Some(constraint) = self.constraints.get(constraint_idx) {
                return format!("constraint(id={}, call={})", constraint.id, constraint.call.id);
            }
            return format!("constraint(index={})", constraint_idx);
        }

        for (var_name, &var_node_idx) in &self.variables_map {
            if var_node_idx == node {
                return format!("variable({})", var_name);
            }
        }

        format!("unknown(node={})", node.index())
    }

    fn export_dot(&self, file_path: &str) {
        let mut dot = String::from("digraph InvariantGraph {\n");

        for node in self.graph.node_indices() {
            let label = self.node_label(node).replace('"', "\\\"");
            dot.push_str(&format!("  n{} [label=\"{}\"];\n", node.index(), label));
        }

        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                let weight = self.graph.edge_weight(edge).copied().unwrap_or_default();
                dot.push_str(&format!(
                    "  n{} -> n{} [label=\"{}\"];\n",
                    source.index(),
                    target.index(),
                    weight
                ));
            }
        }

        dot.push_str("}\n");

        match fs::write(file_path, dot) {
            Ok(_) => eprintln!("[InvariantGraph] Graph saved to {}", file_path),
            Err(err) => eprintln!("[InvariantGraph] Failed to save graph to {}: {}", file_path, err),
        }
    }

}
