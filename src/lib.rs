#![no_std]

use ft_io::{FTAction, FTEvent};
use gstd::{async_main, exec, msg, prelude::*, ActorId};
use gear_subscription_io::{Actions, Period, Price, SubscriptionState};

// todo add metastate.
// todo  control tokens are of erc20 standard
// todo error workflow done by eco-system guys
// todo add readme
// todo add tests
// todo metahash

static mut SUPPORTED_TOKENS: BTreeMap<ActorId, Price> = BTreeMap::new();
static mut SUBSCRIBERS: BTreeSet<ActorId> = BTreeSet::new();

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
                Actions::CheckSubscription {
                    subscriber,
                },
                0,
                period.to_blocks(),
            )
            .expect("failed");

            add_subscriber(subscriber);
        }
        Actions::CheckSubscription { .. } => todo!(),
    }
}

#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    msg::reply(metahash, 0).expect("Failed to share metahash");
}

extern "C" fn state() {
    let ret_state = SubscriptionState { subscribers: SUBSCRIBERS.iter().collect() };
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

fn add_subscriber(subscriber: ActorId) {
    unsafe {
        SUBSCRIBERS.insert(subscriber);
    }
}
