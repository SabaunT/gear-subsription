#![no_std]

use ft_io::{FTAction, FTEvent};
use gear_subscription_io::{Actions, Period, Price, SubscriberData, SubscriptionState};
use gstd::{async_main, exec, msg, prelude::*, ActorId};

// todo control tokens are of erc20 standard
// todo error workflow done by eco-system guys
// todo control errors between async calls
// todo add readme + docs
// todo add tests
// todo metahash

static mut SUPPORTED_TOKENS: BTreeMap<ActorId, Price> = BTreeMap::new();
static mut SUBSCRIBERS: BTreeMap<ActorId, SubscriberData> = BTreeMap::new();

#[no_mangle]
extern "C" fn init() {
    // Add provided tokensubscriber
    add_token(msg::load().expect("wrong payload: expected token id"));
}

#[async_main]
async fn main() {
    match msg::load().expect("wrong payload: expected `Actions`") {
        Actions::RegisterSubscription {
            period,
            payment_method,
            with_renewal,
        } => {
            let subscriber = msg::source();
            let price = get_price(&payment_method).expect("register sub: no such token");
            let _: FTEvent = msg::send_for_reply_as(
                payment_method,
                FTAction::Transfer {
                    from: subscriber,
                    to: exec::program_id(),
                    amount: price,
                },
                0,
            )
            .expect("error sending async message")
            .await
            .expect("some other error has to be handled");

            let _ = msg::send_delayed(
                exec::program_id(),
                Actions::CheckSubscription { subscriber },
                0,
                period.to_blocks(),
            )
            .expect("failed");

            let start_date = exec::block_timestamp();
            let start_block = exec::block_height();
            add_subscriber(
                subscriber,
                SubscriberData {
                    with_renewal,
                    payment_method,
                    subscription_start: (start_date, start_block),
                    period,
                    renewal_date: (
                        start_date + Period::ThirtySecs.to_millis(),
                        start_block + Period::ThirtySecs.to_blocks(),
                    ),
                },
            );
        }
        Actions::CheckSubscription { subscriber } => {
            // todo update end date
            let sub_data = get_subscriber(&subscriber).expect("no data");
            if sub_data.with_renewal
                && sub_data.subscription_start.1 + sub_data.period.to_blocks()
                    >= exec::block_height()
            {
                let price =
                    get_price(&sub_data.payment_method).expect("register sub: no such token");
                let _: FTEvent = msg::send_for_reply_as(
                    sub_data.payment_method,
                    FTAction::Transfer {
                        from: subscriber,
                        to: exec::program_id(),
                        amount: price,
                    },
                    0,
                )
                .expect("error sending async message")
                .await
                .expect("some other error has to be handled");

                let _ = msg::send_delayed(
                    exec::program_id(),
                    Actions::CheckSubscription { subscriber },
                    0,
                    Period::ThirtySecs.to_blocks(),
                )
                .expect("failed");

                let (renewal_date, renewal_block) = sub_data.renewal_date;
                let renewal_date = renewal_date + Period::ThirtySecs.to_millis();
                let renewal_block = renewal_block + Period::ThirtySecs.to_blocks();
                add_subscriber(
                    subscriber,
                    SubscriberData {
                        with_renewal: true,
                        payment_method: sub_data.payment_method,
                        subscription_start: sub_data.subscription_start,
                        period: sub_data.period,
                        renewal_date: (renewal_date, renewal_block),
                    },
                );
            } else {
                // todo handle remove subscription
                delete_subscriber(&subscriber)
            }
        }
        Actions::CancelSubscription { subscriber } => {
            todo!()
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
