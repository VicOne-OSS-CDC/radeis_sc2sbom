use super::dependency::{Dependency, DependencyRelationship};
use std::collections::{HashMap, HashSet};

/// Dependency graph for tree visualization
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, DependencyNode>,
    pub roots: Vec<String>, // Direct dependencies (entry points for tree traversal)
}

/// Node in the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub dependency: Dependency,
    pub children: Vec<String>, // Package identifiers (name@version) of direct dependencies
}

impl DependencyGraph {
    /// Build graph with parent-child relationships from lock files
    pub fn build_from_deps_with_relationships(
        dependencies: &[Dependency],
        relationships: &[DependencyRelationship],
    ) -> Self {
        let mut nodes = HashMap::new();
        let mut roots = Vec::new();
        let mut name_to_id: HashMap<String, String> = HashMap::new();

        // First pass: create nodes for all dependencies and build name→id lookup map
        for dep in dependencies {
            let id = format!("{}@{}", dep.name, dep.version);
            nodes.insert(
                id.clone(),
                DependencyNode {
                    dependency: dep.clone(),
                    children: Vec::new(),
                },
            );

            // Build name→id map for O(1) lookups during relationship resolution
            name_to_id.insert(dep.name.clone(), id.clone());

            if dep.is_direct {
                roots.push(id);
            }
        }

        // Second pass: populate children vectors using relationships
        for rel in relationships {
            // Resolve all child names to IDs using O(1) HashMap lookup
            let child_ids: Vec<String> = rel
                .child_names
                .iter()
                .filter_map(|child_name| {
                    let child_id = name_to_id.get(child_name).cloned();
                    if child_id.is_none() {
                        eprintln!(
                            "Warning: Child '{}' of '{}' not found in dependency list. \
                            This may indicate a corrupted or out-of-date lock file, or a \
                            dependency that was filtered out or excluded during scanning.",
                            child_name, rel.parent_id
                        );
                    }
                    child_id
                })
                .collect();

            // Then update parent node (mutable borrow)
            // Use a HashSet to deduplicate children before adding
            if let Some(parent_node) = nodes.get_mut(&rel.parent_id) {
                let existing_children: std::collections::HashSet<_> =
                    parent_node.children.iter().cloned().collect();
                let new_children: Vec<String> = child_ids
                    .into_iter()
                    .filter(|id| !existing_children.contains(id))
                    .collect();
                parent_node.children.extend(new_children);
            }
        }

        let mut graph = DependencyGraph { nodes, roots };

        // Correct is_direct flags based on graph structure
        if !relationships.is_empty() {
            graph.correct_direct_flags();
        }

        graph
    }

    pub fn get_node(&self, id: &str) -> Option<&DependencyNode> {
        self.nodes.get(id)
    }

    /// Corrects is_direct flags based on actual dependency graph structure.
    /// A dependency is truly direct if it has no parents in the graph.
    /// A dependency is transitive if it appears as a child of any package.
    pub fn correct_direct_flags(&mut self) {
        // Build set of all packages that appear as children (are dependencies of something)
        let mut has_parents = HashSet::new();

        // Collect all packages that appear as children in any relationship
        for node in self.nodes.values() {
            for child_id in &node.children {
                has_parents.insert(child_id.clone());
            }
        }

        // Rebuild roots list - only packages with no parents are true direct dependencies
        let mut new_roots = Vec::new();

        // Update is_direct flags and build new roots list
        for (dep_id, node) in self.nodes.iter_mut() {
            if has_parents.contains(dep_id) {
                // This package is a child of another package
                // Therefore it's transitive
                node.dependency.is_direct = false;
            } else {
                // This package has no parents at all
                // It's a root (direct dependency)
                node.dependency.is_direct = true;
                new_roots.push(dep_id.clone());
            }
        }

        // Replace roots with the corrected list
        self.roots = new_roots;
    }

    pub fn get_dependency_chains(&self, target_id: &str) -> Vec<Vec<String>> {
        let mut all_chains = Vec::new();
        let mut current_chain = Vec::new();
        let mut visited = HashSet::new();

        // Start DFS from each root
        for root_id in &self.roots {
            self.dfs_find_chains(
                root_id,
                target_id,
                &mut current_chain,
                &mut all_chains,
                &mut visited,
            );
        }

        // If no chains found from roots, check if target exists and return single-element chain
        if all_chains.is_empty() && self.nodes.contains_key(target_id) {
            vec![vec![target_id.to_string()]]
        } else {
            all_chains
        }
    }

    pub fn dfs_find_chains(
        &self,
        current_id: &str,
        target_id: &str,
        chain: &mut Vec<String>,
        all_chains: &mut Vec<Vec<String>>,
        visited: &mut HashSet<String>,
    ) {
        // Prevent infinite loops from circular dependencies
        if visited.contains(current_id) {
            return;
        }

        chain.push(current_id.to_string());
        visited.insert(current_id.to_string());

        // Found the target - save this chain
        if current_id == target_id {
            all_chains.push(chain.clone());
        } else if let Some(node) = self.nodes.get(current_id) {
            // Recurse into children
            for child_id in &node.children {
                self.dfs_find_chains(child_id, target_id, chain, all_chains, visited);
            }
        }

        // Backtrack
        chain.pop();
        visited.remove(current_id);
    }
}
