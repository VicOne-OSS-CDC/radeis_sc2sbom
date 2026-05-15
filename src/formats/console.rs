use crate::cli::TreeStyle;
use crate::models::{Dependency, DependencyGraph, DependencyNode, DependencyRelationship, Sbom};
#[cfg(feature = "internal")]
use crate::models::{};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::fs;

// Tree rendering constants
const TREE_BRANCH: &str = "├── ";
const TREE_LAST: &str = "└── ";
const TREE_VERTICAL: &str = "│   ";
const TREE_SPACE: &str = "    ";
const COMPACT_ARROW: &str = "→ ";
const MAX_DEPTH: usize = 5;

pub struct TreeRenderer {
    style: TreeStyle,
}

impl TreeRenderer {
    pub fn new(style: TreeStyle) -> Self {
        TreeRenderer { style }
    }

    pub fn render_dependency_list(
        &self,
        dependencies: &[&Dependency],
        graph: &DependencyGraph,
    ) -> String {
        match self.style {
            TreeStyle::Flat => {
                // Current flat list style
                let mut output = String::new();
                for dep in dependencies {
                    let dev_marker = if dep.is_dev { " [dev]" } else { "" };
                    output.push_str(&format!("  {} @ {}{}\n", dep.name, dep.version, dev_marker));
                }
                output
            }
            TreeStyle::Tree => {
                // Main section: Only show direct production dependencies with their trees
                // Optimize: filter first to avoid creating unnecessary ID strings for all dependencies
                let direct_deps: Vec<String> = dependencies
                    .iter()
                    .filter_map(|d| {
                        let dep_id = format!("{}@{}", d.name, d.version);
                        graph.get_node(&dep_id).and_then(|node| {
                            if node.dependency.is_direct && !node.dependency.is_dev {
                                Some(dep_id)
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                self.render_tree_classic(&direct_deps, graph)
            }
            TreeStyle::Compact => {
                // Main section: Only show direct production dependencies with their trees
                // Optimize: filter first to avoid creating unnecessary ID strings for all dependencies
                let direct_deps: Vec<String> = dependencies
                    .iter()
                    .filter_map(|d| {
                        let dep_id = format!("{}@{}", d.name, d.version);
                        graph.get_node(&dep_id).and_then(|node| {
                            if node.dependency.is_direct && !node.dependency.is_dev {
                                Some(dep_id)
                            } else {
                                None
                            }
                        })
                    })
                    .collect();

                self.render_tree_compact(&direct_deps, graph)
            }
        }
    }

    /// Renders a flat list of all distinct packages grouped by type
    pub fn render_distinct_packages_list(
        &self,
        dependencies: &[&Dependency],
        graph: &DependencyGraph,
    ) -> String {
        let mut output = String::new();

        // Collect packages by type using corrected flags from graph
        let mut direct_prod: Vec<&Dependency> = Vec::new();
        let mut direct_dev: Vec<&Dependency> = Vec::new();
        let mut transitive: Vec<&Dependency> = Vec::new();

        for dep in dependencies {
            let dep_id = format!("{}@{}", dep.name, dep.version);
            if let Some(node) = graph.get_node(&dep_id) {
                let corrected_dep = &node.dependency;
                if corrected_dep.is_direct && !corrected_dep.is_dev {
                    direct_prod.push(dep);
                } else if corrected_dep.is_direct && corrected_dep.is_dev {
                    direct_dev.push(dep);
                } else {
                    transitive.push(dep);
                }
            }
        }

        // Sort each group alphabetically
        direct_prod.sort_by(|a, b| a.name.cmp(&b.name));
        direct_dev.sort_by(|a, b| a.name.cmp(&b.name));
        transitive.sort_by(|a, b| a.name.cmp(&b.name));

        // Render direct production dependencies
        if !direct_prod.is_empty() {
            for dep in &direct_prod {
                output.push_str(&format!("{} @ {} [direct]\n", dep.name, dep.version));
            }
        }

        // Render development dependencies
        if !direct_dev.is_empty() {
            if !direct_prod.is_empty() {
                output.push('\n');
            }
            for dep in &direct_dev {
                output.push_str(&format!("{} @ {} [direct, dev]\n", dep.name, dep.version));
            }
        }

        // Render transitive dependencies
        if !transitive.is_empty() {
            if !direct_prod.is_empty() || !direct_dev.is_empty() {
                output.push('\n');
            }
            for dep in &transitive {
                output.push_str(&format!("{} @ {}\n", dep.name, dep.version));
            }
        }

        output
    }

    /// Renders appendix sections for development and transitive dependencies
    pub fn render_appendix_sections(
        &self,
        dependencies: &[&Dependency],
        graph: &DependencyGraph,
    ) -> String {
        let mut output = String::new();

        // Get dev and transitive-only dependency IDs
        // Optimize: use filter_map to avoid creating ID strings for filtered-out items
        let dev_deps: Vec<String> = dependencies
            .iter()
            .filter_map(|d| {
                let dep_id = format!("{}@{}", d.name, d.version);
                graph.get_node(&dep_id).and_then(|node| {
                    if node.dependency.is_direct && node.dependency.is_dev {
                        Some(dep_id)
                    } else {
                        None
                    }
                })
            })
            .collect();

        let transitive_only: Vec<String> = dependencies
            .iter()
            .filter_map(|d| {
                let dep_id = format!("{}@{}", d.name, d.version);
                graph.get_node(&dep_id).and_then(|node| {
                    if !node.dependency.is_direct {
                        Some(dep_id)
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Render dev dependencies section
        if !dev_deps.is_empty() {
            output.push_str("#### Development Dependencies\n");
            match &self.style {
                TreeStyle::Tree => {
                    output.push_str(&self.render_tree_classic(&dev_deps, graph));
                }
                TreeStyle::Compact => {
                    output.push_str(&self.render_tree_compact(&dev_deps, graph));
                }
                TreeStyle::Flat => {
                    // Flat style doesn't need appendix sections
                }
            }
        }

        // Render transitive-only dependencies section
        if !transitive_only.is_empty() {
            if !dev_deps.is_empty() {
                output.push('\n');
            }
            output.push_str("#### Other Transitive Dependencies\n");
            match &self.style {
                TreeStyle::Tree => {
                    output.push_str(&self.render_tree_classic(&transitive_only, graph));
                }
                TreeStyle::Compact => {
                    output.push_str(&self.render_tree_compact(&transitive_only, graph));
                }
                TreeStyle::Flat => {
                    // Flat style doesn't need appendix sections
                }
            }
        }

        output
    }

    fn render_tree_classic(&self, roots: &[String], graph: &DependencyGraph) -> String {
        let mut output = String::with_capacity(roots.len() * 100); // Pre-allocate
        let total = roots.len();
        let mut visited = std::collections::HashSet::new();

        for (idx, root_id) in roots.iter().enumerate() {
            let is_last = idx == total - 1;
            if let Some(node) = graph.get_node(root_id) {
                self.render_node_classic_to_buffer(
                    node,
                    "",
                    is_last,
                    0,
                    MAX_DEPTH,
                    graph,
                    &mut visited,
                    &mut output,
                );
            }
        }

        output
    }

    fn render_node_classic_to_buffer(
        &self,
        node: &DependencyNode,
        prefix: &str,
        is_last: bool,
        depth: usize,
        max_depth: usize,
        graph: &DependencyGraph,
        visited: &mut HashSet<String>,
        output: &mut String,
    ) {
        let dep = &node.dependency;
        let dep_id = format!("{}@{}", dep.name, dep.version);

        // Check for circular reference
        if visited.contains(&dep_id) {
            let node_prefix = if is_last { TREE_LAST } else { TREE_BRANCH };
            let _ = writeln!(
                output,
                "{}{}{}@{} [circular reference]",
                prefix, node_prefix, dep.name, dep.version
            );
            return;
        }

        visited.insert(dep_id.clone());

        // Node prefix
        let node_prefix = if is_last { TREE_LAST } else { TREE_BRANCH };

        // Status markers
        let mut markers = Vec::new();
        if dep.is_direct {
            markers.push("direct");
        }
        if dep.is_dev {
            markers.push("dev");
        }
        let marker_str = if markers.is_empty() {
            String::new()
        } else {
            format!(" [{}]", markers.join(", "))
        };

        let _ = writeln!(
            output,
            "{}{}{} @ {}{}",
            prefix, node_prefix, dep.name, dep.version, marker_str
        );

        // Check if we've reached max depth
        if depth >= max_depth {
            let child_count = node.children.len();
            if child_count > 0 {
                let child_prefix = if is_last {
                    format!("{}{}", prefix, TREE_SPACE)
                } else {
                    format!("{}{}", prefix, TREE_VERTICAL)
                };
                let _ = writeln!(
                    output,
                    "{}{}[{} more dependencies... max depth {} reached]",
                    child_prefix, TREE_LAST, child_count, max_depth
                );
            }
            visited.remove(&dep_id);
            return;
        }

        // Render children
        let child_count = node.children.len();
        for (idx, child_id) in node.children.iter().enumerate() {
            let is_last_child = idx == child_count - 1;
            let child_prefix = if is_last {
                format!("{}{}", prefix, TREE_SPACE)
            } else {
                format!("{}{}", prefix, TREE_VERTICAL)
            };

            if let Some(child_node) = graph.get_node(child_id) {
                self.render_node_classic_to_buffer(
                    child_node,
                    &child_prefix,
                    is_last_child,
                    depth + 1,
                    max_depth,
                    graph,
                    visited,
                    output,
                );
            }
        }

        // Backtrack for other branches
        visited.remove(&dep_id);
    }

    fn render_tree_compact(&self, roots: &[String], graph: &DependencyGraph) -> String {
        let mut output = String::with_capacity(roots.len() * 100); // Pre-allocate
        let mut visited = std::collections::HashSet::new();

        for root_id in roots {
            if let Some(node) = graph.get_node(root_id) {
                self.render_node_compact_to_buffer(
                    node,
                    0,
                    MAX_DEPTH,
                    graph,
                    &mut visited,
                    &mut output,
                );
            }
        }

        output
    }

    fn render_node_compact_to_buffer(
        &self,
        node: &DependencyNode,
        depth: usize,
        max_depth: usize,
        graph: &DependencyGraph,
        visited: &mut HashSet<String>,
        output: &mut String,
    ) {
        let dep = &node.dependency;
        let dep_id = format!("{}@{}", dep.name, dep.version);

        // Check for circular reference
        if visited.contains(&dep_id) {
            let indent = "  ".repeat(depth);
            let arrow = if depth > 0 { COMPACT_ARROW } else { "" };
            let _ = writeln!(
                output,
                "{}{}{} @ {} [circular reference]",
                indent, arrow, dep.name, dep.version
            );
            return;
        }

        visited.insert(dep_id.clone());

        // Indentation
        let indent = "  ".repeat(depth);

        // Arrow for non-root nodes
        let arrow = if depth > 0 { COMPACT_ARROW } else { "" };

        // Status markers
        let mut markers = Vec::new();
        if dep.is_direct {
            markers.push("direct");
        }
        if dep.is_dev {
            markers.push("dev");
        }
        let marker_str = if markers.is_empty() {
            String::new()
        } else {
            format!(" [{}]", markers.join(", "))
        };

        let _ = writeln!(
            output,
            "{}{}{} @ {}{}",
            indent, arrow, dep.name, dep.version, marker_str
        );

        // Check if we've reached max depth
        if depth >= max_depth {
            let child_count = node.children.len();
            if child_count > 0 {
                let child_indent = "  ".repeat(depth + 1);
                let _ = writeln!(
                    output,
                    "{}{}[{} more dependencies... max depth {} reached]",
                    child_indent, COMPACT_ARROW, child_count, max_depth
                );
            }
            visited.remove(&dep_id);
            return;
        }

        // Render children with visited tracking
        for child_id in &node.children {
            if let Some(child_node) = graph.get_node(child_id) {
                self.render_node_compact_to_buffer(
                    child_node,
                    depth + 1,
                    max_depth,
                    graph,
                    visited,
                    output,
                );
            }
        }

        // Backtrack for other branches
        visited.remove(&dep_id);
    }

    pub fn render_dependency_chain(&self, chain: &[String], graph: &DependencyGraph) -> String {
        if chain.is_empty() {
            return String::new();
        }
        let mut output = String::new();

        for (idx, dep_id) in chain.iter().enumerate() {
            let is_last = idx == chain.len() - 1;
            let indent = "  ".repeat(idx);
            let prefix = if is_last { TREE_LAST } else { TREE_BRANCH };

            if let Some(node) = graph.get_node(dep_id) {
                let dep = &node.dependency;
                let marker = if dep.is_direct { " [direct]" } else { "" };
                output.push_str(&format!(
                    "{}{}{} @ {}{}\n",
                    indent, prefix, dep.name, dep.version, marker
                ));
            } else {
                // Fallback if node not found
                output.push_str(&format!("{}{}{}\n", indent, prefix, dep_id));
            }
        }

        output
    }
}

pub fn print_dependencies_tree(
    sbom: &Sbom,
    tree_style: &TreeStyle,
    _compact: bool, // Reserved for future use
    relationships: &[DependencyRelationship],
) -> Vec<(String, String)> {
    println!("\n{}", "═".repeat(51));
    println!("📦 DEPENDENCIES{:>36} total", sbom.dependencies.len());
    println!("{}", "═".repeat(51));

    if sbom.dependencies.is_empty() {
        println!("\nNo dependencies found.");
        return Vec::new();
    }

    // Build dependency graph with relationships
    let graph =
        DependencyGraph::build_from_deps_with_relationships(&sbom.dependencies, relationships);
    let renderer = TreeRenderer::new(tree_style.clone());

    // Group dependencies by ecosystem
    let mut by_ecosystem: HashMap<String, Vec<&Dependency>> = HashMap::new();
    for dep in &sbom.dependencies {
        by_ecosystem
            .entry(dep.ecosystem.clone())
            .or_default()
            .push(dep);
    }

    // Sort ecosystems for consistent output
    let mut ecosystems: Vec<_> = by_ecosystem.keys().collect();
    ecosystems.sort();

    // Collect appendix data for later printing
    let mut appendix_sections: Vec<(String, String)> = Vec::new();

    for ecosystem in ecosystems {
        let deps = &by_ecosystem[ecosystem];

        // Calculate counts using corrected flags from graph in a single pass
        let mut direct_count = 0;
        let mut transitive_count = 0;
        let mut dev_count = 0;

        for d in deps.iter() {
            let dep_id = format!("{}@{}", d.name, d.version);
            if let Some(node) = graph.get_node(&dep_id) {
                let dep = &node.dependency;
                if dep.is_dev {
                    dev_count += 1;
                }
                if dep.is_direct {
                    direct_count += 1;
                } else {
                    transitive_count += 1;
                }
            }
        }

        println!("\n{} ({} packages)", ecosystem.to_uppercase(), deps.len());
        println!("{}", "─".repeat(51));

        // For tree style, show only direct dependencies at root level
        // For flat style, show all dependencies in this ecosystem
        match tree_style {
            TreeStyle::Flat => {
                for dep in deps {
                    let dev_marker = if dep.is_dev { " [dev]" } else { "" };
                    println!("  {} @ {}{}", dep.name, dep.version, dev_marker);
                }
            }
            TreeStyle::Tree | TreeStyle::Compact => {
                // Section 1: Main tree showing only direct production dependencies
                let output = renderer.render_dependency_list(deps, &graph);
                if !output.trim().is_empty() {
                    print!("{}", output);
                } else {
                    // Fallback if no direct production dependencies
                    println!("  (no direct production dependencies)");
                }

                // Section 2: Distinct packages list (flat list of all packages)
                println!(
                    "\n#### {} Distinct Packages List ({} packages)",
                    ecosystem.to_uppercase(),
                    deps.len()
                );
                println!(
                    "{} direct, {} transitive, {} dev\n",
                    direct_count, transitive_count, dev_count
                );
                let distinct_list = renderer.render_distinct_packages_list(deps, &graph);
                if !distinct_list.trim().is_empty() {
                    print!("{}", distinct_list);
                }

                // Collect appendix data (don't print yet)
                let appendix = renderer.render_appendix_sections(deps, &graph);
                if !appendix.trim().is_empty() {
                    appendix_sections.push((ecosystem.to_uppercase().to_string(), appendix));
                }
            }
        }
    }

    appendix_sections
}

pub fn print_appendix_sections(appendix_sections: Vec<(String, String)>) {
    if appendix_sections.is_empty() {
        return;
    }

    println!("\n{}", "═".repeat(51));
    println!("📋 APPENDIX");
    println!("{}", "═".repeat(51));

    for (ecosystem, content) in appendix_sections {
        println!("\n### {}\n", ecosystem);
        print!("{}", content);
    }
}

pub fn print_summary_section(
    sbom: &Sbom,
    relationships: &[DependencyRelationship],
) {
    // Build graph with relationships to get corrected is_direct flags
    let graph =
        DependencyGraph::build_from_deps_with_relationships(&sbom.dependencies, relationships);

    // Count direct, transitive, and dev dependencies using corrected flags from graph
    let total_deps = sbom.dependencies.len();
    let mut direct_count = 0;
    let mut transitive_count = 0;
    let mut dev_count = 0;

    // Count all direct dependencies, including those that are also dev
    for d in &sbom.dependencies {
        let dep_id = format!("{}@{}", d.name, d.version);
        // Use corrected flags from graph if available, otherwise fallback to original flags
        let dep = if let Some(node) = graph.get_node(&dep_id) {
            &node.dependency
        } else {
            // Node not found in graph (e.g., no relationships available for this ecosystem)
            // Use original dependency with uncorrected flags
            d
        };

        if dep.is_dev {
            dev_count += 1;
        }
        if dep.is_direct {
            if !dep.is_dev {
                direct_count += 1;
            }
        } else {
            transitive_count += 1;
        }
    }


    // Count ecosystems
    let ecosystems: std::collections::HashSet<_> = sbom
        .dependencies
        .iter()
        .map(|d| d.ecosystem.as_str())
        .collect();
    let mut ecosystem_list: Vec<_> = ecosystems.into_iter().collect();
    ecosystem_list.sort();

    // Print summary
    println!("\n{}", "═".repeat(51));
    println!("SBOM SUMMARY");
    println!("{}", "═".repeat(51));
    println!("Project Path:     {:?}", sbom.project_path);
    println!("Generated At:     {}", sbom.generated_at);
    println!();
    println!(
        "Dependencies:     {} total ({} direct, {} transitive, {} dev)",
        total_deps, direct_count, transitive_count, dev_count
    );
    println!(
        "Ecosystems:       {} ({})",
        ecosystem_list.len(),
        ecosystem_list.join(", ")
    );

    // v1.0.6: Print scope statistics if available
    if let Some(stats) = &sbom.scope_statistics {
        let pct = |n: usize| {
            if stats.total > 0 {
                (n as f32 / stats.total as f32) * 100.0
            } else {
                0.0
            }
        };
        println!();
        println!("Dependency Scopes:");
        println!(
            "  Runtime:        {} ({:.0}%)",
            stats.runtime,
            pct(stats.runtime)
        );
        println!(
            "  Build:          {} ({:.0}%)",
            stats.build,
            pct(stats.build)
        );
        println!("  Test:           {} ({:.0}%)", stats.test, pct(stats.test));
        println!(
            "  Development:    {} ({:.0}%)",
            stats.development,
            pct(stats.development)
        );
        println!(
            "  Optional:       {} ({:.0}%)",
            stats.optional,
            pct(stats.optional)
        );
        println!(
            "  Provided:       {} ({:.0}%)",
            stats.provided,
            pct(stats.provided)
        );
        println!("  Avg Confidence: {:.1}%", stats.avg_confidence * 100.0);
    }

}

pub fn print_sbom(
    sbom: &Sbom,
    tree_style: &TreeStyle,
    compact: bool,
    relationships: &[DependencyRelationship],
) {
    // Print summary section with relationships for correct counts
    print_summary_section(sbom, relationships);

    // Print dependencies with tree visualization and collect appendix data
    // Note: ROS-specific formats can be enhanced in future iterations
    // For now, we use the standard tree format for all projects
    let appendix_sections = print_dependencies_tree(sbom, tree_style, compact, relationships);

    // Print appendix sections last
    print_appendix_sections(appendix_sections);

    println!("\n{}", "═".repeat(51));
}

pub fn save_console_report(
    sbom: &Sbom,
    path: &str,
    tree_style: &TreeStyle,
    relationships: &[DependencyRelationship],
    summary_only: bool,
) -> Result<()> {
    let mut output = String::new();

    // Generate markdown report similar to print_sbom but to string
    writeln!(output, "# SBOM Report\n")?;
    writeln!(
        output,
        "**Project Path:** `{}`",
        sbom.project_path.display()
    )?;
    writeln!(output, "**Generated At:** {}\n", sbom.generated_at)?;

    // Build graph early to get corrected counts
    let graph =
        DependencyGraph::build_from_deps_with_relationships(&sbom.dependencies, relationships);

    // Add summary section - use corrected counts from graph
    // Optimize: count in a single pass without creating ID strings repeatedly
    let mut direct_count = 0;
    let mut transitive_count = 0;
    let mut dev_count = 0;
    for d in &sbom.dependencies {
        let dep_id = format!("{}@{}", d.name, d.version);
        let dep = if let Some(node) = graph.get_node(&dep_id) {
            &node.dependency
        } else {
            d // fallback to original flags
        };
        if dep.is_dev {
            dev_count += 1;
        }
        if dep.is_direct {
            if !dep.is_dev {
                direct_count += 1;
            }
        } else {
            transitive_count += 1;
        }
    }

    let ecosystems: std::collections::HashSet<_> = sbom
        .dependencies
        .iter()
        .map(|d| d.ecosystem.as_str())
        .collect();

    writeln!(output, "## Summary\n")?;
    writeln!(
        output,
        "**Dependencies:** {} total ({} direct, {} transitive, {} dev)",
        sbom.dependencies.len(),
        direct_count,
        transitive_count,
        dev_count
    )?;
    writeln!(
        output,
        "**Ecosystems:** {} ({})",
        ecosystems.len(),
        ecosystems
            .iter()
            .map(|s| s.to_uppercase())
            .collect::<Vec<_>>()
            .join(", ")
    )?;

    // v1.0.6: Add scope statistics if available
    if let Some(stats) = &sbom.scope_statistics {
        let pct = |n: usize| {
            if stats.total > 0 {
                (n as f32 / stats.total as f32) * 100.0
            } else {
                0.0
            }
        };
        writeln!(output, "\n**Dependency Scopes:**")?;
        writeln!(
            output,
            "- Runtime: {} ({:.0}%)",
            stats.runtime,
            pct(stats.runtime)
        )?;
        writeln!(
            output,
            "- Build: {} ({:.0}%)",
            stats.build,
            pct(stats.build)
        )?;
        writeln!(output, "- Test: {} ({:.0}%)", stats.test, pct(stats.test))?;
        writeln!(
            output,
            "- Development: {} ({:.0}%)",
            stats.development,
            pct(stats.development)
        )?;
        writeln!(
            output,
            "- Optional: {} ({:.0}%)",
            stats.optional,
            pct(stats.optional)
        )?;
        writeln!(
            output,
            "- Provided: {} ({:.0}%)",
            stats.provided,
            pct(stats.provided)
        )?;
        writeln!(
            output,
            "- Average Confidence: {:.1}%\n",
            stats.avg_confidence * 100.0
        )?;
    }

    // Create renderer for dependencies sections
    let renderer = TreeRenderer::new(tree_style.clone());

    // Skip dependency tree section if summary_only mode is enabled
    if summary_only {
        writeln!(output, "\n---\n")?;
        writeln!(
            output,
            "*Summary-only mode: Dependency trees omitted for smaller file size.*\n"
        )?;
        writeln!(
            output,
            "*Run without `--summary-only` flag to see full dependency trees.*\n"
        )?;
    } else {
        // Dependencies section
        writeln!(output, "## 📦 Dependencies\n")?;

        // ROS multi-package output
        if !sbom.ros_packages.is_empty() {
            for ros_pkg in &sbom.ros_packages {
                writeln!(
                    output,
                    "### {} v{}",
                    ros_pkg.metadata.name, ros_pkg.metadata.version
                )?;
                writeln!(
                    output,
                    "**Source:** `{}`",
                    ros_pkg.metadata.source_file.display()
                )?;
                writeln!(output, "**Dependencies:** {}\n", ros_pkg.dependencies.len())?;

                // Group by ecosystem
                let mut by_ecosystem: std::collections::HashMap<String, Vec<&Dependency>> =
                    std::collections::HashMap::new();
                for dep in &ros_pkg.dependencies {
                    by_ecosystem
                        .entry(dep.ecosystem.clone())
                        .or_default()
                        .push(dep);
                }

                let mut ecosystem_keys: Vec<_> = by_ecosystem.keys().collect();
                ecosystem_keys.sort();
                for ecosystem in ecosystem_keys {
                    let deps = &by_ecosystem[ecosystem];
                    writeln!(
                        output,
                        "#### {} ({} packages)\n",
                        ecosystem.to_uppercase(),
                        deps.len()
                    )?;
                    writeln!(output, "```")?;
                    // For ROS multi-package output, show all direct dependencies with tree structure
                    // This matches the behavior of regular (non-ROS) project reports
                    let all_direct_deps: Vec<String> = deps
                        .iter()
                        .filter_map(|d| {
                            let dep_id = format!("{}@{}", d.name, d.version);
                            graph.get_node(&dep_id).and_then(|node| {
                                // Include all direct dependencies (both prod and dev)
                                if node.dependency.is_direct {
                                    Some(dep_id)
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();

                    if !all_direct_deps.is_empty() {
                        let tree_output = renderer.render_tree_classic(&all_direct_deps, &graph);
                        write!(output, "{}", tree_output)?;
                    } else {
                        writeln!(output, "(no dependencies)")?;
                    }
                    writeln!(output, "```\n")?;
                }
            }

            // Total unique dependencies
            let mut all_deps: std::collections::HashSet<(&str, &str)> =
                std::collections::HashSet::new();
            for ros_pkg in &sbom.ros_packages {
                for dep in &ros_pkg.dependencies {
                    all_deps.insert((&dep.name, &dep.version));
                }
            }
            writeln!(
                output,
                "**Total Unique Dependencies Across All Packages:** {}\n",
                all_deps.len()
            )?;
        } else {
            // Group by ecosystem
            let mut by_ecosystem: std::collections::HashMap<String, Vec<&Dependency>> =
                std::collections::HashMap::new();
            for dep in &sbom.dependencies {
                by_ecosystem
                    .entry(dep.ecosystem.clone())
                    .or_default()
                    .push(dep);
            }

            let mut ecosystem_keys: Vec<_> = by_ecosystem.keys().collect();
            ecosystem_keys.sort();
            for ecosystem in ecosystem_keys {
                let deps = &by_ecosystem[ecosystem];
                // Calculate counts using corrected flags from graph in a single pass
                let mut direct_count = 0;
                let mut transitive_count = 0;
                let mut dev_count = 0;

                for d in deps.iter() {
                    let dep_id = format!("{}@{}", d.name, d.version);
                    if let Some(node) = graph.get_node(&dep_id) {
                        let dep = &node.dependency;
                        if dep.is_dev {
                            dev_count += 1;
                        }
                        if dep.is_direct {
                            if !dep.is_dev {
                                direct_count += 1;
                            }
                        } else {
                            transitive_count += 1;
                        }
                    }
                }

                writeln!(
                    output,
                    "### {} ({} packages)\n",
                    ecosystem.to_uppercase(),
                    deps.len()
                )?;

                // Main tree section - only direct production dependencies
                writeln!(output, "```")?;
                let tree_output = renderer.render_dependency_list(deps, &graph);
                if !tree_output.trim().is_empty() {
                    write!(output, "{}", tree_output)?;
                } else {
                    writeln!(output, "(no direct production dependencies)")?;
                }
                writeln!(output, "```\n")?;

                // Distinct packages list section
                writeln!(
                    output,
                    "#### {} Distinct Packages List ({} packages)\n",
                    ecosystem.to_uppercase(),
                    deps.len()
                )?;
                writeln!(
                    output,
                    "{} direct, {} transitive, {} dev\n",
                    direct_count, transitive_count, dev_count
                )?;
                writeln!(output, "```")?;
                let distinct_list = renderer.render_distinct_packages_list(deps, &graph);
                write!(output, "{}", distinct_list)?;
                writeln!(output, "```\n")?;
            }
        }

        // AI Model Details section (v1.0.9)
        let ai_models: Vec<&Dependency> = sbom
            .dependencies
            .iter()
            .filter(|d| d.ai_model_metadata.is_some())
            .collect();
        if !ai_models.is_empty() {
            writeln!(output, "## 🤖 AI Model Details\n")?;
            for dep in &ai_models {
                if let Some(ref meta) = dep.ai_model_metadata {
                    writeln!(output, "### {}\n", dep.name)?;
                    writeln!(output, "| Field | Value |")?;
                    writeln!(output, "|---|---|")?;
                    let ecosystem_label = if dep.ecosystem == "safetensors" {
                        "Safetensors (HuggingFace)"
                    } else {
                        "GGUF"
                    };
                    writeln!(output, "| **Ecosystem** | {} |", ecosystem_label)?;
                    if let Some(ref arch) = meta.architecture {
                        writeln!(output, "| **Architecture** | {} |", arch)?;
                    }
                    if let Some(ref quant) = meta.quantization {
                        writeln!(output, "| **Quantization** | {} |", quant)?;
                    }
                    if let Some(ref size) = meta.size_label {
                        writeln!(output, "| **Size Label** | {} |", size)?;
                    }
                    if let Some(count) = meta.parameter_count {
                        writeln!(output, "| **Parameters (declared)** | {} |", count)?;
                    }
                    if let Some(computed) = meta.computed_parameter_count {
                        writeln!(output, "| **Parameters (computed from tensors)** | {} |", computed)?;
                    }
                    // Integrity verification: parameter_count vs computed
                    match (meta.parameter_count, meta.computed_parameter_count) {
                        (Some(declared), Some(computed)) if declared != computed => {
                            writeln!(
                                output,
                                "| **⚠️ Integrity Check (parameter_count)** | **MISMATCH** — declared {} ≠ computed {} |",
                                declared, computed
                            )?;
                        }
                        (Some(_), Some(_)) => {
                            writeln!(output, "| **✅ Integrity Check (parameter_count)** | PASSED — declared matches computed |")?;
                        }
                        (None, Some(_)) => {
                            writeln!(output, "| **ℹ️ Integrity Check (parameter_count)** | No declared count — computed only |")?;
                        }
                        _ => {}
                    }
                    // Integrity verification: size_label vs computed
                    if let (Some(ref size_label), Some(computed)) = (&meta.size_label, meta.computed_parameter_count) {
                        if let Some(expected) = crate::parsers::gguf::parse_size_label_public(size_label) {
                            let lower = (expected as f64 * 0.95) as u64;
                            let upper = (expected as f64 * 1.05) as u64;
                            if computed < lower || computed > upper {
                                writeln!(
                                    output,
                                    "| **⚠️ Integrity Check (size_label)** | **MISMATCH** — size_label '{}' (~{} params) ≠ computed {} |",
                                    size_label, expected, computed
                                )?;
                            } else {
                                writeln!(
                                    output,
                                    "| **✅ Integrity Check (size_label)** | PASSED — '{}' consistent with computed {} |",
                                    size_label, computed
                                )?;
                            }
                        }
                    }
                    if let Some(count) = meta.tensor_count {
                        writeln!(output, "| **Tensor Count** | {} |", count)?;
                    }
                    // v1.0.11: Safetensors-specific fields
                    if let Some(shards) = meta.shard_count {
                        writeln!(output, "| **Shard Count** | {} |", shards)?;
                    }
                    if let Some(total) = meta.total_size_bytes {
                        writeln!(
                            output,
                            "| **Total Size** | {:.2} GB |",
                            total as f64 / 1_073_741_824.0
                        )?;
                    }
                    if let Some(ref dtype) = meta.torch_dtype {
                        writeln!(output, "| **Torch Dtype** | {} |", dtype)?;
                    }
                    if let Some(ref tv) = meta.transformers_version {
                        writeln!(output, "| **Transformers Version** | {} |", tv)?;
                    }
                    if let Some(vocab) = meta.vocab_size {
                        writeln!(output, "| **Vocab Size** | {} |", vocab)?;
                    }
                    if let Some(ref license) = dep.license {
                        writeln!(output, "| **License** | {} |", license)?;
                    }
                    if let Some(ref author) = dep.author {
                        writeln!(output, "| **Author** | {} |", author)?;
                    }
                    if !meta.base_models.is_empty() {
                        for (i, bm) in meta.base_models.iter().enumerate() {
                            let bm_desc = bm.name.as_deref().unwrap_or("unknown");
                            let bm_org = bm.organization.as_deref().unwrap_or("");
                            let bm_url = bm.repo_url.as_deref().unwrap_or("");
                            if !bm_org.is_empty() {
                                writeln!(output, "| **Base Model [{}]** | {} by {} ({}) |", i, bm_desc, bm_org, bm_url)?;
                            } else {
                                writeln!(output, "| **Base Model [{}]** | {} |", i, bm_desc)?;
                            }
                        }
                    }
                    if let Some(ref desc) = meta.description {
                        writeln!(output, "| **Description** | {} |", desc)?;
                    }
                    // v1.0.12: Architecture
                    if let Some(ref val) = meta.model_type {
                        writeln!(output, "| **Model Type** | {} |", val)?;
                    }
                    if let Some(val) = meta.num_hidden_layers {
                        writeln!(output, "| **Hidden Layers** | {} |", val)?;
                    }
                    if let Some(val) = meta.hidden_size {
                        writeln!(output, "| **Hidden Size** | {} |", val)?;
                    }
                    if let Some(val) = meta.num_attention_heads {
                        writeln!(output, "| **Attention Heads** | {} |", val)?;
                    }
                    if let Some(val) = meta.max_position_embeddings {
                        writeln!(output, "| **Context Window** | {} |", val)?;
                    }
                    // v1.0.12: Multimodal
                    let mut modalities = vec!["text".to_string()];
                    if meta.has_vision == Some(true) { modalities.push("vision".to_string()); }
                    if meta.has_audio == Some(true) { modalities.push("audio".to_string()); }
                    if meta.has_video == Some(true) { modalities.push("video".to_string()); }
                    if modalities.len() > 1 {
                        writeln!(output, "| **Modalities** | {} |", modalities.join(", "))?;
                    }
                    if !meta.sub_models.is_empty() {
                        writeln!(output)?;
                        writeln!(output, "#### Sub-Models\n")?;
                        writeln!(output, "| Modality | Model Type | Layers | Hidden | Heads | Dtype | Extra |")?;
                        writeln!(output, "|----------|-----------|--------|--------|-------|-------|-------|")?;
                        for sm in &meta.sub_models {
                            let mt = sm.model_type.as_deref().unwrap_or("-");
                            let layers = sm.num_hidden_layers.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
                            let hidden = sm.hidden_size.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
                            let heads = sm.num_attention_heads.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
                            let dtype = sm.dtype.as_deref().unwrap_or("-");
                            let mut extras = Vec::new();
                            if let Some(v) = sm.vocab_size { extras.push(format!("vocab={}", v)); }
                            if let Some(v) = sm.max_position_embeddings { extras.push(format!("context={}", v)); }
                            if let Some(v) = sm.patch_size { extras.push(format!("patch={}", v)); }
                            if let Some(v) = sm.default_output_length { extras.push(format!("tokens/img={}", v)); }
                            if let Some(v) = sm.conv_kernel_size { extras.push(format!("conv_kernel={}", v)); }
                            if let Some(v) = sm.output_proj_dims { extras.push(format!("output_proj={}", v)); }
                            let extra_str = if extras.is_empty() { "-".to_string() } else { extras.join(", ") };
                            writeln!(output, "| {} | {} | {} | {} | {} | {} | {} |",
                                sm.modality, mt, layers, hidden, heads, dtype, extra_str)?;
                        }
                        writeln!(output)?;
                    }
                    if let Some(ref val) = meta.processor_class {
                        writeln!(output, "| **Processor Class** | {} |", val)?;
                    }
                    if let Some(ref val) = meta.image_processor_type {
                        writeln!(output, "| **Image Processor** | {} |", val)?;
                    }
                    if let Some(ref val) = meta.audio_feature_extractor_type {
                        writeln!(output, "| **Audio Extractor** | {} |", val)?;
                    }
                    if let Some(ref val) = meta.video_processor_type {
                        writeln!(output, "| **Video Processor** | {} |", val)?;
                    }
                    // v1.0.12: Generation
                    if let Some(val) = meta.generation_temperature {
                        writeln!(output, "| **Generation Temp** | {} |", val)?;
                    }
                    if let Some(val) = meta.generation_top_k {
                        writeln!(output, "| **Generation Top-K** | {} |", val)?;
                    }
                    if let Some(val) = meta.generation_top_p {
                        writeln!(output, "| **Generation Top-P** | {} |", val)?;
                    }
                    // v1.0.12: Provenance
                    if let Some(ref val) = meta.pipeline_tag {
                        writeln!(output, "| **Pipeline Tag** | {} |", val)?;
                    }
                    if let Some(ref val) = meta.quantized_by {
                        writeln!(output, "| **Quantized By** | {} |", val)?;
                    }
                    if let Some(ref pt) = meta.prompt_template {
                        let display = if pt.chars().count() > 80 {
                            format!("{}...", pt.chars().take(80).collect::<String>())
                        } else {
                            pt.clone()
                        };
                        writeln!(output, "| **Prompt Template** | {} |", display)?;
                    }
                    if meta.is_adapter == Some(true) {
                        if let Some(ref val) = meta.adapter_type {
                            writeln!(output, "| **Adapter** | {} |", val)?;
                        } else {
                            writeln!(output, "| **Adapter** | yes |")?;
                        }
                    }
                    writeln!(output)?;
                }
            }
        }

        // Collect appendix sections
        let mut appendix_data = String::new();
        if sbom.ros_packages.is_empty() {
            // Group by ecosystem
            let mut by_ecosystem: std::collections::HashMap<String, Vec<&Dependency>> =
                std::collections::HashMap::new();
            for dep in &sbom.dependencies {
                by_ecosystem
                    .entry(dep.ecosystem.clone())
                    .or_default()
                    .push(dep);
            }

            for (ecosystem, deps) in by_ecosystem.iter() {
                let appendix = renderer.render_appendix_sections(deps, &graph);
                if !appendix.trim().is_empty() {
                    if appendix_data.is_empty() {
                        writeln!(&mut appendix_data, "## 📋 Appendix\n")?;
                    }
                    writeln!(&mut appendix_data, "### {}\n", ecosystem.to_uppercase())?;

                    // Split appendix into sections based on #### headings
                    let sections: Vec<&str> = appendix.split("#### ").collect();
                    for (idx, section) in sections.iter().enumerate() {
                        if idx == 0 && section.trim().is_empty() {
                            continue; // Skip empty first section before first ####
                        }

                        if let Some(newline_pos) = section.find('\n') {
                            let heading = &section[..newline_pos];
                            let content = &section[newline_pos + 1..];

                            writeln!(&mut appendix_data, "#### {}", heading)?;
                            writeln!(&mut appendix_data, "\n```")?;
                            write!(&mut appendix_data, "{}", content)?;
                            writeln!(&mut appendix_data, "```\n")?;
                        }
                    }
                }
            }
        }

        // Append the appendix sections at the end
        if !appendix_data.is_empty() {
            write!(output, "{}", appendix_data)?;
        }
    } // End of summary_only if-else

    fs::write(path, output)?;
    Ok(())
}


