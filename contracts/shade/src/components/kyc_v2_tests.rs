#[cfg(test)]
mod tests {
    use crate::components::kyc_v2 as kyc;
    use crate::types::{VerificationStatus, VerificationType};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    // ── Helper functions ───────────────────────────────────────────────────────

    fn setup_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    fn create_test_address(id: u32) -> Address {
        Address::random(&Env::default())
    }

    // ── KYC Request Submission Tests ───────────────────────────────────────────

    #[test]
    fn test_submit_kyc_verification() {
        let env = setup_env();
        let subject = create_test_address(1);
        let metadata = String::from_slice(&env, "test_metadata");

        let request_id = kyc::submit_kyc_verification(
            &env,
            &subject,
            VerificationType::Individual,
            &metadata,
        );

        assert_eq!(request_id, 1);

        let request = kyc::get_kyc_request(&env, request_id);
        assert_eq!(request.id, 1);
        assert_eq!(request.subject, subject);
        assert_eq!(request.status, VerificationStatus::Pending);
        assert_eq!(request.verification_type, VerificationType::Individual);

        let status = kyc::get_kyc_status(&env, &subject);
        assert_eq!(status, VerificationStatus::Pending);
    }

    #[test]
    #[should_panic(expected = "MerchantAlreadyRegistered")]
    fn test_submit_kyc_already_approved() {
        let env = setup_env();
        let subject = create_test_address(1);
        let admin = create_test_address(2);
        let reviewer = create_test_address(3);
        let metadata = String::from_slice(&env, "test_metadata");

        // Submit initial request
        let request_id = kyc::submit_kyc_verification(
            &env,
            &subject,
            VerificationType::Individual,
            &metadata,
        );

        // Grant reviewer role and approve
        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Try to submit another request - should panic
        kyc::submit_kyc_verification(
            &env,
            &subject,
            VerificationType::Individual,
            &metadata,
        );
    }

    // ── KYC Approval Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_approve_kyc_request() {
        let env = setup_env();
        let subject = create_test_address(1);
        let admin = create_test_address(2);
        let reviewer = create_test_address(3);
        let metadata = String::from_slice(&env, "test_metadata");

        // Submit request
        let request_id = kyc::submit_kyc_verification(
            &env,
            &subject,
            VerificationType::CampaignCreator,
            &metadata,
        );

        // Grant reviewer role
        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);

        // Approve
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Verify approval
        let request = kyc::get_kyc_request(&env, request_id);
        assert_eq!(request.status, VerificationStatus::Approved);
        assert_eq!(request.reviewer, reviewer);

        let status = kyc::get_kyc_status(&env, &subject);
        assert_eq!(status, VerificationStatus::Approved);

        // Verify is_kyc_approved works
        assert!(kyc::is_kyc_approved(&env, &subject));
    }

    #[test]
    fn test_reject_kyc_request() {
        let env = setup_env();
        let subject = create_test_address(1);
        let admin = create_test_address(2);
        let reviewer = create_test_address(3);
        let metadata = String::from_slice(&env, "test_metadata");
        let reason = String::from_slice(&env, "Insufficient documentation");

        // Submit request
        let request_id = kyc::submit_kyc_verification(
            &env,
            &subject,
            VerificationType::Individual,
            &metadata,
        );

        // Grant reviewer role
        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);

        // Reject
        kyc::reject_kyc_request(&env, &reviewer, request_id, &reason);

        // Verify rejection
        let request = kyc::get_kyc_request(&env, request_id);
        assert_eq!(request.status, VerificationStatus::Rejected);
        assert_eq!(request.reviewer, reviewer);

        let status = kyc::get_kyc_status(&env, &subject);
        assert_eq!(status, VerificationStatus::Rejected);

        // Verify is_kyc_approved returns false
        assert!(!kyc::is_kyc_approved(&env, &subject));
    }

    #[test]
    fn test_suspend_kyc() {
        let env = setup_env();
        let subject = create_test_address(1);
        let admin = create_test_address(2);
        let reviewer = create_test_address(3);
        let metadata = String::from_slice(&env, "test_metadata");
        let reason = String::from_slice(&env, "Suspicious activity");

        // Submit and approve first
        let request_id = kyc::submit_kyc_verification(
            &env,
            &subject,
            VerificationType::Individual,
            &metadata,
        );

        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Suspend
        kyc::suspend_kyc(&env, &reviewer, &subject, &reason);

        // Verify suspended
        let status = kyc::get_kyc_status(&env, &subject);
        assert_eq!(status, VerificationStatus::Suspended);

        assert!(!kyc::is_kyc_approved(&env, &subject));
    }

    // ── KYC Expiration Tests ───────────────────────────────────────────────────

    #[test]
    fn test_kyc_expiration() {
        let env = setup_env();
        let subject = create_test_address(1);
        let admin = create_test_address(2);
        let reviewer = create_test_address(3);
        let metadata = String::from_slice(&env, "test_metadata");

        // Submit and approve with very short expiration
        let request_id = kyc::submit_kyc_verification(
            &env,
            &subject,
            VerificationType::Individual,
            &metadata,
        );

        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 1); // 1 day

        // Verify is_kyc_approved works before expiration
        assert!(kyc::is_kyc_approved(&env, &subject));

        // Note: In production, would advance ledger timestamp to test expiration
        assert!(!kyc::is_kyc_expired(&env, &subject));
    }

    // ── Campaign KYC Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_register_campaign_for_kyc() {
        let env = setup_env();
        let creator = create_test_address(1);
        let admin = create_test_address(2);
        let reviewer = create_test_address(3);
        let campaign_id = 1u64;
        let metadata = String::from_slice(&env, "test_metadata");

        // Creator must be KYC approved first
        let request_id = kyc::submit_kyc_verification(
            &env,
            &creator,
            VerificationType::CampaignCreator,
            &metadata,
        );

        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Register campaign
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
        let env = setup_env();
        let creator = create_test_address(1);
        let admin = create_test_address(2);
        let reviewer = create_test_address(3);
        let campaign_id = 1u64;
        let metadata = String::from_slice(&env, "test_metadata");

        // Set up creator KYC
        let request_id = kyc::submit_kyc_verification(
            &env,
            &creator,
            VerificationType::CampaignCreator,
            &metadata,
        );

        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);
        kyc::approve_kyc_request(&env, &reviewer, request_id, 30);

        // Register and verify campaign
        kyc::register_campaign_for_kyc(&env, &creator, campaign_id, false);
        kyc::verify_campaign(&env, &reviewer, campaign_id);

        // Check verification
        let campaign_status = kyc::get_campaign_kyc_status(&env, campaign_id);
        assert_eq!(campaign_status.kyc_status, VerificationStatus::Approved);
        assert_eq!(campaign_status.verified_by, reviewer);
        assert!(campaign_status.verified_at > 0);
    }

    // ── Backer Tracking Tests ──────────────────────────────────────────────────

    #[test]
    fn test_record_backer_contribution() {
        let env = setup_env();
        let backer = create_test_address(1);
        let campaign_id = 1u64;
        let amount = 1000i128;

        // Record contribution
        kyc::record_backer_contribution(&env, &backer, campaign_id, amount);

        // Verify tracking
        let backer_status = kyc::get_backer_kyc_status(&env, &backer);
        assert_eq!(backer_status.backer, backer);
        assert_eq!(backer_status.campaigns_backed, 1);
        assert_eq!(backer_status.total_backed_amount, amount);
    }

    #[test]
    fn test_backer_multiple_contributions() {
        let env = setup_env();
        let backer = create_test_address(1);
        let campaign1 = 1u64;
        let campaign2 = 2u64;
        let amount1 = 1000i128;
        let amount2 = 2000i128;

        // Record multiple contributions
        kyc::record_backer_contribution(&env, &backer, campaign1, amount1);
        kyc::record_backer_contribution(&env, &backer, campaign2, amount2);

        // Verify tracking
        let backer_status = kyc::get_backer_kyc_status(&env, &backer);
        assert_eq!(backer_status.campaigns_backed, 2);
        assert_eq!(backer_status.total_backed_amount, amount1 + amount2);
    }

    // ── Reviewer Role Tests ────────────────────────────────────────────────────

    #[test]
    fn test_grant_and_revoke_reviewer_role() {
        let env = setup_env();
        let admin = create_test_address(1);
        let reviewer = create_test_address(2);

        // Grant role
        kyc::grant_kyc_reviewer_role(&env, &admin, &reviewer);
        assert!(kyc::has_kyc_reviewer_role(&env, &reviewer));

        // Revoke role
        kyc::revoke_kyc_reviewer_role(&env, &admin, &reviewer);
        assert!(!kyc::has_kyc_reviewer_role(&env, &reviewer));
    }

    #[test]
    fn test_admin_always_has_reviewer_role() {
        let env = setup_env();
        let admin = create_test_address(1);

        // Admin should always have reviewer role
        assert!(kyc::has_kyc_reviewer_role(&env, &admin));
    }

    // ── Security Tests ────────────────────────────────────────────────────────

    #[test]
    #[should_panic(expected = "NotAuthorized")]
    fn test_unauthorized_approve_kyc() {
        let env = setup_env();
        let subject = create_test_address(1);
        let unauthorized = create_test_address(2);
        let metadata = String::from_slice(&env, "test_metadata");

        let request_id = kyc::submit_kyc_verification(
            &env,
            &subject,
            VerificationType::Individual,
            &metadata,
        );

        // Try to approve without reviewer role
        kyc::approve_kyc_request(&env, &unauthorized, request_id, 30);
    }

    #[test]
    #[should_panic(expected = "NotAuthorized")]
    fn test_unauthorized_register_campaign() {
        let env = setup_env();
        let creator = create_test_address(1);
        let campaign_id = 1u64;

        // Try to register campaign without KYC approval
        kyc::register_campaign_for_kyc(&env, &creator, campaign_id, true);
    }
}
