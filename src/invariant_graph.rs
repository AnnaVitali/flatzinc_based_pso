use flatzinc_serde::{Array, Identifier, Literal};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::algo::toposort;
use petgraph::Directed;
use std::collections::{HashMap, HashSet};
use std::fs;
use crate::args_extractor::args_extractor::ArgsExtractor;
use crate::evaluator::mini_evaluator::CallWithDefines;

#[derive(Clone, Debug, Default)]
pub struct InvariantGraph {
    graph: Graph<(), i32, Directed>,
    index_map: HashMap<NodeIndex, usize>,            
    variables_map: HashMap<String, NodeIndex>,      
    constraints: Vec<CallWithDefines>,
}

impl InvariantGraph {

    pub fn build(constraints: &[CallWithDefines], arrays: &HashMap<Identifier, Array>, save:bool) -> Self {
        let mut graph = Graph::<(), i32, Directed>::new();
        let mut constraint_nodes: Vec<NodeIndex> = Vec::with_capacity(constraints.len());
        let mut variables_map: HashMap<String, NodeIndex> = HashMap::new();
        let mut index_map: HashMap<NodeIndex, usize> = HashMap::new();
        let mut constraint_to_node: HashMap<usize, NodeIndex> = HashMap::new();
        let extractor = ArgsExtractor::new();

        let mut argument_variable_ids_per_constraint: Vec<Vec<String>> = Vec::with_capacity(constraints.len());

        //Create nodes
        for (constraint_index, constraint) in constraints.iter().enumerate() {
            let constraint_node_index = graph.add_node(());
            constraint_nodes.push(constraint_node_index);
            index_map.insert(constraint_node_index, constraint_index);
            constraint_to_node.insert(constraint.id, constraint_node_index);

            let argument_variable_ids = extractor.extract_literal_identifiers(&constraint.call.args);

            if let Some(defined_var) = &constraint.defines {
                variables_map.entry(defined_var.clone()).or_insert_with(|| graph.add_node(()));
            }
            for arg_var in &argument_variable_ids {
                let identifier = Identifier::from(arg_var.as_str());
                
                //Create nodes for its contents instead of the array itself
                if let Some(array) = arrays.get(&identifier) {
                    for arg in &array.contents {
                        if let Literal::Identifier(id) = arg {
                            variables_map.entry(id.to_string()).or_insert_with(|| graph.add_node(()));
                        }
                    }
                } else {
                    //If not an array, create node for this variable
                    variables_map.entry(arg_var.clone()).or_insert_with(|| graph.add_node(()));
                }
            }
            
            argument_variable_ids_per_constraint.push(argument_variable_ids);
        }

        //Create edges
        for (constraint_index, constraint) in constraints.iter().enumerate() {
            let constraint_node_index = constraint_nodes[constraint_index];
            let mut seen_input_nodes: HashSet<NodeIndex> = HashSet::new();

            for variable_name in &argument_variable_ids_per_constraint[constraint_index] {
                let identifier = Identifier::from(variable_name.as_str());
                
                // If this is an array, create edges to its contents
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
            index_map,
            variables_map,
            constraints: constraints.to_vec(),
        };

        if save {
            invariant_graph.export_dot("invariant_graph.dot");
        }

        invariant_graph
    }

    pub fn topologically_sorted_constraints(&self, constraints: &[CallWithDefines]) -> Vec<CallWithDefines> {
        let indices = self.topologically_sort_constraints_indices();
        indices.iter().map(|&i| constraints[i].clone()).collect()
    }

    pub fn export_dot(&self, file_path: &str) {
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

    fn topologically_sort_constraints_indices(&self) -> Vec<usize> {
        let order = toposort(&self.graph, None).expect("Invariant graph has a cycle");
        order
            .into_iter()
            .filter_map(|node| self.index_map.get(&node).copied())
            .collect()
    }

}
