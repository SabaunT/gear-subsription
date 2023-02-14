use gear_subscription_io::SubscriptionMetadata;
use gmeta::{metawasm, Metadata};
use gstd::ActorId;

#[metawasm]
pub trait Metawasm {
    type State = <SubscriptionMetadata as Metadata>::State;

    fn subscribers(state: Self::State) -> Vec<ActorId> {
        state.subscribers.clone()
    }
}