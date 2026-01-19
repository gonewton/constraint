#!/bin/bash

# Git Hooks Installation Script
# Install git hooks from .githooks/ to .git/hooks/

set -e

echo "🔗 Installing Git Hooks"
echo "======================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Run this script from the project root"
    exit 1
fi

# Check if .githooks directory exists
if [ ! -d ".githooks" ]; then
    echo "❌ Error: .githooks directory not found"
    exit 1
fi

echo "✅ Repository structure verified"

# Create .git/hooks directory if it doesn't exist
if [ ! -d ".git/hooks" ]; then
    echo ""
    echo "📁 Creating .git/hooks directory..."
    mkdir -p .git/hooks
    echo "✅ .git/hooks directory created"
fi

# Install hooks
hooks_installed=0

echo ""
echo "📋 Installing hooks..."

for hook_file in .githooks/*; do
    if [ -f "$hook_file" ]; then
        hook_name=$(basename "$hook_file")
        echo "  Installing $hook_name..."

        # Copy hook to .git/hooks/
        cp "$hook_file" ".git/hooks/$hook_name"

        # Make it executable
        chmod +x ".git/hooks/$hook_name"

        echo "  ✅ $hook_name installed"
        ((hooks_installed++))
    fi
done

echo ""
echo "🎉 Git hooks installation completed!"
echo ""
echo "Summary:"
echo "  ✅ Hooks installed: $hooks_installed"
echo ""
echo "Available hooks:"
if [ -f ".git/hooks/pre-commit" ]; then
    echo "  • pre-commit: Runs formatting, linting, and unit tests before commits"
fi
if [ -f ".git/hooks/pre-push" ]; then
    echo "  • pre-push: Runs full test suite and documentation build before pushes"
fi
if [ -f ".git/hooks/commit-msg" ]; then
    echo "  • commit-msg: Validates conventional commit message format"
fi
echo ""

exit 0