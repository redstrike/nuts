// @ START-DOC CRATE
//! Nuts is a library that offers a simple publish-subscribe API, featuring decoupled creation of the publisher and the subscriber.
//!
//! ## Quick first example
//! ```rust
//! struct Activity;
//! let activity = nuts::new_activity(Activity);
//! activity.subscribe(
//!     |_activity, n: &usize|
//!     println!("Subscriber received {}", n)
//! );
//! nuts::publish(17usize);
//! // "Subscriber received 17" is printed
//! nuts::publish(289usize);
//! // "Subscriber received 289" is printed
//! ```
//!
//! As you can see in the example above, no explicit channel between publisher and subscriber is necessary.
//! The call to `publish` is a static method that requires no state from the user.
//! The connection between them is implicit because both use `usize` as message type.
//!
//! Nuts enables this simple API by managing all necessary state in thread-local storage.
//! This is particularly useful when targeting the web. However, Nuts can be used on other platforms, too.
//! In fact, Nuts has no dependencies aside from std.
// @ END-DOC CRATE

// code quality
#![forbid(unsafe_code)]
#![deny(clippy::mem_forget)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::mutex_integer)]
#![warn(clippy::needless_pass_by_value)]
// docs
#![warn(missing_docs)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::missing_errors_doc)]

mod nut;

pub use crate::nut::iac::managed_state::{DefaultDomain, DomainEnumeration, DomainState};
use core::any::Any;
pub use nut::activity::*;
pub use nut::iac::filter::*;

use nut::iac::managed_state::*;
use nut::iac::topic::*;

/// A method on an activity. Can be registered dynamically on activities at runtime.
pub struct Method<ACTIVITY>(dyn Fn(&mut ACTIVITY, Option<&mut DomainState>));

/// Consumes a struct and registers it as an Activity.
///
/// `nuts::new_activity(...)` is the simplest method to create a new activity.
/// It takes only a single argument, which can be any struct instance or primitive.
/// This object will be the private data for the activity.
///
/// An `ActivityId` is returned, which is a handle to the newly registered activity.
/// Use it to register callbacks on the activity.
///
/// ### Example:
// @ START-DOC NEW_ACTIVITY
/// ```rust
/// #[derive(Default)]
/// struct MyActivity {
///     round: usize
/// }
/// struct MyMessage {
///     no: usize
/// }
///
/// // Create activity
/// let activity = MyActivity::default();
/// // Activity moves into globally managed state, ID to handle it is returned
/// let activity_id = nuts::new_activity(activity);
///
/// // Add event listener that listens to published `MyMessage` types
/// activity_id.subscribe(
///     |my_activity, msg: &MyMessage| {
///         println!("Round: {}, Message No: {}", my_activity.round, msg.no);
///         my_activity.round += 1;
///     }
/// );
///
/// // prints "Round: 0, Message No: 1"
/// nuts::publish( MyMessage { no: 1 } );
/// // prints "Round: 1, Message No: 2"
/// nuts::publish( MyMessage { no: 2 } );
/// ```
// @ END-DOC NEW_ACTIVITY
pub fn new_activity<A>(activity: A) -> ActivityId<A>
where
    A: Activity,
{
    nut::new_activity(activity, DomainId::default(), LifecycleStatus::Active)
}

/// Consumes a struct that is registered as an Activity that has access to the specified domain.
/// Use the returned `ActivityId` to register callbacks on the activity.
///
// @ START-DOC NEW_ACTIVITY_WITH_DOMAIN
/// ```rust
/// use nuts::{domain_enum, DomainEnumeration};
///
/// #[derive(Default)]
/// struct MyActivity;
/// struct MyMessage;
///
/// #[derive(Clone, Copy)]
/// enum MyDomain {
///     DomainA,
///     DomainB,
/// }
/// domain_enum!(MyDomain);
///
/// // Add data to domain
/// nuts::store_to_domain(&MyDomain::DomainA, 42usize);
///
/// // Register activity
/// let activity_id = nuts::new_domained_activity(MyActivity, &MyDomain::DomainA);
///
/// // Add event listener that listens to published `MyMessage` types and has also access to the domain data
/// activity_id.subscribe_domained(
///     |_my_activity, domain, msg: &MyMessage| {
///         // borrow data from the domain
///         let data = domain.try_get::<usize>();
///         assert_eq!(*data.unwrap(), 42);
///     }
/// );
///
/// // make sure the subscription closure is called
/// nuts::publish( MyMessage );
/// ```
// @ END-DOC NEW_ACTIVITY_WITH_DOMAIN
pub fn new_domained_activity<A, D>(activity: A, domain: &D) -> ActivityId<A>
where
    A: Activity,
    D: DomainEnumeration,
{
    nut::new_activity(activity, DomainId::new(domain), LifecycleStatus::Active)
}

/// Puts the data object to the domain, which can be accessed by all associated activities.
///
/// This function is only valid outside of activities.
/// Inside activities, only access domains through the handlers borrowed access.
/// Typically, this function is only used for initialization of the domain state.
pub fn store_to_domain<D, T>(domain: &D, data: T)
where
    D: DomainEnumeration,
    T: core::any::Any,
{
    nut::write_domain(domain, data).expect("You cannot use `store_to_domain` after initialization.")
}

/// Send the message to all subscribed activities
///
// @ START-DOC PUBLISH
/// Any instance of a struct or primitive can be published, as long as its type is known at compile-time. (The same constraint as for Activities.)
/// Upon calling `nuts::publish`, all active subscriptions for the same type are executed and the published object will be shared with all of them.
///
/// ### Example
/// ```rust
/// struct ChangeUser { user_name: String }
/// pub fn main() {
///     let msg = ChangeUser { user_name: "Donald Duck".to_owned() };
///     nuts::publish(msg);
///     // Subscribers to messages of type `ChangeUser` will be notified
/// }
/// ```
// @ END-DOC PUBLISH
/// ### Advanced: Understanding the Execution Order
// @ START-DOC PUBLISH_ADVANCED
/// When calling `nuts::publish(...)`, the message may not always be published immediately. While executing a subscription handler from previous `publish`, all new messages are queued up until the previous one is completed.
/// ```rust
/// struct MyActivity;
/// let activity = nuts::new_activity(MyActivity);
/// activity.subscribe(
///     |_, msg: &usize| {
///         println!("Start of {}", msg);
///         if *msg < 3 {
///             nuts::publish( msg + 1 );
///         }
///         println!("End of {}", msg);
///     }
/// );
///
/// nuts::publish(0usize);
/// // Output:
/// // Start of 0
/// // End of 0
/// // Start of 1
/// // End of 1
/// // Start of 2
/// // End of 2
/// // Start of 3
/// // End of 3
/// ```
// @ END-DOC PUBLISH_ADVANCED
pub fn publish<A: Any>(a: A) {
    nut::publish_custom(a)
}
