# Newton Constraints CLI

A CLI tool for managing project constraints with RFC 2119 compliance. Define and validate project requirements using formal constraint terminology (MUST, SHALL, SHOULD, MAY, FORBIDDEN).

## Installation

```bash
# Clone and build
git clone <repository-url>
cd constraints
cargo build --release

# Optional: Install globally
cargo install --path .
```

## Quick Start

```bash
# Add your first constraint
constraint add --type MUST --category security --text "All passwords must be hashed using bcrypt" --author "your-name"

# View all constraints
constraint list

# Expected output:
# nt-a1b2c3: MUST [security] All passwords must be hashed using bcrypt
#   Author: your-name | Created: 2026-01-19 10:30:00 UTC
```

## Basic Usage

```bash
# Add constraints for different categories
constraint add --type MUST --category testing --text "All public functions must have unit tests" --author "maintainer"
constraint add --type SHOULD --category performance --text "API responses should complete within 200ms" --author "maintainer"

# List and search
constraint list
constraint list --category security
constraint search "password"

# Update constraints
constraint patch nt-a1b2c3 --text "Updated security requirement"

# Validate compliance
constraint validate --execute
constraint validate --category security --execute
```

## Documentation

For detailed documentation, see [docs/quickstart.md](docs/quickstart.md).

## License

MIT OR Apache-2.0

## Repository

https://github.com/your-org/constraint