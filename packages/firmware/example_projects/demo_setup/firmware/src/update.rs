use embassy_net::Stack;

#[embassy_executor::task]
pub async fn update_task(mut stack: Stack<'static>, port: u16) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        todo!()
    }
}
