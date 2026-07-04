//! Integration tests for golden/snapshot testing of the Motarjim compiler.
//!
//! These tests compile HTML/CSS fixtures and compare the output against
//! saved golden files for each platform (Flutter, Compose, SwiftUI).
//!
//! Set `UPDATE_EXPECT=1` to automatically update golden files when
//! the compiler output changes intentionally.

use std::path::Path;

/// Run all golden tests using the `GoldenTestRunner`.
///
/// Set `UPDATE_EXPECT=1` to regenerate all golden output files.
#[test]
fn golden_tests_all_platforms() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let runner = motarjim_test_utils::GoldenTestRunner::new(golden_dir);
    let update_mode = runner.is_update_mode();

    let tests = runner
        .discover()
        .expect("Failed to discover golden tests");

    if tests.is_empty() {
        panic!("No golden test fixtures found in tests/golden/html/");
    }

    for test in &tests {
        let platform_label = match test.platform {
            motarjim_config::OutputFormat::Dart => "flutter",
            motarjim_config::OutputFormat::Kotlin => "compose",
            motarjim_config::OutputFormat::Swift => "swiftui",
        };

        let result = runner.run_test(test);
        if let Err(msg) = &result {
            if update_mode {
                panic!("{msg}");
            }
        }
        assert!(
            result.is_ok(),
            "Golden test failed for {}/{}: {}",
            test.name,
            platform_label,
            result.unwrap_err()
        );
    }
}

/// Test that the runner correctly discovers HTML fixtures.
#[test]
fn test_discover_fixtures() {
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let runner = motarjim_test_utils::GoldenTestRunner::new(golden_dir);
    let tests = runner.discover().expect("Failed to discover tests");

    // We have 5 HTML fixtures x 3 platforms = 15 test cases
    assert_eq!(
        tests.len(),
        15,
        "Expected 15 test cases (5 fixtures x 3 platforms), got {}",
        tests.len()
    );

    // Verify we have all fixture names
    let names: std::collections::HashSet<String> =
        tests.iter().map(|t| t.name.clone()).collect();
    assert!(names.contains("simple-div"));
    assert!(names.contains("nested-elements"));
    assert!(names.contains("form-with-inputs"));
    assert!(names.contains("navigation-bar"));
    assert!(names.contains("card-grid"));
}
