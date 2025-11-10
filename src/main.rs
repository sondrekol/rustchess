mod client;


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