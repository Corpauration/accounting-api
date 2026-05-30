-- Core accounting tables. Labels use jsonb (mapped to HashMap<String,String> in Rust).
CREATE TYPE account_status AS ENUM ('ACTIVE', 'DEACTIVATED', 'DELETED');

CREATE TABLE IF NOT EXISTS accounts
(
    id               uuid primary key,
    owner_id         text unique    not null,
    name             text           not null,
    description      text,
    tags             text[]         not null,
    labels           jsonb          not null default '{}'::jsonb,
    max_allowed_debt int            not null,
    balance          int            not null,
    status           account_status not null default 'ACTIVE',
    created_at       timestamptz    not null default now(),
    updated_at       timestamptz    not null default now()
);

CREATE INDEX idx_accounts_balance ON accounts (balance);
CREATE INDEX idx_accounts_status ON accounts (status);

CREATE TYPE operation_kind AS ENUM ('DEBIT', 'CREDIT');

-- Append-only ledger. balance on accounts must always equal sum(CREDIT) - sum(DEBIT) here.
CREATE TABLE IF NOT EXISTS operations
(
    id         uuid primary key,
    account_id uuid references accounts (id) not null,
    amount     int                           not null,
    timestamp  timestamptz                   not null,
    labels     jsonb                         not null default '{}'::jsonb,
    kind       operation_kind                not null
);

CREATE INDEX idx_operations_account_id ON operations (account_id);

CREATE TABLE IF NOT EXISTS admin_actions
(
    id         uuid primary key,
    account_id uuid references accounts (id) not null,
    timestamp  timestamptz                   not null,
    labels     jsonb                         not null default '{}'::jsonb,
    reason     text                          not null,
    kind       text                          not null,
    data       jsonb                         not null default '{}'::jsonb,
    actor      text
);

CREATE INDEX idx_admin_actions_account_id ON admin_actions (account_id);
