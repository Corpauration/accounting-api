-- Payment method is now chosen at pay-time, not at creation: a payment is
-- created as a method-less PENDING intent, then paid (method selected), moving
-- through PROCESSING before reaching a terminal status.
ALTER TYPE payment_status ADD VALUE IF NOT EXISTS 'PROCESSING';
ALTER TABLE payments ALTER COLUMN method DROP NOT NULL;
