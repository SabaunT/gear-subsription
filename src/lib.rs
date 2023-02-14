#![no_std]

use gstd::{debug, msg, prelude::*, ActorId};

// todo add metadata.
// todo  control tokens are of erc20 standard
// todo error workflow done by eco-system guys

static mut SUPPORTED_TOKENS: BTreeSet<ActorId> = BTreeSet::new();
static mut SUBSCRIBERS: BTreeSet<ActorId> = BTreeSet::new();

#[no_mangle]
extern "C" fn init() {
    // Add provided token
    add_token(msg::load().expect("wrong payload: expected token id"));
}

#[no_mangle]
extern "C" fn handle() {
    // let new_msg = String::from_utf8(msg::load_bytes().expect("Unable to load bytes"))
    //     .expect("Invalid message");

    // if new_msg == "PING" {
    //     msg::reply_bytes("PONG", 0).expect("Unable to reply");
    // }

    // unsafe {
    //     MESSAGE_LOG.push(new_msg);

    //     debug!("{:?} total message(s) stored: ", MESSAGE_LOG.len());

    //     for log in &MESSAGE_LOG {
    //         debug!(log);
    //     }
    // }
}

fn add_token(token: ActorId) {
    unsafe {
        SUPPORTED_TOKENS.insert(token);
    }
}
