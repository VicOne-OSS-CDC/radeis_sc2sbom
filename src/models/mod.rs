pub mod dependency;
pub mod graph;
pub mod sbom;

pub use dependency::{
    AIModelMetadata, AutosarMetadata, BaseModelInfo, Dependency, DependencyRelationship,
    DependencyScope, DependencySource, LockFileData, ScanContext, SubModelInfo,
};
pub use graph::{DependencyGraph, DependencyNode};
pub use sbom::{RosPackageMetadata, RosPackageWithDeps, Sbom, ScopeStatistics};
