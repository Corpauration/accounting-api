use crate::models::operation::OperationKind;
use crate::models::payment::{PaymentMethod, PaymentPurpose, PaymentStatus};

/// A payment method/purpose pairing is valid for every combination except
/// `ACCOUNT_BALANCE` + `TOP_UP` (you cannot top up a balance from itself).
pub fn valid_combo(method: PaymentMethod, purpose: PaymentPurpose) -> bool {
    !matches!(
        (method, purpose),
        (PaymentMethod::AccountBalance, PaymentPurpose::TopUp)
    )
}

/// Transitions allowed via the generic confirm/reject/cancel endpoints. A paid
/// (PROCESSING) payment may reach a terminal state; an unpaid (PENDING) payment
/// may only be canceled. (PENDING→PROCESSING and the PROCESSING→PENDING revert
/// are driven directly by `pay`, not through this guard.)
pub fn can_transition(from: PaymentStatus, to: PaymentStatus) -> bool {
    matches!(
        (from, to),
        (PaymentStatus::Processing, PaymentStatus::Succeeded)
            | (PaymentStatus::Processing, PaymentStatus::Failed)
            | (PaymentStatus::Processing, PaymentStatus::Canceled)
            | (PaymentStatus::Pending, PaymentStatus::Canceled)
    )
}

/// The ledger move applied when a payment reaches `SUCCEEDED`:
/// `ACCOUNT_BALANCE` + `PURCHASE` debits the account, `EXTERNAL` + `TOP_UP`
/// credits it, and every other valid combination has no ledger effect.
pub fn ledger_effect(method: PaymentMethod, purpose: PaymentPurpose) -> Option<OperationKind> {
    match (method, purpose) {
        (PaymentMethod::AccountBalance, PaymentPurpose::Purchase) => Some(OperationKind::Debit),
        (PaymentMethod::External, PaymentPurpose::TopUp) => Some(OperationKind::Credit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_combo_matrix() {
        assert!(valid_combo(
            PaymentMethod::AccountBalance,
            PaymentPurpose::Purchase
        ));
        assert!(valid_combo(
            PaymentMethod::External,
            PaymentPurpose::Purchase
        ));
        assert!(valid_combo(PaymentMethod::External, PaymentPurpose::TopUp));
        // The only invalid combination.
        assert!(!valid_combo(
            PaymentMethod::AccountBalance,
            PaymentPurpose::TopUp
        ));
    }

    #[test]
    fn ledger_effect_matrix() {
        assert_eq!(
            ledger_effect(PaymentMethod::AccountBalance, PaymentPurpose::Purchase),
            Some(OperationKind::Debit)
        );
        assert_eq!(
            ledger_effect(PaymentMethod::External, PaymentPurpose::TopUp),
            Some(OperationKind::Credit)
        );
        assert_eq!(
            ledger_effect(PaymentMethod::External, PaymentPurpose::Purchase),
            None
        );
        // Invalid combination has no ledger effect either.
        assert_eq!(
            ledger_effect(PaymentMethod::AccountBalance, PaymentPurpose::TopUp),
            None
        );
    }

    #[test]
    fn confirm_reject_cancel_transitions() {
        // PROCESSING (paid) → terminal, and PENDING → CANCELED.
        assert!(can_transition(
            PaymentStatus::Processing,
            PaymentStatus::Succeeded
        ));
        assert!(can_transition(
            PaymentStatus::Processing,
            PaymentStatus::Failed
        ));
        assert!(can_transition(
            PaymentStatus::Processing,
            PaymentStatus::Canceled
        ));
        assert!(can_transition(
            PaymentStatus::Pending,
            PaymentStatus::Canceled
        ));
    }

    #[test]
    fn pending_cannot_be_finalized_via_guard() {
        // A PENDING payment must be paid first; it can't be confirmed/rejected,
        // and pay()/revert transitions are not exposed through this guard.
        assert!(!can_transition(
            PaymentStatus::Pending,
            PaymentStatus::Succeeded
        ));
        assert!(!can_transition(
            PaymentStatus::Pending,
            PaymentStatus::Failed
        ));
        assert!(!can_transition(
            PaymentStatus::Pending,
            PaymentStatus::Processing
        ));
        assert!(!can_transition(
            PaymentStatus::Processing,
            PaymentStatus::Pending
        ));
    }

    #[test]
    fn no_transition_from_terminal() {
        let terminals = [
            PaymentStatus::Succeeded,
            PaymentStatus::Failed,
            PaymentStatus::Canceled,
        ];
        let targets = [
            PaymentStatus::Pending,
            PaymentStatus::Processing,
            PaymentStatus::Succeeded,
            PaymentStatus::Failed,
            PaymentStatus::Canceled,
        ];
        for from in terminals {
            for to in targets {
                assert!(!can_transition(from, to), "{from} -> {to} must be illegal");
            }
        }
    }
}
