#![no_std]

use ft_io::{FTAction, FTEvent};
use gear_subscription_io::{Actions, Period, Price, SubscriberData, SubscriptionState};
use gstd::{async_main, exec, msg, prelude::*, ActorId};

// todo control tokens are of erc20 standard
// todo error workflow done by eco-system guys
// todo control errors between async calls
// todo add readme + docs
// todo add tests

static mut PAID_SUBSCRIPTIONS: BTreeSet<ActorId> = BTreeSet::new();
static mut SUPPORTED_TOKENS: BTreeMap<ActorId, Price> = BTreeMap::new();
static mut SUBSCRIBERS: BTreeMap<ActorId, SubscriberData> = BTreeMap::new();

#[no_mangle]
extern "C" fn init() {
    add_token(msg::load().expect("init: wrong payload: expected token id"));
}

#[async_main]
async fn main() {
    match msg::load().expect("handle: wrong payload: expected `Actions`") {
        Actions::RegisterSubscription {
            period,
            payment_method,
            with_renewal,
        } => {
            let subscriber = msg::source();
            let payment_fee = get_price(&payment_method);

            if get_subscriber(&subscriber).is_some() || payment_fee.is_none() {
                panic!("RegisterSubscription: invalid subscription state");
            }

            let payment_fee = payment_fee.expect("checked");

            let _: FTEvent = msg::send_for_reply_as(
                payment_method,
                FTAction::Transfer {
                    from: subscriber,
                    to: exec::program_id(),
                    amount: payment_fee,
                },
                0,
            )
            .unwrap_or_else(|e| { 
                panic!("RegisterSubscription: error sending async message: {e:?}")
            })
            .await
            .unwrap_or_else(|e| {
                panic!("RegisterSubscription: transfer ended up with an error {e:?}")
            });
            add_paid_sub(subscriber);

            if msg::send_delayed(
                exec::program_id(),
                Actions::CheckSubscription { subscriber },
                0,
                Period::minimal_unit().to_blocks(),
            ).is_ok() {
                let start_date = exec::block_timestamp();
                let start_block = exec::block_height();
                add_subscriber(
                    subscriber,
                    SubscriberData { 
                        with_renewal,
                        payment_method,
                        period,
                        subscription_start: (start_date, start_block),
                        renewal_date: (
                            start_date + Period::minimal_unit().to_millis(),
                            start_block + Period::minimal_unit().to_blocks(),
                        )
                    }
                );
            }
        }
        Actions::CheckSubscription { subscriber } => {
            let this_program = exec::program_id();
            if msg::source() != this_program {
                panic!("CheckSubscription: message allowed only for this program");
            }

            let SubscriberData {
                with_renewal,
                payment_method,
                period,
                subscription_start,
                ..
            } = get_subscriber(&subscriber).copied().expect("CheckSubscription: subscriber not found");
            let (start_date, start_block) = subscription_start;

            let current_block = exec::block_height();
            let current_date = exec::block_timestamp();
            let is_not_expired = start_block + period.to_blocks() >= current_block || start_date + period.to_millis() >= current_date;
            let is_renewal_case = !is_not_expired && with_renewal;

            if is_not_expired || is_renewal_case {
                let price = get_price(&payment_method).expect("CheckSubscription: payment method was deleted");
                let _: FTEvent = msg::send_for_reply_as(
                    payment_method,
                    FTAction::Transfer {
                        from: subscriber,
                        to: this_program,
                        amount: price,
                    },
                    0,
                )
                .unwrap_or_else(|e| { 
                    panic!("CheckSubscription: error sending async message: {e:?}")
                })
                .await
                .unwrap_or_else(|e| {
                    panic!("CheckSubscription: transfer ended up with an error {e:?}")
                });

                if msg::send_delayed(
                    this_program,
                    Actions::CheckSubscription { subscriber },
                    0,
                    Period::minimal_unit().to_blocks(),
                ).is_ok() {
                    let subscription_start = if is_renewal_case {
                        (current_date, current_block)
                    } else {
                        (start_date, start_block)
                    };
                    add_subscriber(
                        subscriber,
                        SubscriberData {
                            with_renewal,
                            payment_method,
                            period,
                            subscription_start,
                            renewal_date: (
                                current_date + Period::minimal_unit().to_millis(),
                                current_block + Period::minimal_unit().to_blocks(),
                            ),
                        },
                    );
                }
            } else {
                delete_subscriber(&subscriber)
            }
        }
        Actions::CancelSubscription => {
            let subscriber = msg::source();
            let subscription = get_subscriber(&subscriber);
            if subscription.is_none() {
                panic!("CancelSubscription: subscription not found");
            }

            let updated_subscription = {
                let mut new_data = subscription.copied().expect("checked");
                new_data.with_renewal = false;
                
                new_data
            };

            add_subscriber(subscriber, updated_subscription);
        }
    }
}

#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    msg::reply(metahash, 0).expect("Failed to share metahash");
}

#[no_mangle]
extern "C" fn state() {
    let ret_state = unsafe { SUBSCRIBERS.clone() };
    let ret_state2 = unsafe { SUPPORTED_TOKENS.clone() };
    let _ = msg::reply::<SubscriptionState>((ret_state, ret_state2).into(), 0);
}

fn add_token(token_data: (ActorId, Price)) {
    let (token_addr, price) = token_data;
    unsafe {
        SUPPORTED_TOKENS.insert(token_addr, price);
    }
}

fn get_price(token: &ActorId) -> Option<Price> {
    unsafe { SUPPORTED_TOKENS.get(token).copied() }
}

fn add_subscriber(subscriber: ActorId, data: SubscriberData) {
    unsafe {
        SUBSCRIBERS.insert(subscriber, data);
    }
}

fn add_paid_sub(subscriber: ActorId) {
    unsafe { PAID_SUBSCRIPTIONS.insert(subscriber); }
}

fn get_subscriber(subscriber: &ActorId) -> Option<&SubscriberData> {
    unsafe { SUBSCRIBERS.get(subscriber) }
}

fn delete_subscriber(subscriber: &ActorId) {
    unsafe {
        PAID_SUBSCRIPTIONS.remove(subscriber);
        SUBSCRIBERS.remove(subscriber);
    }
}
