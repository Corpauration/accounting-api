# accounting-api

An HTTP service for tracking customer **accounts**, the money that moves in and
out of them, and the **payments** that drive those movements.

## Overview

The service manages prepaid-style accounts and the payments against them:

- **Accounts** — open an account, give it a name, tags and labels, and set how
  far it is allowed to go into debt. Accounts can be listed and searched
  (for example: accounts in debt, accounts above or below a given balance,
  by status, owner or tag).
- **Balance & history** — every credit or debit is kept as a permanent entry,
  so an account always has a complete, auditable history of how its balance
  came to be. Spending is automatically refused once it would push the account
  past its allowed debt.
- **Payments** — a payment represents either a **purchase** (spending) or a
  **top-up** (adding funds). A payment is first created as a pending request,
  and is then paid using a chosen method:
  - **from the account balance** — settled immediately;
  - **by an external method** — held for an administrator to validate before it
    completes.

  Payments that can't be settled (e.g. not enough funds) can simply be retried,
  and each payment keeps a full timeline of what happened to it.
- **Administration & audit** — administrators can deactivate or reactivate an
  account and adjust its allowed debt. Every administrative action is recorded
  in a searchable audit log.

The full list of routes is in [`SPECS.md`](./SPECS.md).

## API documentation

The service serves its own OpenAPI documentation:

- **Interactive docs (Scalar UI):** `GET /api/docs/ui`
- **OpenAPI spec (JSON):** `GET /api/docs/openapi.json`

With the default configuration these are at
<http://localhost:3000/api/docs/ui> and
<http://localhost:3000/api/docs/openapi.json>.

## Tech stack

- Rust (edition 2021), **`rustc` ≥ 1.94**.
- [axum](https://github.com/tokio-rs/axum) for HTTP.
- [sqlx](https://github.com/launchbadge/sqlx) with **compile-time-checked
  queries** against PostgreSQL.
- [utoipa](https://github.com/juhaku/utoipa) + Scalar for the OpenAPI docs.

## Prerequisites

- Rust toolchain ≥ 1.94.
- A PostgreSQL database (v12+), reachable both to build and to run.

## Configuration

Configuration is read from environment variables (the application does **not**
load a `.env` file at runtime — export the variables, or use your process
manager). A template is provided in [`.env.example`](./.env.example).

| Variable                | Used at  | Default     | Description                                          |
| ----------------------- | -------- | ----------- | ---------------------------------------------------- |
| `DATABASE_URL`          | build    | —           | PostgreSQL connection string the sqlx macros check queries against at compile time. |
| `POSTGRESQL_ADDON_URI`  | run      | —           | PostgreSQL connection string used by the running service. |
| `HOST`                  | run      | `0.0.0.0`   | Bind address.                                        |
| `PORT`                  | run      | `3000`      | Bind port.                                           |
| `RUST_LOG`              | run      | `info`      | Log filter (e.g. `info,accounting_api=debug`).       |

## Build

Queries are verified at compile time, so building requires `DATABASE_URL` to
point at a reachable PostgreSQL database:

```sh
export DATABASE_URL=postgres://<user>:<password>@<host>:5432/<db>
cargo build
```

## Run

Running requires a reachable PostgreSQL database; pass its connection string via
`POSTGRESQL_ADDON_URI`. Database migrations (in `migrations/`) are applied
automatically on startup.

```sh
export POSTGRESQL_ADDON_URI=postgres://<user>:<password>@<host>:5432/<db>
export HOST=0.0.0.0 PORT=3000 RUST_LOG=info
cargo run
```

The server then listens on `http://$HOST:$PORT`, with the docs at
`/api/docs/ui`.

## Tests

Integration tests use [`#[sqlx::test]`](https://docs.rs/sqlx), which provisions
an isolated database per test, so a reachable PostgreSQL is required:

```sh
export DATABASE_URL=postgres://<user>:<password>@<host>:5432/<db>
cargo test
```
