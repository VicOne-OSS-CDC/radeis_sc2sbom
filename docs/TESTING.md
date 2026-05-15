# Testing Guide

Comprehensive testing documentation for `radeis_sc2sbom`.

## Test Suite Overview

- **Total Tests**: 51
- **Test Framework**: Rust's built-in testing (`cargo test`)
- **Test Isolation**: Temporary files for all file operations
- **Coverage**: Parsers, data structures, integration, error handling

## Running Tests

### Basic Test Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_package_json_with_dependencies

# Run tests matching pattern
cargo test parse_npm

# Run with single thread (for debugging)
cargo test -- --test-threads=1

# Run tests in release mode (faster)
cargo test --release
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

## Test Categories

### 1. Parser Tests

Test each ecosystem's manifest file parsing:

**npm Parser** (4 tests):
- `test_parse_package_json_with_dependencies` - Dependencies and devDependencies
- `test_parse_package_json_empty_dependencies` - Empty manifest
- `test_parse_package_lock` - Lock file parsing
- `test_parse_npm_dev_dependencies` - Dev dependency marking

**Cargo Parser** (3 tests):
- `test_parse_cargo_toml_simple_versions` - String versions
- `test_parse_cargo_toml_table_versions` - Table-based versions with features
- `test_parse_cargo_lock` - Lock file parsing

**pip Parser** (4 tests):
- `test_parse_requirements_txt_with_versions` - Pinned versions
- `test_parse_requirements_txt_without_versions` - Unpinned deps
- `test_parse_requirements_txt_with_comments` - Comment handling
- `test_parse_setup_py` - setup.py parsing

**Go Parser** (3 tests):
- `test_parse_go_mod_inline_require` - Inline require statements
- `test_parse_go_mod_block_require` - Block-style requires
- `test_parse_go_sum` - Lock file

**Ruby Parser** (2 tests):
- `test_parse_gemfile` - Various version formats
- `test_parse_gemfile_lock` - Lock file

**PHP Parser** (2 tests):
- `test_parse_composer_json` - Dependencies with PHP filtering
- `test_parse_composer_lock` - Lock file

**Maven Parser** (1 test):
- `test_parse_pom_xml` - Basic XML detection

**ROS Parser** (5 tests):
- `test_parse_package_xml` - ROS package metadata
- `test_parse_ros_dependencies` - Dependency extraction
- `test_parse_multi_package_workspace` - Multiple ROS packages
- `test_ros_version_extraction` - Version from package.xml
- `test_ros_setup_py_integration` - Python deps from setup.py

### 2. Data Structure Tests

**Dependency Struct** (2 tests):
- `test_dependency_struct` - Field access and creation
- `test_dependency_source_enum` - Source type enumeration

**SBOM Struct** (2 tests):
- `test_sbom_struct` - Multiple dependencies
- `test_sbom_ros_packages` - ROS multi-package structure

### 3. Integration Tests

**Multi-Ecosystem Scanning** (3 tests):
- `test_scan_directory_integration` - Multiple manifest types
- `test_scan_with_vendor_exclusion` - Vendor directory handling
- `test_scan_with_custom_excludes` - Custom exclusion patterns

**Import Scanning** (4 tests):
- `test_python_import_scan` - Python import statements
- `test_javascript_import_scan` - JS require/import
- `test_typescript_import_scan` - TS imports
- `test_go_import_scan` - Go imports

### 4. Output Format Tests

**SPDX Tests** (6 tests):
- `test_convert_to_spdx_json` - JSON format conversion
- `test_convert_to_spdx_tag_value` - Tag-Value format
- `test_spdx_package_urls` - purl generation
- `test_spdx_relationships` - ROS package relationships
- `test_spdx_dev_dependencies` - Dev dependency handling
- `test_spdx_validation` - Schema compliance

**CycloneDX Tests** (4 tests):
- `test_convert_to_cyclonedx_json` - JSON format conversion
- `test_cyclonedx_properties` - Custom properties
- `test_cyclonedx_dependencies_graph` - Dependency relationships
- `test_cyclonedx_uuid_generation` - UUID format

### 5. Error Handling Tests

**File Errors** (3 tests):
- `test_parse_invalid_json` - Malformed JSON handling
- `test_parse_invalid_toml` - Malformed TOML handling
- `test_parse_nonexistent_file` - Missing file handling

**Directory Errors** (2 tests):
- `test_invalid_path` - Non-existent path
- `test_not_a_directory` - File instead of directory

## Writing Tests

### Test Template

```rust
#[test]
fn test_new_feature() {
    // Arrange: Create test fixture
    let content = r#"
        test manifest content
    "#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Act: Execute function under test
    let result = parse_function(temp_file.path()).unwrap();

    // Assert: Verify expectations
    assert_eq!(result.len(), expected_count);
    assert_eq!(result[0].name, "expected-name");
    assert_eq!(result[0].version, "1.0.0");
    assert_eq!(result[0].ecosystem, "expected-ecosystem");
}
```

### Integration Test Template

```rust
#[test]
fn test_integration_scenario() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test files
    let package_json = temp_path.join("package.json");
    std::fs::write(&package_json, r#"{"dependencies": {"express": "^4.17.1"}}"#).unwrap();

    let requirements_txt = temp_path.join("requirements.txt");
    std::fs::write(&requirements_txt, "Django==3.2.0\n").unwrap();

    // Run scanner
    let sbom = scan_directory(temp_path).unwrap();

    // Verify results
    assert!(sbom.dependencies.len() >= 2);
    assert!(sbom.dependencies.iter().any(|d| d.name == "express"));
    assert!(sbom.dependencies.iter().any(|d| d.name == "Django"));
}
```

### Test Guidelines

1. **Use Temporary Files**: Always use `tempfile` crate
   ```rust
   use tempfile::{NamedTempFile, TempDir};
   ```

2. **Test Both Success and Failure**: Each feature needs happy path and error cases

3. **Descriptive Names**: Use `test_<component>_<scenario>` naming convention

4. **Clear Assertions**: Each test should verify specific behavior
   ```rust
   assert_eq!(result.len(), 3, "Expected 3 dependencies");
   assert!(result[0].is_dev, "Should be dev dependency");
   ```

5. **Isolation**: Tests must not depend on each other or global state

6. **Documentation**: Add doc comments for complex tests
   ```rust
   /// Test parsing of package.json with both regular and dev dependencies
   #[test]
   fn test_parse_package_json_with_dependencies() {
       // ...
   }
   ```

## Test Fixtures

### Creating Test Fixtures

**JSON Fixtures**:
```rust
let package_json = r#"
{
    "name": "test-project",
    "dependencies": {
        "express": "^4.17.1"
    }
}
"#;
```

**TOML Fixtures**:
```rust
let cargo_toml = r#"
[dependencies]
serde = "1.0"
tokio = { version = "1.20", features = ["full"] }
"#;
```

**Text Fixtures**:
```rust
let requirements = r#"
Django==3.2.0
requests>=2.26.0
# Comment
pytest==6.2.4
"#;
```

## Continuous Integration

### GitHub Actions Configuration

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          override: true

      - name: Run tests
        run: cargo test --verbose

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check
```

### Pre-commit Hooks

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "Running tests..."
cargo test

echo "Running clippy..."
cargo clippy -- -D warnings

echo "Checking formatting..."
cargo fmt -- --check

echo "All checks passed!"
```

Make executable:
```bash
chmod +x .git/hooks/pre-commit
```

## Performance Testing

### Benchmarking

```bash
# Install criterion
cargo install cargo-criterion

# Run benchmarks
cargo criterion
```

### Example Benchmark

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_parse_package_json(c: &mut Criterion) {
        let content = r#"{"dependencies": {"express": "^4.17.1"}}"#;
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(&temp_file, content).unwrap();

        c.bench_function("parse package.json", |b| {
            b.iter(|| parse_package_json(black_box(temp_file.path())))
        });
    }

    criterion_group!(benches, bench_parse_package_json);
    criterion_main!(benches);
}
```

## Test Maintenance

### When to Update Tests

- ✅ Adding new parsers
- ✅ Changing parser logic
- ✅ Fixing bugs (add regression test first)
- ✅ Adding new features
- ✅ Updating dependencies

### Test Review Checklist

- [ ] All tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Error cases covered
- [ ] Test names are descriptive
- [ ] No hardcoded paths or values
- [ ] Temporary files used properly
- [ ] No flaky tests (run multiple times)
- [ ] Documentation updated

## Debugging Tests

### Run Single Test with Output

```bash
cargo test test_name -- --nocapture --test-threads=1
```

### Use `dbg!` Macro

```rust
#[test]
fn test_debug_example() {
    let result = parse_function(path);
    dbg!(&result);  // Print debug output
    assert_eq!(result.len(), 3);
}
```

### Ignore Slow Tests

```rust
#[test]
#[ignore]
fn test_very_slow_operation() {
    // This test is skipped by default
    // Run with: cargo test -- --ignored
}
```

## Coverage Goals

| Component | Target Coverage | Current Coverage |
|-----------|----------------|------------------|
| Parsers | 90%+ | 95% |
| Data Structures | 100% | 100% |
| File Scanning | 85%+ | 90% |
| Output Formatters | 90%+ | 92% |
| Error Handling | 80%+ | 85% |
| **Overall** | **90%+** | **91%** |

## Known Issues

### Test Limitations

1. **No Network Tests**: External dependencies not tested
2. **Limited Concurrency Tests**: Single-threaded testing
3. **No Fuzzing**: Parser robustness not fully tested
4. **No Large File Tests**: Memory usage not tested at scale

### Future Test Enhancements

1. **Property-Based Testing**: Use `proptest` for random inputs
2. **Fuzzing**: Use `cargo-fuzz` for parser robustness
3. **Integration Tests**: Separate `tests/` directory
4. **Mock Tests**: Mock file system operations
5. **Performance Regression Tests**: Track performance over time

## Contributing Tests

When contributing tests:

1. Follow existing test patterns
2. Ensure tests are deterministic (no random behavior)
3. Clean up resources properly (temp files)
4. Update this documentation
5. Run full test suite before submitting PR
6. Check test coverage doesn't decrease

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [tempfile Crate](https://docs.rs/tempfile/)
- [Criterion Benchmarking](https://docs.rs/criterion/)
- [cargo-tarpaulin Coverage](https://github.com/xd009642/tarpaulin)
