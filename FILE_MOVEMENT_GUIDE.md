# File Movement Guide

This document provides step-by-step instructions for implementing the project reorganization plan.

## Overview

The reorganization involves:
1. Moving root-level documents to appropriate `docs/` subdirectories
2. Organizing standalone test files
3. Creating proper directory structure
4. Updating references and links

## File Movements Required

### Documents to Move

| Source File | Target Location | Type | Priority |
|-------------|----------------|------|----------|
| `CODE_ANALYSIS_REPORT.md` | `docs/reports/CODE_ANALYSIS_REPORT.md` | Report | High |
| `PRODUCTION_READINESS_SUMMARY.md` | `docs/reports/PRODUCTION_READINESS_SUMMARY.md` | Report | High |
| `MEMORY_SAFETY_IMPLEMENTATION_SUMMARY.md` | `docs/reports/MEMORY_SAFETY_IMPLEMENTATION_SUMMARY.md` | Report | High |
| `MODULARIZATION_STRATEGY.md` | `docs/plans/MODULARIZATION_STRATEGY.md` | Plan | Medium |
| `AGENTS.md` | `docs/project/AGENTS.md` | Project | Medium |
| `CLAUDE.md` | `docs/project/CLAUDE.md` | Project | Medium |
| `TODOS.md` | `docs/project/TODOS.md` | Project | Medium |

### Test Files to Process

| Source File | Target Location | Action | Status |
|-------------|----------------|-------|---------|
| `test_jsonld_compliance.rs` | `tests/standalone/jsonld_compliance.rs` | Move & Update | Compatible |
| `test_tokenization.rs` | `tests/legacy/test_tokenization.rs` | Archive | Incompatible |
| `test_tokenizer.rs` | `tests/legacy/test_tokenizer.rs` | Archive | Incompatible |

## Step-by-Step Instructions

### Phase 1: Backup Preparation

1. **Create Backup**
```bash
# Create a backup of current structure
cp -r . ../owl2-reasoner-backup-$(date +%Y%m%d)
cd ../owl2-reasoner-backup-$(date +%Y%m%d)
git status
```

2. **Document Current State**
```bash
# List all files in root
ls -la *.md > root_files_list.txt
ls -la test_*.rs > root_tests_list.txt

# Create file inventory
find . -name "*.md" -type f > all_md_files.txt
find . -name "test_*.rs" -type f > all_test_files.txt
```

### Phase 2: Directory Structure Creation

1. **Create Documentation Directories**
```bash
# Create reports directory
mkdir -p docs/reports

# Create plans directory  
mkdir -p docs/plans

# Ensure project directory exists
mkdir -p docs/project
```

2. **Create Test Directories**
```bash
# Create test subdirectories
mkdir -p tests/standalone
mkdir -p tests/legacy
```

### Phase 3: Document Movements

1. **Move Analysis Reports**
```bash
# Move to reports directory
mv CODE_ANALYSIS_REPORT.md docs/reports/
mv PRODUCTION_READINESS_SUMMARY.md docs/reports/
mv MEMORY_SAFETY_IMPLEMENTATION_SUMMARY.md docs/reports/
```

2. **Move Planning Documents**
```bash
# Move to plans directory
mv MODULARIZATION_STRATEGY.md docs/plans/
```

3. **Move Project Documents**
```bash
# Move to project directory
mv AGENTS.md docs/project/
mv CLAUDE.md docs/project/
mv TODOS.md docs/project/
```

### Phase 4: Test File Processing

1. **Move Compatible Test**
```bash
# Move JSON-LD compliance test (needs code updates)
mv test_jsonld_compliance.rs tests/standalone/
```

2. **Archive Incompatible Tests**
```bash
# Move legacy tests to archive directory
mv test_tokenization.rs tests/legacy/
mv test_tokenizer.rs tests/legacy/
```

### Phase 5: Code Updates for JSON-LD Test

The `test_jsonld_compliance.rs` file needs updates to work with the current API:

**Required Changes:**
1. Update imports to use current module structure
2. Add memory-safe testing patterns
3. Update to use current error handling
4. Add proper integration with test suite

**Example Updates:**
```rust
// Before (potentially incompatible)
use owl2_reasoner::parser::JsonLdParser;
use owl2_reasoner::OntologyParser;

// After (current API)
use owl2_reasoner::{JsonLdParser, OntologyParser};
use owl2_reasoner::test_helpers::memory_safe_stress_test;
```

### Phase 6: Reference Updates

1. **Update README Files**
```bash
# Update main README.md to reflect new structure
# Update docs/README.md with new organization
# Update project documentation indexes
```

2. **Update Internal Links**
- Check all moved documents for internal links
- Update cross-references to new file locations
- Validate table of contents entries

3. **Update CI/CD Configuration**
```bash
# Update .github/workflows/ if needed
# Update build scripts
# Update documentation generation scripts
```

### Phase 7: Validation

1. **Test Documentation Build**
```bash
# Verify documentation builds correctly
cargo doc --no-deps

# Check for broken links
find docs/ -name "*.md" -exec grep -l "\[.*\](" {} \;
```

2. **Run Test Suite**
```bash
# Run all tests to ensure no breaks
cargo test --lib

# Run standalone tests
cargo run --bin test_jsonld_compliance
```

3. **Validate File Structure**
```bash
# Check final structure
tree docs/ -I 'target'
tree tests/ -I 'target'

# Verify no broken imports
cargo check --all-targets
```

## Post-Movement Tasks

### 1. Update Main README

Update the main `README.md` to reflect new documentation structure:

```markdown
## Documentation

- **[User Guide](docs/user-guide/)** - Getting started and usage examples
- **[API Reference](docs/api/)** - Complete API documentation
- **[Developer Guide](docs/developer/)** - Development and contribution guidelines
- **[Reports](docs/reports/)** - Analysis reports and status summaries
- **[Plans](docs/plans/)** - Strategic planning documents
```

### 2. Update Documentation Index

Update `docs/README.md` with new organization:

```markdown
## Documentation Structure

### Reports
Analysis and status reports:
- [Code Analysis Report](reports/CODE_ANALYSIS_REPORT.md)
- [Production Readiness](reports/PRODUCTION_READINESS_SUMMARY.md)
- [Memory Safety Implementation](reports/MEMORY_SAFETY_IMPLEMENTATION_SUMMARY.md)

### Plans
Strategic planning documents:
- [Modularization Strategy](plans/MODULARIZATION_STRATEGY.md)

### Project Management
Project-related documentation:
- [Agents Configuration](project/AGENTS.md)
- [Development TODOs](project/TODOS.md)
```

### 3. Update Git Configuration

Add appropriate `.gitignore` entries if needed:

```gitignore
# Temporary files during reorganization
*_files_list.txt
reorganization_backup/
```

### 4. Commit Changes

Use clear, descriptive commit messages:

```bash
git add .
git commit -m "Reorganize project documentation and test structure

- Move analysis reports to docs/reports/
- Move planning documents to docs/plans/  
- Move project docs to docs/project/
- Organize test files into standalone/ and legacy/
- Update documentation structure and references"
```

## Rollback Plan

If issues arise during reorganization:

1. **Quick Rollback**
```bash
# Restore from backup
cd ../owl2-reasoner-backup-$(date +%Y%m%d)
cp -r . ../owl2-reasoner/
cd ../owl2-reasoner
```

2. **Partial Rollback**
```bash
# Move specific files back
mv docs/reports/CODE_ANALYSIS_REPORT.md .
mv docs/reports/PRODUCTION_READINESS_SUMMARY.md .
# ... etc
```

## Validation Checklist

### Before Completion
- [ ] All documents moved to correct locations
- [ ] Internal links updated and working
- [ ] Test suite passes completely
- [ ] Documentation builds without errors
- [ ] JSON-LD test updated and working
- [ ] Main README updated with new structure
- [ ] CI/CD processes working correctly

### After Completion
- [ ] Team notified of changes
- [ ] Documentation updated in project wiki
- [ ] Development environment updated
- [ ] Links in external references updated

## Expected Benefits

After reorganization:

1. **Improved Organization**: Clear separation of concerns
2. **Better Navigation**: Intuitive file structure
3. **Enhanced Maintainability**: Single source of truth for content
4. **Professional Appearance**: Clean project root directory
5. **Easier Onboarding**: New developers can find information easily

## Troubleshooting

### Common Issues

#### Broken Links
```bash
# Find broken internal links
grep -r "\[.*](" docs/ | grep -v "http"
```

#### Test Failures
```bash
# Check for missing imports
cargo check --all-targets 2>&1 | grep "unresolved import"

# Run specific failing test
cargo test test_name -- --nocapture
```

#### Documentation Build Issues
```bash
# Check for markdown syntax errors
find docs/ -name "*.md" -exec markdownlint {} \; 2>/dev/null || true

# Manual validation
cargo doc --no-deps --open
```

This reorganization will significantly improve the project's structure and maintainability while preserving all existing functionality.