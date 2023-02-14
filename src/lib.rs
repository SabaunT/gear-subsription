#![no_std]

use ft_io::{FTAction, FTEvent};
use gstd::{async_main, exec, msg, prelude::*, ActorId};

// todo add metadata.
// todo add metastate.
// todo  control tokens are of erc20 standard
// todo error workflow done by eco-system guys
// todo add readme
// todo add tests

type Price = u128;

static mut SUPPORTED_TOKENS: BTreeMap<ActorId, Price> = BTreeMap::new();
static mut SUBSCRIBERS: BTreeSet<ActorId> = BTreeSet::new();

#[derive(Debug, Encode, Decode)]
#[codec(crate = gstd::codec)]
enum Actions {
    RegisterSubscription {
        payment_method: ActorId,
        period: Period,
    },
    CheckSubscription {
        subscriber: ActorId,
    },
}

#[derive(Debug, Encode, Decode)]
#[codec(crate = gstd::codec)]
enum Period {
    Month,
    ThreeMonths,
    SixMonths,
    NineMonths,
    Year,
    FiveMinutes, // todo for test
}

impl Period {
    // todo Must be changeable
    const TARGET_BLOCK_TIME: u32 = Self::SECOND;
    const MONTH: u32 = Self::DAY * 30;

    const DAY: u32 = Self::HOUR * 24;
    const HOUR: u32 = Self::MINUTE * 60;
    const MINUTE: u32 = Self::SECOND * 60;
    const SECOND: u32 = 1;

    fn to_blocks(&self) -> u32 {
        let time = match self {
            Period::Month => Self::MONTH,
            Period::ThreeMonths => Self::MONTH * 3,
            Period::SixMonths => Self::MONTH * 6,
            Period::NineMonths => Self::MONTH * 9,
            Period::Year => Self::MONTH * 12,
            Period::FiveMinutes => Self::MINUTE * 5,
        };

        time / Self::TARGET_BLOCK_TIME
    }
}

#[no_mangle]
extern "C" fn init() {
    // Add provided token
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
                    subscriber: subscriber,
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
