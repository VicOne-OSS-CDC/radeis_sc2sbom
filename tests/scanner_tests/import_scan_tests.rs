use radeis_sc2sbom::models::DependencySource;
use radeis_sc2sbom::parsers::{scan_go_imports, scan_js_ts_imports, scan_python_imports};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_scan_python_imports_basic() {
    let content = r#"
import requests
import flask
from django.core import utils
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_python_imports(temp_file.path()).unwrap();

    assert_eq!(deps.len(), 3);
    assert!(deps.iter().any(|d| d.name == "requests"));
    assert!(deps.iter().any(|d| d.name == "flask"));
    assert!(deps.iter().any(|d| d.name == "django"));

    // All should have version "detected"
    for dep in &deps {
        assert_eq!(dep.version, "detected");
        assert_eq!(dep.ecosystem, "pip");
        assert!(matches!(dep.source, DependencySource::ImportScan));
    }
}

#[test]
fn test_scan_python_imports_stdlib_filtered() {
    let content = r#"
import os
import sys
import requests
from datetime import datetime
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_python_imports(temp_file.path()).unwrap();

    // Should only find requests (os, sys, datetime are stdlib)
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "requests");
}

#[test]
fn test_scan_python_imports_relative_filtered() {
    let content = r#"
from .utils import helper
from ..models import User
import requests
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_python_imports(temp_file.path()).unwrap();

    // Should only find requests (relative imports are filtered)
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "requests");
}

#[test]
fn test_scan_js_imports_require() {
    let content = r#"
const express = require('express');
const axios = require('axios');
const path = require('path'); // Node.js built-in
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_js_ts_imports(temp_file.path()).unwrap();

    // Should find express and axios (path is built-in)
    assert_eq!(deps.len(), 2);
    assert!(deps.iter().any(|d| d.name == "express"));
    assert!(deps.iter().any(|d| d.name == "axios"));

    for dep in &deps {
        assert_eq!(dep.version, "detected");
        assert_eq!(dep.ecosystem, "npm");
        assert!(matches!(dep.source, DependencySource::ImportScan));
    }
}

#[test]
fn test_scan_js_imports_es6() {
    let content = r#"
import React from 'react';
import { useState } from 'react';
import axios from 'axios';
import './styles.css'; // Relative import
import '../utils/helper'; // Relative import
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_js_ts_imports(temp_file.path()).unwrap();

    // Should find react and axios (relative imports filtered, react deduplicated)
    assert_eq!(deps.len(), 2);
    assert!(deps.iter().any(|d| d.name == "react"));
    assert!(deps.iter().any(|d| d.name == "axios"));
}

#[test]
fn test_scan_js_imports_scoped_packages() {
    let content = r#"
import '@babel/polyfill';
const core = require('@babel/core');
import { transform } from '@babel/core';
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_js_ts_imports(temp_file.path()).unwrap();

    // Should find both scoped packages (deduplicated @babel/core)
    assert_eq!(deps.len(), 2);
    assert!(deps.iter().any(|d| d.name == "@babel/polyfill"));
    assert!(deps.iter().any(|d| d.name == "@babel/core"));
}

#[test]
fn test_scan_go_imports_single() {
    let content = r#"
package main

import "github.com/gin-gonic/gin"
import "fmt" // stdlib
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_go_imports(temp_file.path()).unwrap();

    // Should find only gin (fmt is stdlib)
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "github.com/gin-gonic/gin");
    assert_eq!(deps[0].version, "detected");
    assert_eq!(deps[0].ecosystem, "go");
    assert!(matches!(deps[0].source, DependencySource::ImportScan));
}

#[test]
fn test_micropython_file_uses_micropython_ecosystem() {
    let content = r#"
import SDL
import utime as time
import lvgl as lv
import ustruct
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_python_imports(temp_file.path()).unwrap();
    assert!(
        !deps.is_empty(),
        "expected at least SDL, utime, lvgl, ustruct to be detected"
    );
    assert!(
        deps.iter().any(|d| d.name == "SDL"),
        "SDL must be in detected deps"
    );
    for dep in &deps {
        assert_eq!(
            dep.ecosystem, "micropython",
            "MicroPython import '{}' must use micropython ecosystem, got '{}'",
            dep.name, dep.ecosystem
        );
        assert_ne!(
            dep.ecosystem, "pip",
            "MicroPython import must not use pip ecosystem"
        );
    }
}

#[test]
fn test_regular_python_file_not_affected_by_micropython_detection() {
    let content = r#"
import requests
import flask
from django.core import utils
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_python_imports(temp_file.path()).unwrap();
    for dep in &deps {
        assert_eq!(
            dep.ecosystem, "pip",
            "Regular CPython import '{}' must stay as pip, got '{}'",
            dep.name, dep.ecosystem
        );
    }
}

#[test]
fn test_scan_go_imports_block() {
    let content = r#"
package main

import (
    "fmt"
    "os"
    "github.com/spf13/cobra"
    "github.com/gin-gonic/gin"
)
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = scan_go_imports(temp_file.path()).unwrap();

    // Should find cobra and gin (fmt, os are stdlib)
    assert_eq!(deps.len(), 2);
    assert!(deps.iter().any(|d| d.name == "github.com/spf13/cobra"));
    assert!(deps.iter().any(|d| d.name == "github.com/gin-gonic/gin"));
}
