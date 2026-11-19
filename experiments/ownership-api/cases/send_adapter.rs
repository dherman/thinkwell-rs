//! An explicit adapter can state the Send bound on each borrowed invocation.
use ownership_probe::{block_on, yield_once};
use std::{future::Future, pin::Pin};
type SendFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

fn invoke<'a, F, Adapter>(callback: &'a mut F, adapter: &Adapter) -> SendFuture<'a, u32>
where
    F: AsyncFnMut(u32) -> u32 + Send,
    Adapter: for<'b> Fn(&'b mut F, u32) -> SendFuture<'b, u32>,
{
    adapter(callback, 5)
}
fn assert_send<T: Send>(_: &T) {}
fn main() {
    let mut values = Vec::new();
    let mut callback = async |n: u32| {
        yield_once().await;
        values.push(n);
        n + 1
    };
    // No macro is required, but this is additional caller syntax.
    let result = invoke(&mut callback, &|callback, n| Box::pin(callback(n)));
    assert_send(&result);
    assert_eq!(block_on(result), 6);
    drop(callback);
    assert_eq!(values, [5]);
}
