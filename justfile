set shell := ["bash", "-cu"]

# Set the Coder URL used by the CLI (the demo will use whatever you set here).
# Usage: just set-url https://coder.yourcompany.com
set-url url:
    sed -i -E 's|pub const DEFAULT_CODER_URL: &str = "[^"]*";|pub const DEFAULT_CODER_URL: \&str = "{{url}}";|' src/lib.rs
    @echo "Set DEFAULT_CODER_URL to {{url}} in src/lib.rs"
    @echo "Commit it, then run: just publish v0.1.0"

# Publish a new version to npm: commit + tag + push (CI builds & publishes).
# Usage: just publish v0.1.0
publish version:
    cargo fmt
    cargo clippy --all-targets -- -D warnings
    git add -A
    git commit -m "chore: release {{version}}"
    git tag "{{version}}"
    git push origin main --tags
    @echo "Pushed {{version}} — GitHub Actions will build & publish to npm."
    @echo "Verify at https://www.npmjs.com/package/openflows-cli"
