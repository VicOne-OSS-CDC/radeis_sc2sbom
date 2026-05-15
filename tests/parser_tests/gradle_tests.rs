use radeis_sc2sbom::parsers::java::{parse_gradle_build, parse_gradle_kts_build};
use std::io::Write;
use tempfile::NamedTempFile;

// === Groovy DSL Integration Tests ===

#[test]
fn test_parse_gradle_build_spring_boot() {
    let content = r#"
plugins {
    id 'org.springframework.boot' version '3.1.0'
    id 'io.spring.dependency-management' version '1.1.0'
    id 'java'
}

group = 'com.example'
version = '0.0.1-SNAPSHOT'

repositories {
    mavenCentral()
}

dependencies {
    implementation 'org.springframework.boot:spring-boot-starter-web:3.1.0'
    implementation 'org.springframework.boot:spring-boot-starter-data-jpa:3.1.0'
    runtimeOnly 'com.h2database:h2:2.1.214'
    compileOnly 'org.projectlombok:lombok:1.18.30'
    annotationProcessor 'org.projectlombok:lombok:1.18.30'
    testImplementation 'org.springframework.boot:spring-boot-starter-test:3.1.0'
}
"#;

    let mut temp_file = NamedTempFile::with_suffix(".gradle").unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gradle_build(temp_file.path()).unwrap();
    // 5 unique (name, version) pairs — lombok appears twice but deduped
    assert_eq!(deps.len(), 5);

    // Check spring-boot-starter-web
    let web = deps.iter().find(|d| d.name.contains("starter-web")).unwrap();
    assert_eq!(web.name, "org.springframework.boot:spring-boot-starter-web");
    assert_eq!(web.version, "3.1.0");
    assert_eq!(web.ecosystem, "maven");
    assert!(web.is_direct);

    // Check h2 is runtimeOnly
    let h2 = deps.iter().find(|d| d.name.contains("h2")).unwrap();
    assert_eq!(h2.version, "2.1.214");

    // Check lombok is present (deduped — first match wins: compileOnly)
    let lombok = deps.iter().find(|d| d.name.contains("lombok")).unwrap();
    assert_eq!(lombok.version, "1.18.30");

    // Check test dependency
    let test_dep = deps
        .iter()
        .find(|d| d.name.contains("starter-test"))
        .unwrap();
    assert!(test_dep.is_dev);
    assert_eq!(
        test_dep.scope,
        radeis_sc2sbom::models::DependencyScope::Test
    );
}

#[test]
fn test_parse_gradle_build_android() {
    let content = r#"
plugins {
    id 'com.android.application'
    id 'org.jetbrains.kotlin.android'
}

android {
    compileSdk 34
}

dependencies {
    implementation 'androidx.core:core-ktx:1.12.0'
    implementation 'androidx.appcompat:appcompat:1.6.1'
    implementation 'com.google.android.material:material:1.11.0'
    testImplementation 'junit:junit:4.13.2'
    androidTestImplementation 'androidx.test.ext:junit:1.1.5'
    androidTestImplementation 'androidx.test.espresso:espresso-core:3.5.1'
}
"#;

    let mut temp_file = NamedTempFile::with_suffix(".gradle").unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gradle_build(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 6);

    // Android test deps should be test scope
    let espresso = deps
        .iter()
        .find(|d| d.name.contains("espresso"))
        .unwrap();
    assert!(espresso.is_dev);
    assert_eq!(
        espresso.scope,
        radeis_sc2sbom::models::DependencyScope::Test
    );
}

#[test]
fn test_parse_gradle_build_map_notation() {
    let content = r#"
dependencies {
    implementation group: 'com.google.guava', name: 'guava', version: '31.1-jre'
    testImplementation group: 'org.mockito', name: 'mockito-core', version: '5.3.1'
}
"#;

    let mut temp_file = NamedTempFile::with_suffix(".gradle").unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gradle_build(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 2);
    assert_eq!(deps[0].name, "com.google.guava:guava");
    assert_eq!(deps[0].version, "31.1-jre");
}

// === Kotlin DSL Integration Tests ===

#[test]
fn test_parse_gradle_kts_spring_boot() {
    let content = r#"
plugins {
    id("org.springframework.boot") version "3.1.0"
    id("io.spring.dependency-management") version "1.1.0"
    kotlin("jvm") version "1.9.0"
}

dependencies {
    implementation("org.springframework.boot:spring-boot-starter-web:3.1.0")
    implementation("org.springframework.boot:spring-boot-starter-data-jpa:3.1.0")
    runtimeOnly("com.h2database:h2:2.1.214")
    compileOnly("org.projectlombok:lombok:1.18.30")
    kapt("org.projectlombok:lombok:1.18.30")
    testImplementation("org.springframework.boot:spring-boot-starter-test:3.1.0")
}
"#;

    let mut temp_file = NamedTempFile::with_suffix(".gradle.kts").unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gradle_kts_build(temp_file.path()).unwrap();
    // 5 unique (name, version) pairs — lombok appears twice but deduped
    assert_eq!(deps.len(), 5);

    let web = deps.iter().find(|d| d.name.contains("starter-web")).unwrap();
    assert_eq!(web.ecosystem, "maven");
    assert_eq!(web.version, "3.1.0");
}

#[test]
fn test_parse_gradle_kts_platform_bom() {
    let content = r#"
dependencies {
    implementation(platform("org.springframework.boot:spring-boot-dependencies:3.1.0"))
    implementation("org.springframework.boot:spring-boot-starter-web:3.1.0")
}
"#;

    let mut temp_file = NamedTempFile::with_suffix(".gradle.kts").unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gradle_kts_build(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 2);

    let bom = deps
        .iter()
        .find(|d| d.name.contains("spring-boot-dependencies"))
        .unwrap();
    assert_eq!(bom.version, "3.1.0");
}

#[test]
fn test_parse_gradle_kts_empty_file() {
    let content = r#"
plugins {
    kotlin("jvm") version "1.9.0"
}

repositories {
    mavenCentral()
}
"#;

    let mut temp_file = NamedTempFile::with_suffix(".gradle.kts").unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gradle_kts_build(temp_file.path()).unwrap();
    assert!(deps.is_empty());
}
