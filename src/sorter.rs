use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
pub struct Sorter<T> {
    tx: UnboundedSender<T>,
}
