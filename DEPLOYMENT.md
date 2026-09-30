# Deployment Guide

This is the step-by-step guide to deploy the OpenFlows CLI to **npm** and use it
against your **real Coder deployment**.

> Summary: set the real Coder URL → tag a version → the GitHub Action builds and
> publishes to npm → operators run it with `npx`.

## 0. Prerequisites (one-time)

- **npm account** at https://www.npmjs.com/signup with a **verified email**
  (required to publish unscoped public packages).
- **npm token** (Access Tokens → Generate → Granular):
  - Permission: **Read and write (publish and stage)**
  - Select packages: **All packages** (packages don't exist yet to select)
  - Organizations: **No access** (unscoped personal packages)
  - Expiration: **90 days**
- **`NPM_TOKEN` secret** in GitHub: repo → Settings → Secrets and variables →
  Actions → New repository secret → name `NPM_TOKEN`, paste the token.
- **The real Coder URL** of your deployment (e.g. `https://coder.yourcompany.com`).

## 1. Set the real Coder URL

The CLI connects to Coder using `DEFAULT_CODER_URL` in `src/lib.rs` (currently a
placeholder). Set it to your real deployment URL:

```sh
just set-url https://coder.yourcompany.com
```

Verify:

```sh
grep DEFAULT_CODER_URL src/lib.rs
# pub const DEFAULT_CODER_URL: &str = "https://coder.yourcompany.com";
```

> **Why this matters:** the version you publish with the real URL is the one you
> use for the demo. Don't ship the placeholder (localhost) URL.

## 2. Publish to npm

Publishing is triggered by **version tags** (not code pushes), and npm will not
let you re-publish the same version. Every release = bump + tag.

```sh
just publish v0.1.0
```

This runs fmt + clippy, commits, tags `v0.1.0`, and pushes. The
**Continuous Deployment** workflow then:
1. builds the Rust binary for all 6 platforms
2. publishes the per-platform packages (`openflows-cli-linux-x64`, …)
3. publishes the base `openflows-cli` package last

Watch it under the repo → **Actions** tab → **Continuous Deployment** — both
jobs must finish green.

## 3. Verify on npm

```sh
npm view openflows-cli version      # should print 0.1.0
```

## 4. Operator usage (the demo)

From any machine:

```sh
npx openflows-cli tenant add my-org/my-repo --name payments --fleet 3
```

The CLI will:
- (if no token saved) prompt: open `<your-coder-url>/settings/tokens`, create a
  token, paste it back
- authenticate to Coder, wait for the GitHub App link (device flow),
- provision the `openflows-nexus-payments` workspace with the requested fleet.

The Coder URL and token are resolved as: flags → env vars → saved config
(`~/.config/openflows-cli/config.toml`) → hardcoded default.

## Publishing subsequent versions

Every change you want on npm needs a new, higher version tag. E.g. after a fix:

```sh
just publish v0.1.1
```

## Troubleshooting

- **`npm publish` fails / 403 / email not verified** — your npm account email
  isn't verified or the token lacks publish rights. Verify the email and that
  the token permission is **Read and write (publish and stage)**.
- **Per-platform package "Not found"** when building the base package — the
  per-platform packages must be published **before** the base one; the CI
  workflow already orders this correctly.
- **`npx openflows-cli` can't find a binary for your OS/arch** — the workflow
  must have completed fully for your platform; check the Actions run.
- **Demo points at localhost** — `DEFAULT_CODER_URL` wasn't updated to the real
  URL before publishing.
