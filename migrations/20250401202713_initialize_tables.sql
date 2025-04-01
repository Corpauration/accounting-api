-- Add migration script here
CREATE TYPE account_status AS ENUM ('ACTIVE', 'DEACTIVATED', 'DELETED');

CREATE TABLE IF NOT EXISTS accounts
(
    id               uuid primary key,
    owner_id         text unique    not null,
    name             text           not null,
    description      text,
    tags             text[]         not null,
    labels           hstore         not null,
    max_allowed_debt int            not null,
    balance          int            not null,
    status           account_status not null default 'ACTIVE'
);

CREATE TYPE operation_kind AS ENUM ('DEBIT', 'CREDIT');

CREATE TABLE IF NOT EXISTS operations
(
    id         uuid primary key,
    account_id uuid references accounts (id) not null,
    amount     int                           not null,
    timestamp  timestamptz                   not null,
    labels     hstore                        not null,
    kind       operation_kind                not null
);

CREATE TABLE IF NOT EXISTS admin_actions
(
    id         uuid primary key,
    account_id uuid references accounts (id) not null,
    timestamp  timestamptz                   not null,
    labels     hstore                        not null,
    reason     text                          not null,
    kind       text                          not null,
    data       jsonb                         not null default '{}'
);

