# Quick Start: Newton Constraints CLI Tool

**Audience**: Project maintainers, developers, and auditors
**Time to complete**: 10 minutes

## Prerequisites

- Rust nightly toolchain (managed by `rust-toolchain.toml`)
- Basic command-line familiarity

## Installation

```bash
# Clone and build
git clone <repository-url>
cd constraints
cargo build --release

# Optional: Install globally
cargo install --path .
```

## Your First Constraint

```bash
# Initialize constraints directory (automatic on first use)
# Add your first constraint
constraint add --type MUST --category security --text "All passwords must be hashed using bcrypt" --author "your-name"

# View all constraints
constraint list

# Expected output:
# nt-a1b2c3: MUST [security] All passwords must be hashed using bcrypt
#   Author: your-name | Created: 2026-01-19 10:30:00 UTC
```

## Core Workflows

### 1. Define Project Requirements

```bash
# Add multiple constraints for different categories
constraint add --type MUST --category testing --text "All public functions must have unit tests" --author "maintainer"
constraint add --type SHOULD --category performance --text "API responses should complete within 200ms" --author "maintainer"
constraint add --type FORBIDDEN --category security --text "Never log sensitive user data" --author "maintainer"
```

### 2. Browse and Search

```bash
# List all constraints
constraint list

# Filter by category
constraint list --category security

# Search by content
constraint search "password"

# JSON output for scripting
constraint list --format json
```

### 3. Update Requirements

```bash
# Update a constraint (find ID from list command)
constraint patch nt-a1b2c3 --text "All passwords must be hashed using argon2id" --verification "cargo test --test password-security"
```

### 4. Validate Compliance

```bash
# Validate all constraints (checks format only)
constraint validate

# Validate with verification execution
constraint validate --execute

# Validate specific category
constraint validate --category security --execute
```

## Common Patterns

### Security Requirements
```bash
constraint add --type MUST --category security --text "Input validation required for all user data" --verification "./scripts/check-input-validation.sh"
constraint add --type FORBIDDEN --category security --text "No hardcoded secrets in source code" --verification "grep -r 'password\|secret\|key' src/ || true"
```

### Testing Standards
```bash
constraint add --type MUST --category testing --text "Test coverage must be >80%" --verification "cargo tarpaulin --fail-under 80"
constraint add --type MUST --category testing --text "All tests must pass on CI" --verification "cargo test"
```

### Performance Requirements
```bash
constraint add --type SHOULD --category performance --text "Application startup <5 seconds" --verification "./scripts/benchmark-startup.sh"
constraint add --type MUST --category performance --text "Memory usage <100MB under normal load" --verification "./scripts/check-memory-usage.sh"
```

## Automation Integration

### CI/CD Pipeline
```yaml
# .github/workflows/constraints.yml
name: Validate Constraints
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo build --release
      - run: constraint validate --execute
```

### Pre-commit Hook
```bash
#!/bin/sh
# .git/hooks/pre-commit
constraint validate --execute
if [ $? -ne 0 ]; then
    echo "Constraint validation failed. Fix issues before committing."
    exit 1
fi
```

## Troubleshooting

### Common Issues

**"No constraints directory found"**
- Solution: Run any constraint command - directory is created automatically

**"Permission denied"**
- Solution: Check write permissions on `.newton/constraints/` directory

**"Invalid constraint ID format"**
- Solution: IDs must follow `nt-xxxxxx` format where x are base36 characters

**"Verification command failed"**
- Solution: Check that verification scripts exist and have execute permissions

### Getting Help

```bash
# Show all available commands
constraint --help

# Show command-specific help
constraint add --help
constraint validate --help
```

## Next Steps

1. **Explore advanced features**: Custom verification scripts, constraint tagging, priority levels
2. **Integrate with CI/CD**: Automated validation on every change
3. **Team adoption**: Train team members on constraint creation and maintenance
4. **Compliance tracking**: Use validation reports to track project health

## Reference

- **RFC 2119 Keywords**: MUST/SHALL (required), SHOULD (recommended), MAY (optional), FORBIDDEN (prohibited)
- **ID Format**: `nt-xxxxxx` (6 base36 characters, hash-based)
- **Categories**: Lowercase alphanumeric with hyphens (e.g., `security`, `testing`, `performance`)
- **Verification**: Command, script path, or human-readable description