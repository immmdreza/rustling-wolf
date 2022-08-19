use std::fmt::Debug;

use tokio::sync::mpsc::Receiver;

pub fn setup_console_receiver<T>(converter: fn(String) -> T) -> Receiver<T>
where
    T: Debug + Send + Sync + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::channel(1024);
    tokio::spawn(async move {
        // read from console and send watchers
        let mut input = String::new();
        loop {
            input.clear();
            std::io::stdin().read_line(&mut input).unwrap();
            tx.send(converter(input.trim().to_string())).await.unwrap();
        }
    });

    rx
}
