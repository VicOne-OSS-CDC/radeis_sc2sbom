use crate::models::{Dependency, DependencyScope, DependencySource};
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

pub fn parse_pom_xml(path: &Path) -> Result<Vec<Dependency>> {
    let _content =
        fs::read_to_string(path).context(format!("Failed to read pom.xml at {:?}", path))?;

    // Maven pom.xml detected (basic parsing only)
    Ok(vec![])
}

/// Gradle configuration types and their corresponding dependency scopes.
fn gradle_config_to_scope(config: &str) -> (DependencyScope, bool) {
    match config {
        "testImplementation" | "testCompile" | "testApi" | "testRuntimeOnly"
        | "testCompileOnly" | "androidTestImplementation" | "androidTestCompile" => {
            (DependencyScope::Test, true)
        }
        "compileOnly" | "compileOnlyApi" => (DependencyScope::Provided, false),
        "runtimeOnly" => (DependencyScope::Runtime, false),
        "annotationProcessor" | "kapt" | "ksp" | "classpath" | "checkstyle" => {
            (DependencyScope::Build, false)
        }
        "developmentOnly" => (DependencyScope::Development, true),
        // implementation, api, compile are all runtime
        _ => (DependencyScope::Runtime, false),
    }
}

/// Parse a Gradle build.gradle file (Groovy DSL) for dependencies.
///
/// Supports:
/// - String notation: `implementation 'group:artifact:version'`
/// - Double-quoted strings: `implementation "group:artifact:version"`
/// - Map notation: `implementation group: 'g', name: 'a', version: 'v'`
/// - Platform/BOM: `implementation platform('group:artifact:version')`
/// - Version variables: extracts group:artifact and preserves version tokens such as `$var` or `${var}`; only missing or empty versions are treated as "unspecified"
///
/// Known limitations:
/// - Multi-line map notation (split across lines) is not supported
/// - GString interpolation (`"$var"`) captures the literal `${var}` as version
pub fn parse_gradle_build(path: &Path) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read build.gradle at {:?}", path))?;

    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_prefix = format!(
        "Identified by the java/gradle extractor from {}",
        absolute_path.display()
    );

    parse_gradle_groovy_content(&content, &source_prefix)
}

/// Parse a Gradle build.gradle.kts file (Kotlin DSL) for dependencies.
///
/// Supports:
/// - Function notation: `implementation("group:artifact:version")`
/// - Platform/BOM: `implementation(platform("group:artifact:version"))`
///
/// Known limitations:
/// - Version catalog aliases such as `implementation(libs.foo)` are not currently supported
/// - Multi-line map notation (split across lines) is not supported
pub fn parse_gradle_kts_build(path: &Path) -> Result<Vec<Dependency>> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read build.gradle.kts at {:?}", path))?;

    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_prefix = format!(
        "Identified by the java/gradle-kts extractor from {}",
        absolute_path.display()
    );

    parse_gradle_kts_content(&content, &source_prefix)
}

/// Declarable Gradle dependency configurations.
/// Note: resolvable-only configs like `runtimeClasspath` are intentionally excluded
/// because users do not declare dependencies with them in build files.
/// Project-specific custom configs (e.g., `db`, `integTestImplementation`) are not
/// included — only standard Gradle, Android, and common plugin configs are supported.
const GRADLE_CONFIGS: &[&str] = &[
    "implementation",
    "api",
    "compile",
    "compileOnly",
    "compileOnlyApi",
    "runtimeOnly",
    "testImplementation",
    "testCompile",
    "testApi",
    "testRuntimeOnly",
    "testCompileOnly",
    "annotationProcessor",
    "kapt",
    "ksp",
    "androidTestImplementation",
    "androidTestCompile",
    "classpath",
    // Spring Boot / common plugin configs
    "developmentOnly",
    "checkstyle",
];

// Lazily compiled regex patterns (compiled once, reused across all calls)
static CONFIG_PATTERN: LazyLock<String> = LazyLock::new(|| GRADLE_CONFIGS.join("|"));

// Groovy DSL patterns
static GROOVY_STRING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"(?m)^\s*({configs})\s+['"]([^'"]+:[^'"]+)['"]"#,
        configs = *CONFIG_PATTERN
    ))
    .unwrap()
});

static GROOVY_MAP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"(?m)^\s*({configs})\s+group:\s*['"]([^'"]+)['"],\s*name:\s*['"]([^'"]+)['"],\s*version:\s*['"]([^'"]+)['"]"#,
        configs = *CONFIG_PATTERN
    ))
    .unwrap()
});

static GROOVY_PLATFORM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"(?m)^\s*({configs})\s+(?:enforcedPlatform|platform)\(\s*['"]([^'"]+:[^'"]+)['"]\s*\)"#,
        configs = *CONFIG_PATTERN
    ))
    .unwrap()
});

// Kotlin DSL patterns
static KTS_FUNC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"(?m)^\s*({configs})\(\s*"([^"]+:[^"]+)"\s*\)"#,
        configs = *CONFIG_PATTERN
    ))
    .unwrap()
});

static KTS_PLATFORM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r#"(?m)^\s*({configs})\(\s*(?:enforcedPlatform|platform)\(\s*"([^"]+:[^"]+)"\s*\)\s*\)"#,
        configs = *CONFIG_PATTERN
    ))
    .unwrap()
});

/// Strip comments from Gradle/Groovy/Kotlin source content.
/// Handles both single-line (`//`) and multi-line (`/* ... */`) comments.
/// Respects single-quoted and double-quoted strings — comment markers inside
/// string literals are preserved (e.g., `url "https://..."` is not stripped).
/// Uses char-based iteration to correctly handle non-ASCII / Unicode content.
fn strip_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_block_comment = false;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                in_block_comment = false;
                chars.next();
            } else if ch == '\n' {
                // Preserve newlines inside block comments so line-based regex still works
                result.push('\n');
            }
        } else if in_single_quote {
            result.push(ch);
            if ch == '\\' {
                // Push escaped char as-is
                if let Some(escaped) = chars.next() {
                    result.push(escaped);
                }
            } else if ch == '\'' {
                in_single_quote = false;
            }
        } else if in_double_quote {
            result.push(ch);
            if ch == '\\' {
                // Push escaped char as-is
                if let Some(escaped) = chars.next() {
                    result.push(escaped);
                }
            } else if ch == '"' {
                in_double_quote = false;
            }
        } else if ch == '\'' {
            in_single_quote = true;
            result.push(ch);
        } else if ch == '"' {
            in_double_quote = true;
            result.push(ch);
        } else if ch == '/' && chars.peek() == Some(&'*') {
            in_block_comment = true;
            chars.next();
        } else if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            // Skip until end of line
            for next_ch in chars.by_ref() {
                if next_ch == '\n' {
                    result.push('\n');
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn parse_gradle_groovy_content(content: &str, source_prefix: &str) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();
    let stripped = strip_comments(content);

    // Parse map notation first (more specific)
    for cap in GROOVY_MAP_RE.captures_iter(&stripped) {
        let config = cap.get(1).unwrap().as_str();
        let group = cap.get(2).unwrap().as_str();
        let artifact = cap.get(3).unwrap().as_str();
        let version = cap.get(4).unwrap().as_str();
        let (scope, is_dev) = gradle_config_to_scope(config);

        dependencies.push(create_gradle_dep(
            group,
            artifact,
            version,
            config,
            is_dev,
            scope,
            source_prefix,
        ));
    }

    // Parse platform/BOM notation
    for cap in GROOVY_PLATFORM_RE.captures_iter(&stripped) {
        let config = cap.get(1).unwrap().as_str();
        let coord = cap.get(2).unwrap().as_str();
        if let Some(dep) = parse_maven_coordinate(coord, config, source_prefix) {
            dependencies.push(dep);
        }
    }

    // Parse string notation — skip lines already matched by map or platform
    for cap in GROOVY_STRING_RE.captures_iter(&stripped) {
        let full_match = cap.get(0).unwrap().as_str();
        // Skip if it looks like a map notation or platform line
        if full_match.contains("group:") || full_match.contains("platform(") {
            continue;
        }
        let config = cap.get(1).unwrap().as_str();
        let coord = cap.get(2).unwrap().as_str();
        if let Some(dep) = parse_maven_coordinate(coord, config, source_prefix) {
            dependencies.push(dep);
        }
    }

    // Deduplicate by (name, version) keeping first occurrence
    let mut seen = std::collections::HashSet::new();
    dependencies.retain(|dep| seen.insert((dep.name.clone(), dep.version.clone())));

    Ok(dependencies)
}

fn parse_gradle_kts_content(content: &str, source_prefix: &str) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();
    let stripped = strip_comments(content);

    // Parse platform/BOM first (more specific)
    for cap in KTS_PLATFORM_RE.captures_iter(&stripped) {
        let config = cap.get(1).unwrap().as_str();
        let coord = cap.get(2).unwrap().as_str();
        if let Some(dep) = parse_maven_coordinate(coord, config, source_prefix) {
            dependencies.push(dep);
        }
    }

    // Parse function notation — skip platform lines
    for cap in KTS_FUNC_RE.captures_iter(&stripped) {
        let full_match = cap.get(0).unwrap().as_str();
        if full_match.contains("platform(") {
            continue;
        }
        let config = cap.get(1).unwrap().as_str();
        let coord = cap.get(2).unwrap().as_str();
        if let Some(dep) = parse_maven_coordinate(coord, config, source_prefix) {
            dependencies.push(dep);
        }
    }

    // Deduplicate by (name, version) keeping first occurrence
    let mut seen = std::collections::HashSet::new();
    dependencies.retain(|dep| seen.insert((dep.name.clone(), dep.version.clone())));

    Ok(dependencies)
}

/// Parse a Maven coordinate string "group:artifact:version" into a Dependency.
/// Returns None if the format is invalid (less than 2 colon-separated parts).
fn parse_maven_coordinate(coord: &str, config: &str, source_prefix: &str) -> Option<Dependency> {
    let parts: Vec<&str> = coord.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let group = parts[0];
    let artifact = parts[1];
    let version = if parts.len() >= 3 && !parts[2].is_empty() {
        parts[2]
    } else {
        "unspecified"
    };

    let (scope, is_dev) = gradle_config_to_scope(config);

    Some(create_gradle_dep(
        group,
        artifact,
        version,
        config,
        is_dev,
        scope,
        source_prefix,
    ))
}

fn create_gradle_dep(
    group: &str,
    artifact: &str,
    version: &str,
    config: &str,
    is_dev: bool,
    scope: DependencyScope,
    source_prefix: &str,
) -> Dependency {
    Dependency {
        name: format!("{}:{}", group, artifact),
        version: version.to_string(),
        ecosystem: "maven".to_string(),
        source: DependencySource::Manifest,
        is_dev,
        is_direct: true,
        source_file: Some(format!("{} config:{}", source_prefix, config)),
        scope,
        scope_confidence: 1.0,
        scope_reason: format!("Gradle {} configuration", config),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Comment stripping tests ===

    #[test]
    fn test_strip_line_comments() {
        let content = "// this is a comment\nimplementation 'a:b:1'\n";
        let stripped = strip_comments(content);
        assert!(!stripped.contains("this is a comment"));
        assert!(stripped.contains("implementation"));
    }

    #[test]
    fn test_strip_block_comments() {
        let content = "/* block */\nimplementation 'a:b:1'\n";
        let stripped = strip_comments(content);
        assert!(!stripped.contains("block"));
        assert!(stripped.contains("implementation"));
    }

    #[test]
    fn test_strip_multiline_block_comments() {
        let content = "/*\nimplementation 'a:b:1'\n*/\nimplementation 'c:d:2'\n";
        let stripped = strip_comments(content);
        assert!(!stripped.contains("a:b:1"));
        assert!(stripped.contains("c:d:2"));
    }

    #[test]
    fn test_strip_preserves_urls_in_strings() {
        // URLs containing // inside quoted strings should NOT be stripped
        let content = r#"
repositories {
    maven { url "https://maven.example.com/repo" }
}
dependencies {
    implementation 'com.example:lib:1.0'
}
"#;
        let stripped = strip_comments(content);
        assert!(stripped.contains("https://maven.example.com/repo"));
        assert!(stripped.contains("implementation"));
    }

    #[test]
    fn test_strip_preserves_block_comment_markers_in_strings() {
        let content = r#"
ext.desc = "/* not a comment */"
dependencies {
    implementation 'com.example:lib:1.0'
}
"#;
        let stripped = strip_comments(content);
        assert!(stripped.contains("/* not a comment */"));
        assert!(stripped.contains("implementation"));
    }

    #[test]
    fn test_block_comment_inside_dependencies() {
        let content = r#"
dependencies {
    /*
    implementation 'com.example:should-be-skipped:1.0'
    */
    implementation 'com.example:real:2.0'
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com.example:real");
        assert_eq!(deps[0].version, "2.0");
    }

    // === Groovy DSL Tests ===

    #[test]
    fn test_groovy_string_notation_single_quotes() {
        let content = r#"
dependencies {
    implementation 'com.google.guava:guava:31.1-jre'
    testImplementation 'junit:junit:4.13.2'
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "com.google.guava:guava");
        assert_eq!(deps[0].version, "31.1-jre");
        assert_eq!(deps[0].ecosystem, "maven");
        assert_eq!(deps[0].scope, DependencyScope::Runtime);
        assert!(!deps[0].is_dev);

        assert_eq!(deps[1].name, "junit:junit");
        assert_eq!(deps[1].version, "4.13.2");
        assert_eq!(deps[1].scope, DependencyScope::Test);
        assert!(deps[1].is_dev);
    }

    #[test]
    fn test_groovy_string_notation_double_quotes() {
        let content = r#"
dependencies {
    implementation "org.springframework:spring-core:5.3.23"
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "org.springframework:spring-core");
        assert_eq!(deps[0].version, "5.3.23");
    }

    #[test]
    fn test_groovy_gstring_interpolation() {
        // GString with ${var} — captured as literal placeholder, which is the expected behavior
        let content = r#"
dependencies {
    implementation "com.example:lib:${libraryVersion}"
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com.example:lib");
        assert_eq!(deps[0].version, "${libraryVersion}");
    }

    #[test]
    fn test_groovy_map_notation() {
        let content = r#"
dependencies {
    implementation group: 'com.google.guava', name: 'guava', version: '31.1-jre'
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com.google.guava:guava");
        assert_eq!(deps[0].version, "31.1-jre");
    }

    #[test]
    fn test_groovy_platform_bom() {
        let content = r#"
dependencies {
    implementation platform('org.springframework.boot:spring-boot-dependencies:3.1.0')
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(
            deps[0].name,
            "org.springframework.boot:spring-boot-dependencies"
        );
        assert_eq!(deps[0].version, "3.1.0");
    }

    #[test]
    fn test_groovy_enforced_platform() {
        let content = r#"
dependencies {
    implementation enforcedPlatform('org.springframework.boot:spring-boot-dependencies:3.1.0')
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(
            deps[0].name,
            "org.springframework.boot:spring-boot-dependencies"
        );
        assert_eq!(deps[0].version, "3.1.0");
    }

    #[test]
    fn test_groovy_all_configurations() {
        let content = r#"
dependencies {
    implementation 'com.example:impl:1.0'
    api 'com.example:api:1.0'
    compileOnly 'com.example:provided:1.0'
    runtimeOnly 'com.example:runtime:1.0'
    testImplementation 'com.example:test:1.0'
    annotationProcessor 'com.example:processor:1.0'
    kapt 'com.example:kapt:1.0'
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 7);

        assert_eq!(deps[0].scope, DependencyScope::Runtime); // implementation
        assert_eq!(deps[1].scope, DependencyScope::Runtime); // api
        assert_eq!(deps[2].scope, DependencyScope::Provided); // compileOnly
        assert_eq!(deps[3].scope, DependencyScope::Runtime); // runtimeOnly
        assert_eq!(deps[4].scope, DependencyScope::Test); // testImplementation
        assert_eq!(deps[5].scope, DependencyScope::Build); // annotationProcessor
        assert_eq!(deps[6].scope, DependencyScope::Build); // kapt
    }

    #[test]
    fn test_groovy_classpath_is_build_scope() {
        let content = r#"
buildscript {
    dependencies {
        classpath 'com.android.tools.build:gradle:8.1.0'
    }
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com.android.tools.build:gradle");
        assert_eq!(deps[0].scope, DependencyScope::Build);
    }

    #[test]
    fn test_groovy_no_version() {
        let content = r#"
dependencies {
    implementation 'com.example:mylib'
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com.example:mylib");
        assert_eq!(deps[0].version, "unspecified");
    }

    #[test]
    fn test_groovy_ignores_line_comments() {
        let content = r#"
dependencies {
    // implementation 'com.example:commented:1.0'
    implementation 'com.example:real:1.0'
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com.example:real");
    }

    #[test]
    fn test_groovy_empty_file() {
        let deps = parse_gradle_groovy_content("", "test").unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_groovy_no_dependencies_block() {
        let content = r#"
plugins {
    id 'java'
}

group = 'com.example'
version = '1.0-SNAPSHOT'
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_groovy_android_project() {
        let content = r#"
dependencies {
    implementation 'androidx.core:core-ktx:1.12.0'
    implementation 'androidx.appcompat:appcompat:1.6.1'
    testImplementation 'junit:junit:4.13.2'
    androidTestImplementation 'androidx.test.ext:junit:1.1.5'
}
"#;
        let deps = parse_gradle_groovy_content(content, "test").unwrap();
        assert_eq!(deps.len(), 4);
        assert_eq!(deps[0].name, "androidx.core:core-ktx");
        assert_eq!(deps[3].name, "androidx.test.ext:junit");
        assert_eq!(deps[3].scope, DependencyScope::Test);
        assert!(deps[3].is_dev);
    }

    // === Kotlin DSL Tests ===

    #[test]
    fn test_kts_function_notation() {
        let content = r#"
dependencies {
    implementation("com.google.guava:guava:31.1-jre")
    testImplementation("junit:junit:4.13.2")
}
"#;
        let deps = parse_gradle_kts_content(content, "test").unwrap();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "com.google.guava:guava");
        assert_eq!(deps[0].version, "31.1-jre");
        assert_eq!(deps[0].scope, DependencyScope::Runtime);

        assert_eq!(deps[1].name, "junit:junit");
        assert_eq!(deps[1].version, "4.13.2");
        assert_eq!(deps[1].scope, DependencyScope::Test);
    }

    #[test]
    fn test_kts_platform_bom() {
        let content = r#"
dependencies {
    implementation(platform("org.springframework.boot:spring-boot-dependencies:3.1.0"))
}
"#;
        let deps = parse_gradle_kts_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(
            deps[0].name,
            "org.springframework.boot:spring-boot-dependencies"
        );
        assert_eq!(deps[0].version, "3.1.0");
    }

    #[test]
    fn test_kts_enforced_platform() {
        let content = r#"
dependencies {
    implementation(enforcedPlatform("org.springframework.boot:spring-boot-dependencies:3.1.0"))
}
"#;
        let deps = parse_gradle_kts_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(
            deps[0].name,
            "org.springframework.boot:spring-boot-dependencies"
        );
    }

    #[test]
    fn test_kts_all_configurations() {
        let content = r#"
dependencies {
    implementation("com.example:impl:1.0")
    api("com.example:api:1.0")
    compileOnly("com.example:provided:1.0")
    runtimeOnly("com.example:runtime:1.0")
    testImplementation("com.example:test:1.0")
    kapt("com.example:kapt:1.0")
    ksp("com.example:ksp:1.0")
}
"#;
        let deps = parse_gradle_kts_content(content, "test").unwrap();
        assert_eq!(deps.len(), 7);

        assert_eq!(deps[0].scope, DependencyScope::Runtime);
        assert_eq!(deps[1].scope, DependencyScope::Runtime);
        assert_eq!(deps[2].scope, DependencyScope::Provided);
        assert_eq!(deps[3].scope, DependencyScope::Runtime);
        assert_eq!(deps[4].scope, DependencyScope::Test);
        assert_eq!(deps[5].scope, DependencyScope::Build);
        assert_eq!(deps[6].scope, DependencyScope::Build);
    }

    #[test]
    fn test_kts_empty_file() {
        let deps = parse_gradle_kts_content("", "test").unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_kts_no_version() {
        let content = r#"
dependencies {
    implementation("com.example:mylib")
}
"#;
        let deps = parse_gradle_kts_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].version, "unspecified");
    }

    #[test]
    fn test_kts_block_comment() {
        let content = r#"
dependencies {
    /*
    implementation("com.example:skipped:1.0")
    */
    implementation("com.example:real:2.0")
}
"#;
        let deps = parse_gradle_kts_content(content, "test").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com.example:real");
    }

    // === Scope mapping tests ===

    #[test]
    fn test_gradle_config_to_scope() {
        assert_eq!(
            gradle_config_to_scope("implementation"),
            (DependencyScope::Runtime, false)
        );
        assert_eq!(
            gradle_config_to_scope("testImplementation"),
            (DependencyScope::Test, true)
        );
        assert_eq!(
            gradle_config_to_scope("compileOnly"),
            (DependencyScope::Provided, false)
        );
        assert_eq!(
            gradle_config_to_scope("annotationProcessor"),
            (DependencyScope::Build, false)
        );
        assert_eq!(
            gradle_config_to_scope("androidTestImplementation"),
            (DependencyScope::Test, true)
        );
        assert_eq!(
            gradle_config_to_scope("classpath"),
            (DependencyScope::Build, false)
        );
    }

    // === Maven coordinate parsing tests ===

    #[test]
    fn test_parse_maven_coordinate_full() {
        let dep = parse_maven_coordinate("com.google:guava:31.1", "implementation", "test");
        assert!(dep.is_some());
        let dep = dep.unwrap();
        assert_eq!(dep.name, "com.google:guava");
        assert_eq!(dep.version, "31.1");
    }

    #[test]
    fn test_parse_maven_coordinate_no_version() {
        let dep = parse_maven_coordinate("com.google:guava", "implementation", "test");
        assert!(dep.is_some());
        assert_eq!(dep.unwrap().version, "unspecified");
    }

    #[test]
    fn test_parse_maven_coordinate_invalid() {
        let dep = parse_maven_coordinate("invalid", "implementation", "test");
        assert!(dep.is_none());
    }

    #[test]
    fn test_parse_maven_coordinate_with_classifier() {
        // group:artifact:version:classifier — should still capture version
        let dep =
            parse_maven_coordinate("com.example:lib:1.0:sources", "implementation", "test");
        assert!(dep.is_some());
        let dep = dep.unwrap();
        assert_eq!(dep.name, "com.example:lib");
        assert_eq!(dep.version, "1.0");
    }

    // === Source file tracking ===

    #[test]
    fn test_source_file_contains_config() {
        let content = r#"
dependencies {
    testImplementation 'junit:junit:4.13.2'
}
"#;
        let deps = parse_gradle_groovy_content(content, "test-source").unwrap();
        assert_eq!(deps.len(), 1);
        let source = deps[0].source_file.as_ref().unwrap();
        assert!(source.contains("config:testImplementation"));
        assert!(source.contains("test-source"));
    }
}
