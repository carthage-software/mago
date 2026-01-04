# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added
- Formatter: new opt-in setting `empty_line_after_class_like_open` to force an empty line after the opening brace of class-like structures (classes, interfaces, traits, enums). This is disabled by default to preserve existing formatting behavior.

### Fixed
- Tests: added unit tests verifying the behavior for enabling `empty_line_after_class_like_open`, including Drupal-style same-line braces.
