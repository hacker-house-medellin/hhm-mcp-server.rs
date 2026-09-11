# Hacker House Medellin MCP Server

    Read-only MCP diagnostics for applications, residents, stays, rooms, events, and projects. The server is a Rust MCP process over stdio. Stdout is exclusively the JSON-RPC wire; structured diagnostics go to stderr and optional OTLP.

    ## Tools

    - `hhm_fleet_map`
- `hhm_plan`
- `hhm_runtime_readiness`
- `hhm_shared_platform`
- `hhm_lifecycle_state`
- `hhm_safety_boundary`

    Every tool is read-only. Planning accepts a closed workload enum plus bounded numeric fields. The server has no arbitrary URL, command, filesystem, database, GitHub mutation, cluster mutation, or secret-value input.

    ## Product topology

    - `hhm-api` — applications, residents, stays, rooms, events, and projects API
- `hhm-interfaces` — member, stay, room, event, and project contracts
- `hhm-sync` — offline-first residency and project synchronization
- `hhm-clients` — typed client SDKs
- `hhm-infra` — Kubernetes and bounded Cloudflare edge infrastructure

    ## Security boundary

    - MCP never approves an application, assigns a room, or changes a stay.
- Resident and applicant identities are excluded from tools and telemetry.
- Capacity calculations are advisory and require operator authorization elsewhere.

    The shared core is pinned at `c6101656c8227251d1dbd61df54f03a186b42ade`. It provides bounded MCP framing, explicit OTLP/gRPC traces, metrics and logs, JSON stderr diagnostics, redaction, low-cardinality tool metrics, and the formal runtime lifecycle. Each tool also owns an explicit span with `skip_all`; arguments and results are never recorded. Configuration readiness reports environment-variable presence only and performs no authentication or network request.

    This server contains no authenticated HTTP client. If a future tool adds one, it must use fixed or strictly validated HTTP(S) origins, reject credentials/query/fragment/private/metadata targets, disable redirects and ambient proxies, keep credentials in sensitive headers, cap every response, and add adversarial tests before merge.

    ## Shared platform knowledge

    The bounded `shared_platform` tool documents ORE Kubernetes, shared definitions, dpm, Cloudflare/Squarespace, Supabase, and Fiducia without exposing a mutation or credential surface.

    ## Validate

    ```sh
    cargo fmt --all -- --check
    cargo clippy --locked --all-targets --all-features -- -D warnings
    cargo test --locked --all-targets --all-features
    cargo build --locked --release
    cargo audit --deny warnings
    ```
