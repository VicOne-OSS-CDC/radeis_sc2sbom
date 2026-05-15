use radeis_sc2sbom::parsers::{
    extract_js_package, extract_python_package, is_go_stdlib, is_nodejs_builtin, is_python_stdlib,
};

#[test]
fn test_extract_python_package() {
    assert_eq!(extract_python_package("django"), "django");
    assert_eq!(extract_python_package("django.core.utils"), "django");
    assert_eq!(extract_python_package("flask.ext.sqlalchemy"), "flask");
}

#[test]
fn test_extract_js_package() {
    assert_eq!(extract_js_package("express"), "express");
    assert_eq!(extract_js_package("express/lib/router"), "express");
    assert_eq!(extract_js_package("@babel/core"), "@babel/core");
    assert_eq!(extract_js_package("@babel/core/lib"), "@babel/core");
    assert_eq!(extract_js_package("@types/node"), "@types/node");
}

#[test]
fn test_is_python_stdlib() {
    assert!(is_python_stdlib("os"));
    assert!(is_python_stdlib("sys"));
    assert!(is_python_stdlib("json"));
    assert!(!is_python_stdlib("requests"));
    assert!(!is_python_stdlib("django"));
}

#[test]
fn test_is_nodejs_builtin() {
    assert!(is_nodejs_builtin("fs"));
    assert!(is_nodejs_builtin("path"));
    assert!(is_nodejs_builtin("http"));
    assert!(!is_nodejs_builtin("express"));
    assert!(!is_nodejs_builtin("axios"));
}

#[test]
fn test_is_go_stdlib() {
    assert!(is_go_stdlib("fmt"));
    assert!(is_go_stdlib("os"));
    assert!(is_go_stdlib("net/http"));
    assert!(is_go_stdlib("golang.org/x/crypto"));
    assert!(!is_go_stdlib("github.com/gin-gonic/gin"));
    assert!(!is_go_stdlib("gopkg.in/yaml.v2"));
}
