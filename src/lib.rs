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

            let current_block = exec::block_height();
            add_subscriber(
                subscriber,
                SubscriberData {
                    with_renewal,
                    end_block: current_block + period.to_blocks(),
                    payment_method,
                },
            );
        }
        Actions::CheckSubscription { subscriber } => {
            let sub_data = get_subscriber(&subscriber).expect("no data");
            if sub_data.with_renewal {
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
            } else {
                // todo handle remove subscription
                delete_subscriber(&subscriber)
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
    let _ = msg::reply(ret_state, 0);
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
