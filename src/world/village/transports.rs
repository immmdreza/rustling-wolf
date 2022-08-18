use std::future::Future;

use tokio::sync::mpsc::Receiver;

pub struct Transporter {}

impl Transporter {
    pub fn spawn<K, G, Gut, Args>(
        name: String,
        mut receiver: Receiver<K>,
        callback: G,
        args: Args,
    ) -> tokio::task::JoinHandle<()>
    where
        K: Sync + Send + 'static,
        G: FnOnce(K, Args) -> Gut + Send + Copy + 'static,
        Gut: Future<Output = ()> + Send,
        Args: Sync + Send + Clone + 'static,
    {
        let spawned = tokio::spawn(async move {
            loop {
                let received = receiver.recv().await;
                match received {
                    Some(received) => {
                        callback(received, args.to_owned()).await;
                        continue;
                    }
                    None => {
                        println!("A transporter is down ... ( {} )", name);
                        return;
                    }
                }
            }
        });

        spawned
    }
}
