#![no_std]

use gmeta::{In, Metadata};
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
    type State = SubscriptionState;
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Actions {
    RegisterSubscription {
        payment_method: ActorId,
        period: Period,
        with_renewal: bool,
    },
    CheckSubscription {
        subscriber: ActorId,
    },
    CancelSubscription {
        subscriber: ActorId,
    },
}

#[derive(Debug, Clone, Copy, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Period {
    // Month,
    // ThreeMonths,
    // SixMonths,
    OneMinuteTenSecs,
    Minute,
    #[default]
    ThirtySecs, // todo for test
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
        self.to_secs() / Self::TARGET_BLOCK_TIME
    }

    pub fn to_millis(&self) -> u64 {
        self.to_secs() as u64 * 1000
    }

    fn to_secs(&self) -> u32 {
        match self {
            Period::OneMinuteTenSecs => Self::MINUTE + Self::SECOND * 10,
            Period::Minute => Self::MINUTE,
            // Period::Month => Self::MONTH,
            // Period::ThreeMonths => Self::MONTH * 3,
            // Period::SixMonths => Self::MONTH * 6,
            // Period::NineMonths => Self::MONTH * 9,
            // Period::Year => Self::MONTH * 12,
            Period::ThirtySecs => Self::SECOND * 30,
        }
    }
}

#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriptionState {
    pub subscribers: BTreeMap<ActorId, SubscriberData>,
    pub payment_methods: BTreeMap<ActorId, Price>,
}

type V = (BTreeMap<ActorId, SubscriberData>, BTreeMap<ActorId, Price>);

impl From<V> for SubscriptionState {
    fn from(value: V) -> Self {
        let (one, two) = value;
        SubscriptionState {
            subscribers: one,
            payment_methods: two,
        }
    }
}

#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriberData {
    pub with_renewal: bool,
    pub payment_method: ActorId,
    pub subscription_start: (u64, u32),
    pub period: Period,
    pub renewal_date: (u64, u32),
}

#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriberDataState {
    pub is_active: bool,
    pub start_date: u64,
    pub end_date: u64,
    pub start_block: u32,
    pub end_block: u32,
    pub period: Period,
    pub renewal_date: u64,  // Option
    pub renewal_block: u32, // Option
    pub price: Option<u128>,
}
