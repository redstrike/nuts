use crate::*;

/// Defines under which circumstances a subscribing activity should be called.
/// At the moment, the only filter option is to check the activity lifecycle state.
/// The default filter will ignore events when the activity is inactive.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SubscriptionFilter {
    /// Only call the subscribed closure when the activity is active.
    pub active_only: bool,
}

impl Default for SubscriptionFilter {
    fn default() -> Self {
        Self { active_only: true }
    }
}

impl SubscriptionFilter {
    /// Create a new subscription filter that will ensure the activity always receives a message, even when inactive.
    pub fn no_filter() -> Self {
        Self { active_only: false }
    }
}

impl ActivityContainer {
    /// Returns true if the call should go through (false if it should be filtered out)
    pub(crate) fn filter<A: Activity>(
        &self,
        id: ActivityId<A>,
        filter: &SubscriptionFilter,
    ) -> bool {
        !filter.active_only || self.status(id.id).is_active()
    }
}
