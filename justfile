# just manual: https://github.com/casey/just/#readme

_default:
  @just --list

# Format source code
fmt:
  cargo fmt --all

# Upgrade (and update) dependencies
upgrade:
  cargo upgrade --incompatible
  cargo update

# Run tests
test:
  cargo test --workspace

# Fix lint warnings
fix:
  cargo fix --workspace --all-targets
  cargo clippy --workspace --all-targets --fix

# Install mdbook
install-mdbook:
  cargo install mdbook mdbook-plantuml mdbook-graphviz mdbook-linkcheck

# Serve playground
serve-playground:
  trunk serve

# Serve docs
serve-docs: install-mdbook
  cd doc/ && mdbook serve

frontend-install-npm-packages:
  npm install

# Build CSS file
css: frontend-install-npm-packages
    tailwindcss -i src/style.css -o target/style.css