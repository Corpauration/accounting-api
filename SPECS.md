# SPECS


# Accounts

## Base account routes

- `GET /accounts`
- `POST /accounts`
- `GET /accounts/{id}`
- `DELETE /accounts/{id}`
- `PATCH /accounts/{id}`
- `PUT /accounts/{id}/metadata`

## Advanced account routes

- `GET /accounts/{id}/operations`
- `GET /accounts/{id}/operations/{operation_id}`
- `POST /accounts/{id}/operations` — append a CREDIT or DEBIT operation; the only way balance changes. For DEBIT, rejected unless `amount <= balance + max_allowed_debt`. Operations are append-only (no update/delete).
- 
- `GET /accounts/{id}/admin-actions`
- `GET /accounts/{id}/admin-actions/{admin_action_id}`
- `POST /accounts/{id}/admin-actions`

## Shortcuts account routes

### Admin actions

- `POST /accounts/{id}/deactivate`
- `POST /accounts/{id}/reactivate`
- `POST /accounts/{id}/max-debt-allowed`

# Payments

## Base payment routes

A payment is created **without a method** — it is a `PENDING` intent for a given `purpose` (PURCHASE | TOP_UP). The method (ACCOUNT_BALANCE | EXTERNAL) is chosen later, at pay-time. Lifecycle: `PENDING` → `PROCESSING` (paid) → `SUCCEEDED` / `FAILED` / `CANCELED`.

- `GET /payments` — filterable (account, status, method, purpose, amount range, date range)
- `POST /payments` — create a method-less `PENDING` payment (`account_id`, `amount`, `purpose`, labels, reason).
- `GET /payments/{id}`
- `PATCH /payments/{id}` — metadata only (labels, reason)
- `GET /payments/{id}/events` — append-only status-transition audit log

## Pay & transition routes

- `POST /payments/{id}/pay` — choose the `method` and pay (PENDING → PROCESSING).
  - `ACCOUNT_BALANCE` resolves synchronously: PURCHASE debits (funds-checked) → SUCCEEDED; insufficient funds reverts to PENDING (method cleared, retryable, 422); TOP_UP is invalid → 422.
  - `EXTERNAL` stays PROCESSING, awaiting an admin confirm/reject.
- `POST /payments/{id}/confirm` — admin: PROCESSING → SUCCEEDED (EXTERNAL+TOP_UP credits the account).
- `POST /payments/{id}/reject` — admin: PROCESSING → FAILED.
- `POST /payments/{id}/cancel` — PENDING or PROCESSING → CANCELED.

Each transition enforces the state machine (illegal → 409).

