mod request;

use std::future::Future;

use tokio::sync::mpsc;

pub use request::Request;

#[derive(Clone)]
pub struct Antenna<TAsked, TAnswered> {
    sender: mpsc::Sender<Request<TAsked, TAnswered>>,
}

impl<TAsked, TAnswered> Antenna<TAsked, TAnswered>
where
    TAsked: Send + 'static,
    TAnswered: Send + 'static,
{
    pub fn sender(&self) -> &mpsc::Sender<Request<TAsked, TAnswered>> {
        &self.sender
    }

    pub async fn ask(&self, asked: TAsked) -> Option<TAnswered> {
        let (req, rx) = Request::<TAsked, TAnswered>::new(asked);
        match self.sender.send(req).await {
            Ok(_) => match rx.await {
                Ok(data) => Some(data),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
}

pub struct Tower<TAsked, TAnswered>
where
    TAsked: Send + 'static,
    TAnswered: Send + 'static,
{
    receiver_buffer: usize,
    _asked_bullshit: Option<TAsked>,
    _answered_shit: Option<TAnswered>,
}

impl<TAsked, TAnswered> Tower<TAsked, TAnswered>
where
    TAsked: Send + 'static,
    TAnswered: Send + 'static,
{
    pub fn template(receiver_buffer: usize) -> Self {
        Self {
            receiver_buffer,
            _asked_bullshit: None,
            _answered_shit: None,
        }
    }

    pub fn create_antenna<F, Fut, Args>(
        &self,
        args: Args,
        on_data_received: F,
    ) -> Antenna<TAsked, TAnswered>
    where
        TAsked: Send + 'static,
        TAnswered: Send + 'static,
        F: FnOnce(mpsc::Receiver<Request<TAsked, TAnswered>>, Args) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), TAnswered>> + Send,
        Args: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel(self.receiver_buffer);
        let _ = tokio::spawn(async move { on_data_received(receiver, args).await });

        Antenna { sender }
    }

    pub fn create_antenna_manual(
        &self,
    ) -> (
        Antenna<TAsked, TAnswered>,
        mpsc::Receiver<Request<TAsked, TAnswered>>,
    ) {
        let (sender, receiver) = mpsc::channel(self.receiver_buffer);
        (Antenna { sender }, receiver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tower() {
        let tower_template = Tower::<String, String>::template(1024);

        let antenna =
            tower_template.create_antenna("Test".to_string(), |mut rx, args| async move {
                loop {
                    let received = rx.recv().await;
                    match received {
                        Some(received) => received.answer(format!("{}", args)).await.unwrap(),
                        None => todo!(),
                    }
                }
            });

        match antenna.ask("Hi".to_string()).await {
            Some(data) => assert_eq!(data, "Hello World".to_string()),
            None => todo!(),
        }
    }
}
