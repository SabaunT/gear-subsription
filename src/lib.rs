#![no_std]

use ft_io::{FTAction, FTEvent};
use gear_subscription_io::{Actions, Price, SubscriberData, SubscriptionState};
use gstd::{async_main, exec, msg, prelude::*, ActorId};

// todo control tokens are of erc20 standard
// todo error workflow done by eco-system guys
// todo control errors between async calls
// todo add readme + docs
// todo add tests
// todo UpdateSubscription execution is the same as end date

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
            let price = get_price(&payment_method);

            if get_subscriber(&subscriber).is_some() || price.is_none() {
                panic!("RegisterSubscription: invalid subscription state");
            }

            let payment_fee = price.expect("checked") * period.to_units();

            let _: FTEvent = msg::send_for_reply_as(
                payment_method,
                FTAction::Transfer {
                    from: subscriber,
                    to: exec::program_id(),
                    amount: payment_fee,
                },
                0,
            )
            .unwrap_or_else(|e| panic!("RegisterSubscription: error sending async message: {e:?}"))
            .await
            .unwrap_or_else(|e| {
                panic!("RegisterSubscription: transfer ended up with an error {e:?}")
            });

            if msg::send_delayed(
                exec::program_id(),
                Actions::UpdateSubscription { subscriber },
                0,
                period.to_blocks(),
            )
            .is_ok()
            {
                let start_date = exec::block_timestamp();
                let start_block = exec::block_height();
                let renewal_date = if with_renewal {
                    Some((
                        start_date + period.to_millis(),
                        start_block + period.to_blocks(),
                    ))
                } else {
                    None
                };
                add_subscriber(
                    subscriber,
                    SubscriberData {
                        payment_method,
                        period,
                        renewal_date,
                        subscription_start: Some((start_date, start_block)),
                    },
                );
            } else {
                add_subscriber(
                    subscriber,
                    SubscriberData {
                        payment_method,
                        period,
                        renewal_date: None,
                        subscription_start: None,
                    },
                );
            }
        }
        Actions::UpdateSubscription { subscriber } => {
            let this_program = exec::program_id();
            if msg::source() != this_program {
                panic!("UpdateSubscription: message allowed only for this program");
            }

            let SubscriberData {
                payment_method,
                period,
                renewal_date,
                ..
            } = get_subscriber(&subscriber)
                .copied()
                .expect("UpdateSubscription: subscriber not found");

            let current_block = exec::block_height();
            let current_date = exec::block_timestamp();

            if renewal_date.is_some() {
                let price = get_price(&payment_method)
                    .expect("UpdateSubscription: payment method was deleted");
                let _: FTEvent = msg::send_for_reply_as(
                    payment_method,
                    FTAction::Transfer {
                        from: subscriber,
                        to: this_program,
                        amount: price * period.to_units(),
                    },
                    0,
                )
                .unwrap_or_else(|e| {
                    panic!("UpdateSubscription: error sending async message: {e:?}")
                })
                .await
                .unwrap_or_else(|e| {
                    panic!("UpdateSubscription: transfer ended up with an error {e:?}")
                });

                if msg::send_delayed(
                    this_program,
                    Actions::UpdateSubscription { subscriber },
                    0,
                    period.to_blocks(),
                )
                .is_ok()
                {
                    add_subscriber(
                        subscriber,
                        SubscriberData {
                            payment_method,
                            period,
                            subscription_start: Some((current_date, current_block)),
                            renewal_date: Some((
                                current_date + period.to_millis(),
                                current_block + period.to_blocks(),
                            )),
                        },
                    );
                } else {
                    add_subscriber(
                        subscriber,
                        SubscriberData {
                            payment_method,
                            period,
                            renewal_date: None,
                            subscription_start: None,
                        },
                    );
                }
            } else {
                delete_subscriber(&subscriber);
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
                new_data.renewal_date = None;

                new_data
            };

            add_subscriber(subscriber, updated_subscription);
        }
        Actions::ManagePendingSubscription { enable } => {
            let subscriber = msg::source();
            let this_program = exec::program_id();

            if let Some(SubscriberData {
                subscription_start,
                period,
                payment_method,
                ..
            }) = get_subscriber(&subscriber).copied()
            {
                if subscription_start.is_some() {
                    panic!("ManagePendingSubscription: subscription is not pending");
                }

                if enable {
                    msg::send_delayed(
                        this_program,
                        Actions::UpdateSubscription { subscriber },
                        0,
                        period.to_blocks(),
                    )
                    .unwrap_or_else(|e| {
                        panic!("ManagePendingSubscription: sending delayed message failed {e:?}")
                    });

                    let current_date = exec::block_timestamp();
                    let current_block = exec::block_height();
                    add_subscriber(
                        subscriber,
                        SubscriberData {
                            payment_method,
                            period,
                            subscription_start: Some((current_date, current_block)),
                            renewal_date: Some((
                                current_date + period.to_millis(),
                                current_block + period.to_blocks(),
                            )),
                        },
                    );
                } else {
                    let price = get_price(&payment_method)
                        .expect("ManagePendingSubscription: payment method was deleted");
                    let _: FTEvent = msg::send_for_reply_as(
                        payment_method,
                        FTAction::Transfer {
                            from: this_program,
                            to: subscriber,
                            amount: price * period.to_units(),
                        },
                        0,
                    )
                    .unwrap_or_else(|e| {
                        panic!("ManagePendingSubscription: error sending async message: {e:?}")
                    })
                    .await
                    .unwrap_or_else(|e| {
                        panic!("ManagePendingSubscription: transfer ended up with an error {e:?}")
                    });

                    delete_subscriber(&subscriber);
                }
            } else {
                panic!("ManagePendingSubscription: can't manage non existing subscription");
            }
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

fn get_subscriber(subscriber: &ActorId) -> Option<&SubscriberData> {
    unsafe { SUBSCRIBERS.get(subscriber) }
}

fn delete_subscriber(subscriber: &ActorId) {
    unsafe {
        SUBSCRIBERS.remove(subscriber);
    }
}
