mod client;

use licheszter::client::Licheszter;
use futures_util::StreamExt;
use tokio::{runtime::Runtime, sync::futures};
use std::thread;

use crate::client::li_bot;


fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    rt.block_on(async {
        li_bot().await;
    });
}  