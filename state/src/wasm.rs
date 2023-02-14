use gear_subscription_io::{SubscriberData, SubscriberDataState, SubscriptionMetadata};
use gmeta::{metawasm, BTreeMap, Metadata};
use gstd::{ActorId, ToString};

#[metawasm]
pub trait Metawasm {
    type State = <SubscriptionMetadata as Metadata>::State;

    fn all_subscriptions(state: Self::State) -> BTreeMap<ActorId, SubscriberDataState> {
        state
            .subscribers
            .iter()
            .map(|(k, v)| {
                let (start_date, start_block) = v.subscription_start;
                let period = v.period;
                let (renewal_date, renewal_block) = v.renewal_date;

                let ret_data = SubscriberDataState {
                    is_active: true,
                    start_date,
                    end_date: start_date + period.to_millis(),
                    start_block,
                    end_block: start_block + period.to_blocks(),
                    period,
                    renewal_date,
                    renewal_block,
                    price: state
                        .payment_methods
                        .get(&v.payment_method)
                        .copied(),
                };

                (*k, ret_data)
            })
            .collect()
    }
}