# Publishing OpenFlows CLI to npm

This CLI is distributed on npm using Orhun's "Packaging Rust for npm" approach:
the Rust binary is built for each platform and wrapped in per-platform npm
packages, plus a base `openflows-cli` package that runs the right binary.

Publishing is **triggered by version tags** (not code pushes), and npm will not
let you re-publish the same version. So every release = **bump the version +
tag**.

## Before the first publish (one-time)

1. **npm account** at https://www.npmjs.com/signup — email must be verified
   (required to publish unscoped public packages).
2. **npm token**: npmjs.com → Access Tokens → Generate → **Granular**:
   - Permission: **Read and write (publish and stage)** (our CI runs `npm publish`)
   - Select packages: **All packages** (the packages don't exist yet to select)
   - Organizations: **No access** (unscoped personal packages)
   - Expiration: **90 days** (a short-lived token breaks CI weekly)
3. **Add the token to GitHub**: repo → Settings → Secrets and variables →
   Actions → New repository secret → name `NPM_TOKEN`, paste the token.

## Set the real Coder URL (important)

The CLI points at Coder via `DEFAULT_CODER_URL` in `src/lib.rs` (currently a
placeholder). **Set it to your real deployment URL before the version you'll
use for the demo** — otherwise the demo points at localhost and fails.

```sh
just set-url https://coder.yourcompany.com
# or edit src/lib.rs directly, then commit
```

## Publish a new version

```sh
just publish v0.1.0
```

That runs fmt + clippy, commits, tags, and pushes. The `Continuous Deployment`
GitHub Action then builds all platforms and publishes to npm.

Alternatively, do it by hand:

```sh
git add -A
git commit -m "chore: release v0.1.0"
git tag v0.1.0
git push origin main --tags
```

## Verify

```sh
npx openflows-cli tenant add my-org/my-repo --name payments --fleet 3
```

## Notes

- **Every change you want on npm needs a new, higher version tag.** There is no
  "republish same version."
- The version published **with the real Coder URL** is the one to use for the
  demo. If you published an earlier placeholder version, just bump again.
- Watch the run under the repo's **Actions** tab; it must fully complete
  (including the per-platform packages) before `npx` works on all platforms.
