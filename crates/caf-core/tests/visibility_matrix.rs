use caf_core::visibility::{Visibility, can_modify, sql_scope};

#[test]
fn test_visibility_matrix() {
    println!("Testing visibility matrix across super_admin, admin, member, and viewer");

    // 1. Enum and SQL scope test
    assert_eq!(Visibility::Shared.as_str(), "shared");
    assert_eq!(Visibility::PrivateSummary.as_str(), "private_summary");
    assert_eq!(Visibility::Private.as_str(), "private");

    // Super role bypass scope
    assert_eq!(sql_scope(true), "deleted_at IS NULL");
    // Standard role scoped query
    assert_eq!(
        sql_scope(false),
        "deleted_at IS NULL AND (owner_profile_id = ?1 OR visibility = 'shared')"
    );

    // 2. Ownership modification rule
    assert!(can_modify("profile_A", "profile_A", false));
    assert!(!can_modify("profile_A", "profile_B", false));
    assert!(can_modify("profile_A", "profile_B", true)); // With permission

    println!("Isolation matrix verified successfully.");
}
