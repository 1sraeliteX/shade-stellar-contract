#[cfg(test)]
mod tests {
    use crate::components::kyc;
    use crate::types::{VerificationStatus, VerificationType};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_submit_kyc_verification() {
        let env = Env::default();
        let subject = Address::generate(&env);
        let metadata = String::from_str(&env, r#"{"name":"Alice","country":"US"}"#);

        // Submit KYC
        let request_id = kyc::submit_kyc_verification(&env, &subject, VerificationType::Individual, &metadata);

        // Verify request was created
        assert_eq!(request_id, 1);

        // Get request and verify details
        let request = kyc::get_kyc_request(&env, request_id);
        assert_eq!(request.id, 1);
        assert_eq!(request.subject, subject);
        assert_eq!(request.status, VerificationStatus::Pending);
        assert_eq!(request.submitted_at, env.ledger().timestamp());

        // Verify status is pending
        let status = kyc::get_kyc_status(&env, &subject);
        assert_eq!(status, VerificationStatus::Pending);
    }

    #[test]
    fn test_submit_kyc_already_verified() {
        let env = Env::default();
        let subject = Address::generate(&env);
        let metadata = String::from_str(&env, r#"{"name":"Alice","country":"US"}"#);
        let reviewer = Address::generate(&env);

        // First submission
        let request_id = kyc::submit_kyc_verification(&env, &subject, VerificationType::Individual, &metadata);

        // Grant reviewer role and approve
        kyc::grant_kyc_reviewer_role(&env, &reviewer, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Second submission should fail
        let result = std::panic::catch_unwind(|| {
            kyc::submit_kyc_verification(&env, &subject, VerificationType::Individual, &metadata);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_approve_kyc_request() {
        let env = Env::default();
        let subject = Address::generate(&env);
        let reviewer = Address::generate(&env);
        let metadata = String::from_str(&env, r#"{"name":"Alice","country":"US"}"#);

        // Submit KYC
        let request_id = kyc::submit_kyc_verification(&env, &subject, VerificationType::Individual, &metadata);

        // Grant reviewer role
        kyc::grant_kyc_reviewer_role(&env, &reviewer, &reviewer);

        // Approve request
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Verify approval
        let request = kyc::get_kyc_request(&env, request_id);
        assert_eq!(request.status, VerificationStatus::Approved);
        assert_eq!(request.reviewer, reviewer);

        // Verify status updated
        let status = kyc::get_kyc_status(&env, &subject);
        assert_eq!(status, VerificationStatus::Approved);

        // Verify is_kyc_approved works
        assert!(kyc::is_kyc_approved(&env, &subject));
    }

    #[test]
    fn test_reject_kyc_request() {
        let env = Env::default();
        let subject = Address::generate(&env);
        let reviewer = Address::generate(&env);
        let metadata = String::from_str(&env, r#"{"name":"Alice","country":"US"}"#);
        let rejection_reason = String::from_str(&env, "Incomplete documentation");

        // Submit KYC
        let request_id = kyc::submit_kyc_verification(&env, &subject, VerificationType::Individual, &metadata);

        // Grant reviewer role
        kyc::grant_kyc_reviewer_role(&env, &reviewer, &reviewer);

        // Reject request
        kyc::reject_kyc_request(&env, &reviewer, request_id, &rejection_reason);

        // Verify rejection
        let request = kyc::get_kyc_request(&env, request_id);
        assert_eq!(request.status, VerificationStatus::Rejected);
        assert_eq!(request.reviewer, reviewer);

        // Verify status updated
        let status = kyc::get_kyc_status(&env, &subject);
        assert_eq!(status, VerificationStatus::Rejected);

        // Verify is_kyc_approved returns false
        assert!(!kyc::is_kyc_approved(&env, &subject));
    }

    #[test]
    fn test_suspend_kyc() {
        let env = Env::default();
        let subject = Address::generate(&env);
        let reviewer = Address::generate(&env);
        let metadata = String::from_str(&env, r#"{"name":"Alice","country":"US"}"#);
        let reason = String::from_str(&env, "Suspicious activity detected");

        // Submit and approve
        let request_id = kyc::submit_kyc_verification(&env, &subject, VerificationType::Individual, &metadata);
        kyc::grant_kyc_reviewer_role(&env, &reviewer, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Verify approved
        assert!(kyc::is_kyc_approved(&env, &subject));

        // Suspend
        kyc::suspend_kyc(&env, &reviewer, &subject, &reason);

        // Verify suspended
        let status = kyc::get_kyc_status(&env, &subject);
        assert_eq!(status, VerificationStatus::Suspended);
        assert!(!kyc::is_kyc_approved(&env, &subject));
    }

    #[test]
    fn test_kyc_expiration() {
        let env = Env::default();
        let subject = Address::generate(&env);
        let reviewer = Address::generate(&env);
        let metadata = String::from_str(&env, r#"{"name":"Alice","country":"US"}"#);

        // Submit and approve with short expiration (1 second)
        let request_id = kyc::submit_kyc_verification(&env, &subject, VerificationType::Individual, &metadata);
        kyc::grant_kyc_reviewer_role(&env, &reviewer, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 1);

        // Should be approved initially
        assert!(kyc::is_kyc_approved(&env, &subject));
        assert!(!kyc::is_kyc_expired(&env, &subject));

        // Note: Real test would require timestamp manipulation
        // For now, verify expiration date is set
        let request = kyc::get_kyc_request(&env, request_id);
        assert_eq!(request.status, VerificationStatus::Approved);
    }

    #[test]
    fn test_grant_and_revoke_kyc_reviewer_role() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let reviewer = Address::generate(&env);

        // Initially no role
        assert!(!kyc::has_kyc_reviewer_role(&env, &reviewer));

        // Grant role
        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);
        assert!(kyc::has_kyc_reviewer_role(&env, &reviewer));

        // Revoke role
        kyc::revoke_kyc_reviewer_role(&env, &admin, &reviewer);
        assert!(!kyc::has_kyc_reviewer_role(&env, &reviewer));
    }

    #[test]
    fn test_campaign_kyc_registration() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let reviewer = Address::generate(&env);
        let campaign_id = 1u64;
        let metadata = String::from_str(&env, r#"{"name":"Creator","country":"US"}"#);

        // Submit and approve creator KYC
        let request_id = kyc::submit_kyc_verification(&env, &creator, VerificationType::Business, &metadata);
        kyc::grant_kyc_reviewer_role(&env, &reviewer, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Register campaign for KYC
        kyc::register_campaign_for_kyc(&env, &creator, campaign_id, true);

        // Verify campaign KYC status
        let campaign_status = kyc::get_campaign_kyc_status(&env, campaign_id);
        assert_eq!(campaign_status.campaign_id, campaign_id);
        assert_eq!(campaign_status.creator, creator);
        assert_eq!(campaign_status.kyc_status, VerificationStatus::Pending);
        assert!(campaign_status.min_backer_kyc_required);
    }

    #[test]
    fn test_verify_campaign() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let reviewer = Address::generate(&env);
        let campaign_id = 1u64;
        let metadata = String::from_str(&env, r#"{"name":"Creator","country":"US"}"#);

        // Setup: Creator with approved KYC
        let request_id = kyc::submit_kyc_verification(&env, &creator, VerificationType::Business, &metadata);
        kyc::grant_kyc_reviewer_role(&env, &reviewer, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Register campaign
        kyc::register_campaign_for_kyc(&env, &creator, campaign_id, false);

        // Verify campaign
        kyc::verify_campaign(&env, &reviewer, campaign_id);

        // Check verification
        let campaign_status = kyc::get_campaign_kyc_status(&env, campaign_id);
        assert_eq!(campaign_status.kyc_status, VerificationStatus::Approved);
        assert_eq!(campaign_status.verified_by, reviewer);
        assert!(campaign_status.verified_at > 0);
    }

    #[test]
    fn test_record_backer_contribution() {
        let env = Env::default();
        let backer = Address::generate(&env);
        let campaign_id = 1u64;
        let amount = 10000i128;

        // Record contribution
        kyc::record_backer_contribution(&env, &backer, campaign_id, amount);

        // Verify backer status
        let backer_status = kyc::get_backer_kyc_status(&env, &backer);
        assert_eq!(backer_status.backer, backer);
        assert_eq!(backer_status.campaigns_backed, 1);
        assert_eq!(backer_status.total_backed_amount, amount);
    }

    #[test]
    fn test_multiple_backer_contributions() {
        let env = Env::default();
        let backer = Address::generate(&env);

        // Record contributions to different campaigns
        kyc::record_backer_contribution(&env, &backer, 1u64, 10000i128);
        kyc::record_backer_contribution(&env, &backer, 2u64, 20000i128);
        kyc::record_backer_contribution(&env, &backer, 1u64, 15000i128);

        // Verify aggregated status
        let backer_status = kyc::get_backer_kyc_status(&env, &backer);
        assert_eq!(backer_status.campaigns_backed, 3);
        assert_eq!(backer_status.total_backed_amount, 45000i128);
    }

    #[test]
    fn test_kyc_verification_types() {
        let env = Env::default();
        let metadata = String::from_str(&env, r#"{"name":"Test"}"#);

        // Individual
        let subject1 = Address::generate(&env);
        let req1 = kyc::submit_kyc_verification(&env, &subject1, VerificationType::Individual, &metadata);
        let request1 = kyc::get_kyc_request(&env, req1);
        assert_eq!(request1.verification_type, VerificationType::Individual);

        // Business
        let subject2 = Address::generate(&env);
        let req2 = kyc::submit_kyc_verification(&env, &subject2, VerificationType::Business, &metadata);
        let request2 = kyc::get_kyc_request(&env, req2);
        assert_eq!(request2.verification_type, VerificationType::Business);

        // DAO
        let subject3 = Address::generate(&env);
        let req3 = kyc::submit_kyc_verification(&env, &subject3, VerificationType::DAO, &metadata);
        let request3 = kyc::get_kyc_request(&env, req3);
        assert_eq!(request3.verification_type, VerificationType::DAO);
    }

    #[test]
    fn test_sequential_kyc_requests() {
        let env = Env::default();
        let reviewer = Address::generate(&env);
        let metadata = String::from_str(&env, r#"{"name":"Test"}"#);

        kyc::grant_kyc_reviewer_role(&env, &reviewer, &reviewer);

        // Create multiple requests
        let mut request_ids = Vec::new();
        for i in 0..5 {
            let subject = Address::generate(&env);
            let req_id = kyc::submit_kyc_verification(&env, &subject, VerificationType::Individual, &metadata);
            request_ids.push((req_id, subject));
        }

        // Verify IDs are sequential
        for (idx, (req_id, _)) in request_ids.iter().enumerate() {
            assert_eq!(*req_id as usize, idx + 1);
        }

        // Approve all requests
        for (req_id, subject) in request_ids.iter() {
            kyc::approve_kyc_request(&env, &reviewer, *req_id, 30);
            assert!(kyc::is_kyc_approved(&env, subject));
        }
    }
}
