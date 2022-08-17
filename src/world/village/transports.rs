use std::{sync::mpsc::Receiver, thread, time::Duration};

pub struct Transporter {}

impl Transporter {
    pub fn spawn<T, K>(receiver: Receiver<K>, callback: fn(K, &T) -> (), extra: T) -> ()
    where
        T: Send + 'static,
        K: Send + 'static,
    {
        thread::spawn(move || loop {
            loop {
                let received = receiver.recv();
                match received {
                    Ok(received) => {
                        callback(received, &extra);
                        continue;
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
