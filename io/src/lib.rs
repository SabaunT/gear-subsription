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
    UpdateSubscription {
        subscriber: ActorId,
    },
    CancelSubscription,
    ManagePendingSubscription {
        enable: bool,
    },
}

#[derive(Debug, Clone, Copy, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Period {
    Year,
    NineMonths,
    SixMonths,
    ThreeMonths,
    #[default]
    Month,
}

impl Period {
    // todo Must be changeable
    const TARGET_BLOCK_TIME: u32 = Self::SECOND;
    // const MONTH: u32 = Self::DAY * 30;
    // const DAY: u32 = Self::HOUR * 24;
    // const HOUR: u32 = Self::MINUTE * 60;
    // const MINUTE: u32 = Self::SECOND * 60;
    const SECOND: u32 = 1;

    pub fn minimal_unit() -> Self {
        Self::Month
    }

    pub fn to_units(&self) -> u128 {
        match self {
            Period::Year => 12,
            Period::NineMonths => 9,
            Period::SixMonths => 6,
            Period::ThreeMonths => 3,
            Period::Month => 1,
        }
    }

    pub fn to_blocks(&self) -> u32 {
        self.to_secs() / Self::TARGET_BLOCK_TIME
    }

    pub fn to_millis(&self) -> u64 {
        self.to_secs() as u64 * 1000
    }

    fn to_secs(&self) -> u32 {
        match self {
            Period::Year => Self::Month.to_secs() * 12,
            Period::NineMonths => Self::Month.to_secs() * 9,
            Period::SixMonths => Self::Month.to_secs() * 6,
            Period::ThreeMonths => Self::Month.to_secs() * 3,
            Period::Month => Self::SECOND * 30,
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

#[derive(Debug, Clone, Copy, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriberData {
    pub payment_method: ActorId,
    pub period: Period,
    // todo this must be calculated off-chain
    pub subscription_start: Option<(u64, u32)>,
    // todo this must be calculated off-chain
    pub renewal_date: Option<(u64, u32)>,
}

#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct SubscriberDataState {
    pub is_active: bool,
    pub start_date: u64,
    pub start_block: u32,
    pub end_date: u64,
    pub end_block: u32,
    pub period: Period,
    pub will_renew: bool,
    pub price: u128,
}
