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
- `POST /accounts/{id}/operations`
- 
- `GET /accounts/{id}/admin-actions`
- `GET /accounts/{id}/admin-actions/{admin_action_id}`
- `POST /accounts/{id}/admin-actions`

## Shortcuts account routes

### Admin actions

- `PUT /accounts/{id}/deactivate`
- `PUT /accounts/{id}/reactivate`
- `PUT /accounts/{id}/max-debt-allowed`

### Operations

- `PUT /accounts/{id}/balance`

# Payments

## Base payment routes

- `GET /payments`
- `POST /payments`
- `GET /payments/{id}`
- `DELETE /payments/{id}`
- `PATCH /payments/{id}`
- `PUT /payments/{id}`

## Advanced payment routes

- `PUT /payments/{id}/status`

