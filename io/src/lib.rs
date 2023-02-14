#![no_std]

use gmeta::{Metadata, In};
use gstd::{prelude::*, ActorId};

pub type TokenData = (ActorId, Price);
pub type Price = u128;

pub struct SubscriptionMetadata;

impl Metadata for SubscriptionMetadata {
    type Init = In<TokenData>;
    type Handle = In<Actions>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = ();
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Actions {
    RegisterSubscription {
        payment_method: ActorId,
        period: Period,
    },
    CheckSubscription {
        subscriber: ActorId,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Period {
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

    pub fn to_blocks(&self) -> u32 {
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

#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriptionState {
    pub subscribers: Vec<ActorId>,
}