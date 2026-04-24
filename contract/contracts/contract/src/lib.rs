#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, log,
    Address, Env, String, Symbol, Vec, Map,
};

// ─── Data Structures ──────────────────────────────────────────────────────────

/// Tier of the sustainability badge
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BadgeTier {
    Bronze,   // Entry-level sustainable practices
    Silver,   // Intermediate commitment
    Gold,     // High-impact sustainability leader
    Platinum, // Exceptional, verified net-zero or beyond
}

/// The badge issued to a verified sustainable business
#[contracttype]
#[derive(Clone, Debug)]
pub struct Badge {
    pub business_name: String,
    pub owner: Address,
    pub tier: BadgeTier,
    pub issued_at: u64,      // ledger timestamp
    pub expires_at: u64,     // 0 = never expires
    pub category: String,    // e.g. "Energy", "Agriculture", "Manufacturing"
    pub issuer: Address,
    pub revoked: bool,
}

/// Storage keys
#[contracttype]
pub enum DataKey {
    Admin,
    Badge(Address),          // badge keyed by business address
    Verifier(Address),       // approved verifiers
    TotalIssued,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct GreenBadgeContract;

#[contractimpl]
impl GreenBadgeContract {

    // ── Initialisation ────────────────────────────────────────────────────────

    /// Deploy and set the contract administrator.
    /// Must be called once immediately after deployment.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalIssued, &0u32);
        log!(&env, "GreenBadge: initialized with admin {}", admin);
    }

    // ── Admin helpers ─────────────────────────────────────────────────────────

    fn require_admin(env: &Env, caller: &Address) {
        caller.require_auth();
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if *caller != admin {
            panic!("unauthorized: admin only");
        }
    }

    fn require_verifier(env: &Env, caller: &Address) {
        caller.require_auth();
        let is_verifier: bool = env
            .storage()
            .instance()
            .get(&DataKey::Verifier(caller.clone()))
            .unwrap_or(false);
        if !is_verifier {
            // Also allow admin to act as verifier
            let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            if *caller != admin {
                panic!("unauthorized: verifier or admin only");
            }
        }
    }

    // ── Verifier management ───────────────────────────────────────────────────

    /// Add a trusted verifier (admin only)
    pub fn add_verifier(env: Env, admin: Address, verifier: Address) {
        Self::require_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Verifier(verifier.clone()), &true);
        log!(&env, "GreenBadge: verifier added {}", verifier);
    }

    /// Remove a verifier (admin only)
    pub fn remove_verifier(env: Env, admin: Address, verifier: Address) {
        Self::require_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Verifier(verifier.clone()), &false);
        log!(&env, "GreenBadge: verifier removed {}", verifier);
    }

    // ── Badge lifecycle ───────────────────────────────────────────────────────

    /// Issue a badge to a verified sustainable business.
    /// Only callable by an approved verifier or the admin.
    pub fn issue_badge(
        env: Env,
        verifier: Address,
        business: Address,
        business_name: String,
        tier: BadgeTier,
        category: String,
        validity_days: u64, // 0 = never expires
    ) {
        Self::require_verifier(&env, &verifier);

        if env.storage().instance().has(&DataKey::Badge(business.clone())) {
            panic!("badge already exists for this business; use upgrade_badge or revoke first");
        }

        let now = env.ledger().timestamp();
        let expires_at = if validity_days == 0 {
            0
        } else {
            now + validity_days * 86_400 // seconds per day
        };

        let badge = Badge {
            business_name: business_name.clone(),
            owner: business.clone(),
            tier: tier.clone(),
            issued_at: now,
            expires_at,
            category,
            issuer: verifier.clone(),
            revoked: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::Badge(business.clone()), &badge);

        // increment counter
        let total: u32 = env
            .storage()
            .instance()
            .get(&DataKey::TotalIssued)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalIssued, &(total + 1));

        // emit event
        env.events().publish(
            (Symbol::new(&env, "badge_issued"),),
            (business, business_name, tier, now),
        );

        log!(&env, "GreenBadge: badge issued by {}", verifier);
    }

    /// Upgrade an existing badge to a higher tier.
    pub fn upgrade_badge(
        env: Env,
        verifier: Address,
        business: Address,
        new_tier: BadgeTier,
        validity_days: u64,
    ) {
        Self::require_verifier(&env, &verifier);

        let mut badge: Badge = env
            .storage()
            .instance()
            .get(&DataKey::Badge(business.clone()))
            .expect("badge not found");

        if badge.revoked {
            panic!("cannot upgrade a revoked badge");
        }

        let now = env.ledger().timestamp();
        badge.tier = new_tier.clone();
        badge.issued_at = now;
        badge.expires_at = if validity_days == 0 {
            0
        } else {
            now + validity_days * 86_400
        };
        badge.issuer = verifier.clone();

        env.storage()
            .instance()
            .set(&DataKey::Badge(business.clone()), &badge);

        env.events().publish(
            (Symbol::new(&env, "badge_upgraded"),),
            (business, new_tier, now),
        );
    }

    /// Revoke a badge (verifier or admin). The record is kept for audit.
    pub fn revoke_badge(env: Env, caller: Address, business: Address) {
        Self::require_verifier(&env, &caller);

        let mut badge: Badge = env
            .storage()
            .instance()
            .get(&DataKey::Badge(business.clone()))
            .expect("badge not found");

        badge.revoked = true;

        env.storage()
            .instance()
            .set(&DataKey::Badge(business.clone()), &badge);

        env.events().publish(
            (Symbol::new(&env, "badge_revoked"),),
            (business, env.ledger().timestamp()),
        );
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Fetch the full badge record for a business.
    pub fn get_badge(env: Env, business: Address) -> Option<Badge> {
        env.storage()
            .instance()
            .get(&DataKey::Badge(business))
    }

    /// Returns true if the business holds a valid (non-revoked, non-expired) badge.
    pub fn is_verified(env: Env, business: Address) -> bool {
        let badge: Option<Badge> = env
            .storage()
            .instance()
            .get(&DataKey::Badge(business));

        match badge {
            None => false,
            Some(b) => {
                if b.revoked {
                    return false;
                }
                if b.expires_at == 0 {
                    return true;
                }
                env.ledger().timestamp() < b.expires_at
            }
        }
    }

    /// Total badges ever issued.
    pub fn total_issued(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::TotalIssued)
            .unwrap_or(0)
    }

    /// Return the current admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized")
    }

    // ── Admin transfer ────────────────────────────────────────────────────────

    /// Transfer admin rights to a new address.
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        Self::require_admin(&env, &current_admin);
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        log!(&env, "GreenBadge: admin transferred to {}", new_admin);
    }
}