# OpenFlows CLI

A self-contained **terminal CLI** that onboards a tenant to a deployed
[OpenFlows](https://github.com/The-AgenticFlow/openflows) instance — the
autonomous AI dev team that turns GitHub issues into reviewed, merged pull
requests.

`openflows-cli` registers a project (a *tenant*) with OpenFlows so that the AI
team starts working on a GitHub repository. It authenticates **directly to
Coder** using your own Coder session token, collected securely and
interactively.

> This is a **terminal tool**, not a web UI. A browser is only involved when you
> fetch a Coder session token and paste it back into the terminal.

## Install

```sh
npm install -g openflows-cli
```

or run it directly without installing:

```sh
npx openflows-cli --help
```

## Quick start

Onboard a repository to OpenFlows in one command:

```sh
openflows-cli tenant add my-org/my-repo --name payments --fleet 3
```

The first time you run it, the CLI will:

1. print the Coder token page for your deployment,
2. ask you to create a Coder session token there and paste it back,
3. authenticate to Coder, wait for the GitHub App link,
4. provision the tenant's `openflows-nexus-payments` workspace with a fleet of
   3 FORGE-SENTINEL pairs.

When it finishes, OpenFlows automatically starts working on GitHub issues in
`my-org/my-repo`.

## Usage

### `tenant add <owner/repo> [--name <tenant>] [--fleet N]`

Registers a tenant bound to a GitHub repository.

| Argument | Description |
|---|---|
| `<owner/repo>` | The GitHub repository to bind (required) |
| `--name <tenant>` | Tenant name (defaults to the repo owner) |
| `--fleet N` | Number of FORGE-SENTINEL worker pairs, must be `>= 1` |

```sh
# Default name (repo owner), 2 worker pairs
openflows-cli tenant add acme/payments --fleet 2

# Explicit name and larger fleet
openflows-cli tenant add acme/payments --name payments --fleet 4
```

## Authentication

The CLI authenticates to **Coder** using a Coder **session token**. If you
haven't saved one, it prompts you to create it in the browser and paste it back.
It is stored locally in `~/.config/openflows-cli/config.toml` (owner-only
permissions).

The Coder URL and token are resolved in this order:

1. `--url` / `--token` flags
2. `CODER_URL` / `CODER_SESSION_TOKEN` environment variables
3. saved config file
4. a built-in default (configured by your deployment)

```sh
# Non-interactive (for CI or scripting)
CODER_SESSION_TOKEN=<token> openflows-cli tenant add acme/payments --fleet 2

# Explicit URL and token
openflows-cli --url https://coder.example.com --token <token> tenant add acme/payments --fleet 2
```

## Requirements

- **Node.js** (needed to run the npm wrapper; the actual CLI is a compiled Rust
  binary).
- A running, reachable **Coder** server (the one your OpenFlows deployment uses).
- The **OpenFlows GitHub App** installed on the repository's organization.

## Examples

```sh
# Add the payments team with 3 FORGE-SENTINEL pairs
openflows-cli tenant add acme/payments --name payments --fleet 3

# Add a second tenant without an explicit name
openflows-cli tenant add acme/billing --fleet 1
```

After a tenant is added, create an issue in the bound repository — OpenFlows
assigns it, provisions a workspace, and starts working automatically.

## License

MIT
