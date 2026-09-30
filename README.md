# OpenFlows CLI

A **self-contained terminal CLI** that onboards a tenant to a deployed OpenFlows
instance. It authenticates **directly to Coder** using a Coder session token
(collected interactively) and performs the whole `tenant add` provisioning
itself — it does **not** depend on the main `openflows` binary or the manager API.

> This is a terminal CLI (not a web UI). A browser is only involved when you
> fetch a Coder session token and paste it back into the terminal.

## What it does (for now)

Only one command:

```
openflows-cli tenant add my-org/my-repo --name payments --fleet 3
```

This registers a tenant (bound to a GitHub repo) with `N` FORGE-SENTINEL pairs:
it authenticates to Coder, waits for the GitHub App link, and provisions the
tenant's `openflows-nexus-<tenant>` workspace (which auto-starts the controller).

## Install

From source:

```sh
cargo install --path .
```

Or via npm (once published):

```sh
npx openflows-cli tenant add my-org/my-repo --name payments --fleet 3
```

## Usage

```sh
# Interactive — prompts for the Coder session token if not already saved
openflows-cli tenant add my-org/my-repo --name payments --fleet 3

# Non-interactive — provide the token via env or flag
CODER_SESSION_TOKEN=<token> openflows-cli tenant add my-org/my-repo --fleet 3
openflows-cli --url https://coder.example.com --token <token> tenant add my-org/my-repo --fleet 3
```

The Coder URL and session token are resolved in this order:

1. `--url` / `--token` flags
2. `CODER_URL` / `CODER_SESSION_TOKEN` env vars
3. saved config (`~/.config/openflows-cli/config.toml`, owner-only)
4. the hardcoded default Coder URL (`src/lib.rs` — change it for your deployment)

If no token is available it prompts you to create one in the browser and paste
it back.

## Configuration

- Coder URL: change `DEFAULT_CODER_URL` in `src/lib.rs` for your deployment.
- Hook secret: `CODER_CHAT_HOOK_SECRET` (if unset, one is generated for the run).

## Notes

- `--fleet N` = N FORGE-SENTINEL pairs (N forge + N sentinel workers). Required, >= 1.
- Tenant name defaults to the repo owner.
- This reuses the same Coder REST endpoints as the main OpenFlows `tenant add`,
  implemented directly here (no dependency on the main repo's crates).

## Docs

- [DEPLOYMENT.md](DEPLOYMENT.md) — full guide to deploy to npm and use against your Coder deployment
- [PUBLISHING.md](PUBLISHING.md) — details on publishing to npm (token, versions)
