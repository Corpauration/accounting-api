CREATE TYPE payment_method AS ENUM ('ACCOUNT_BALANCE', 'EXTERNAL');
CREATE TYPE payment_purpose AS ENUM ('PURCHASE', 'TOP_UP');
CREATE TYPE payment_status AS ENUM ('PENDING', 'SUCCEEDED', 'FAILED', 'CANCELED');

CREATE TABLE IF NOT EXISTS payments
(
    id           uuid primary key,
    account_id   uuid references accounts (id)   not null,
    amount       int                             not null,
    method       payment_method                  not null,
    purpose      payment_purpose                 not null,
    status       payment_status                  not null,
    operation_id uuid references operations (id),
    labels       jsonb                           not null default '{}'::jsonb,
    reason       text,
    created_at   timestamptz                     not null,
    updated_at   timestamptz                     not null
);

CREATE INDEX idx_payments_account_id ON payments (account_id);
CREATE INDEX idx_payments_status ON payments (status);
CREATE INDEX idx_payments_method ON payments (method);
CREATE INDEX idx_payments_created_at ON payments (created_at);

-- Append-only audit log of every payment status transition.
CREATE TABLE IF NOT EXISTS payment_events
(
    id          uuid primary key,
    payment_id  uuid references payments (id) not null,
    from_status payment_status,
    to_status   payment_status                not null,
    timestamp   timestamptz                   not null,
    reason      text,
    actor       text
);

CREATE INDEX idx_payment_events_payment_id ON payment_events (payment_id);
