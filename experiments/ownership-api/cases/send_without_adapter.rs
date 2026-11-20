use std::{future::Future, pin::Pin};
fn erased_send<F>(callback: &mut F) -> Pin<Box<dyn Future<Output = u32> + Send + '_>>
where
    F: AsyncFnMut(u32) -> u32 + Send,
{
    Box::pin(async move { callback(1).await })
}
fn main() {}
