target_dir := "target"

doc_dir := "doc"
doc_assets_dir := doc_dir + "/assets"
project_name := "gc9a01"

#----------
# Building
#----------

build: check-formatting check-clippy  build-without-fmt-check

build-without-fmt-check: test check-readme generate-docs

# Run cargo test
test:
    cargo test

# Run cargo test with all features enabled
test-all:
    cargo test --all-features

# Check the formatting
check-formatting:
    cargo fmt --all -- --check --color always

# Check clippy
check-clippy:
    cargo clippy --all-targets --all-features --workspace -- -D warnings

#------
# Docs
#------

# Generates the docs
generate-docs:
    cargo clean --doc
    cargo doc --all-features --no-deps

#----------------------
# README.md generation
#----------------------

# Generate README.md for a single crate
generate-readme: _build-readme
    #!/usr/bin/env bash
    set -euo pipefail
    cp "{{target_dir}}/README.md" "README.md"

# Check README.md for a single crate
check-readme: _build-readme
    #!/usr/bin/env bash
    set -euo pipefail
    diff -q "{{target_dir}}/README.md" "README.md" || ( \
        echo -e "\033[1;31mError:\033[0m README.md for {{project_name}} needs to be regenerated."; \
        echo -e "       Run 'just generate-readme' to regenerate.\n"; \
        exit 1 \
    )

# Builds README.md for a single crate
_build-readme:
    #!/usr/bin/env bash
    set -e -o pipefail
    mkdir -p {{target_dir}}
    echo "Building README.md for {{project_name}}"
    cargo readme > {{target_dir}}/README.md
