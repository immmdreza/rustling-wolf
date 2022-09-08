use tokio::sync::oneshot::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct Request<TAsked, TAnswered> {
    data: TAsked,
    sender: Sender<TAnswered>,
}

impl<TAsked, TAnswered> Request<TAsked, TAnswered> {
    pub(super) fn new(data: TAsked) -> (Request<TAsked, TAnswered>, Receiver<TAnswered>) {
        let (sender, receiver) = channel();
        (Self { data, sender }, receiver)
    }

    pub fn asked(&self) -> &TAsked {
        &self.data
    }

    pub async fn answer(self, data: TAnswered) -> Result<(), TAnswered> {
        self.sender.send(data)
    }

    pub fn extract(self) -> (TAsked, Sender<TAnswered>) {
        (self.data, self.sender)
    }
}
