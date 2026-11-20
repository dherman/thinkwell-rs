//! Isolated ownership experiment. No protocol, schema, or production API.
use std::{
    any::Any,
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    future::{Future, IntoFuture, poll_fn},
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

pub type Payload = Box<dyn Any>;
pub type LocalFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    UnknownTool(String),
    InputType,
    ResultType,
    Tool(&'static str),
}

trait Tool {
    fn call(&mut self, input: Payload) -> LocalFuture<'_, Result<Payload>>;
}

struct Callback<F, I, O> {
    function: F,
    types: PhantomData<fn(I) -> O>,
}

impl<F, I, O> Tool for Callback<F, I, O>
where
    F: AsyncFnMut(I) -> Result<O>,
    I: 'static,
    O: 'static,
{
    fn call(&mut self, input: Payload) -> LocalFuture<'_, Result<Payload>> {
        Box::pin(async move {
            let input = *input.downcast::<I>().map_err(|_| Error::InputType)?;
            let output = (self.function)(input).await?;
            Ok(Box::new(output) as Payload)
        })
    }
}

pub struct Reusable;
pub struct Scoped;

/// Only plans without scoped callbacks are cloneable in this experiment.
/// The state parameter changes once, not once per callback type.
pub struct Plan<'a, T, State = Reusable> {
    prompt: Vec<String>,
    tools: BTreeMap<String, Box<dyn Tool + 'a>>,
    types: PhantomData<fn() -> (T, State)>,
}

impl<T> Default for Plan<'_, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Plan<'_, T> {
    pub fn new() -> Self {
        Self {
            prompt: Vec::new(),
            tools: BTreeMap::new(),
            types: PhantomData,
        }
    }
}

impl<T> Clone for Plan<'_, T> {
    fn clone(&self) -> Self {
        // Reusable is constructible only without callbacks.
        Self {
            prompt: self.prompt.clone(),
            tools: BTreeMap::new(),
            types: PhantomData,
        }
    }
}

impl<'a, T, State> Plan<'a, T, State> {
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.prompt.push(text.into());
        self
    }

    pub fn quote(self, content: &str, label: &str) -> Self {
        self.text(format!("<{label}>{content}</{label}>"))
    }

    pub fn prompt(&self) -> String {
        self.prompt.concat()
    }

    /// Normalize the type before a helper conditionally adds scoped callbacks.
    pub fn scoped(self) -> Plan<'a, T, Scoped> {
        Plan {
            prompt: self.prompt,
            tools: self.tools,
            types: PhantomData,
        }
    }

    pub fn tool<I, O, F>(self, name: &str, _description: &str, function: F) -> Plan<'a, T, Scoped>
    where
        I: 'static,
        O: 'static,
        F: AsyncFnMut(I) -> Result<O> + 'a,
    {
        let mut plan = self.scoped();
        plan.tools.insert(
            name.to_owned(),
            Box::new(Callback {
                function,
                types: PhantomData,
            }),
        );
        plan
    }
}

pub struct Call {
    name: String,
    input: Payload,
}

impl Call {
    pub fn new<I: 'static>(name: &str, input: I) -> Self {
        Self {
            name: name.into(),
            input: Box::new(input),
        }
    }
}

/// In-memory fixture; output deliberately bypasses schemas and model behavior.
pub struct Script {
    calls: Vec<Call>,
    result: Payload,
}

impl Script {
    pub fn new<T: 'static>(calls: Vec<Call>, result: T) -> Self {
        Self {
            calls,
            result: Box::new(result),
        }
    }
}

impl<'a, T: 'static, State> Plan<'a, T, State> {
    pub fn execute(self, script: Script) -> Execution<'a, T> {
        // State has no runtime role and is removed before creating the future.
        let mut tools = self.tools;
        let prompt = self.prompt.concat();
        let (sink, events) = event_channel();
        let result = Box::pin(async move {
            sink.push(Event::Prompt(prompt));
            yield_once().await;
            for call in script.calls {
                sink.push(Event::Started(call.name.clone()));
                yield_once().await;
                let tool = tools
                    .get_mut(&call.name)
                    .ok_or_else(|| Error::UnknownTool(call.name.clone()))?;
                let _output = tool.call(call.input).await?;
                sink.push(Event::Finished(call.name));
                yield_once().await;
            }
            script
                .result
                .downcast::<T>()
                .map(|v| *v)
                .map_err(|_| Error::ResultType)
        });
        Execution { result, events }
    }

    pub async fn run(self, script: Script) -> Result<T> {
        self.execute(script).await
    }
}

pub struct Execution<'a, T> {
    result: LocalFuture<'a, Result<T>>,
    events: Events,
}

impl<'a, T> Execution<'a, T> {
    pub fn into_parts(self) -> (LocalFuture<'a, Result<T>>, Events) {
        (self.result, self.events)
    }
}

impl<'a, T> IntoFuture for Execution<'a, T> {
    type Output = Result<T>;
    type IntoFuture = LocalFuture<'a, Result<T>>;
    fn into_future(self) -> Self::IntoFuture {
        // Dropping the unused observer releases its queue; execution remains alive.
        self.result
    }
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Prompt(String),
    Started(String),
    Finished(String),
}

const EVENT_CAPACITY: usize = 2;
struct EventState {
    queue: VecDeque<Event>,
    observing: bool,
    closed: bool,
    dropped: usize,
    waker: Option<Waker>,
}
struct Sink(Rc<RefCell<EventState>>);
pub struct Events(Rc<RefCell<EventState>>);

fn event_channel() -> (Sink, Events) {
    let state = Rc::new(RefCell::new(EventState {
        queue: VecDeque::new(),
        observing: true,
        closed: false,
        dropped: 0,
        waker: None,
    }));
    (Sink(state.clone()), Events(state))
}

impl Sink {
    fn push(&self, event: Event) {
        let wake = {
            let mut state = self.0.borrow_mut();
            if !state.observing {
                return;
            }
            if state.queue.len() == EVENT_CAPACITY {
                state.queue.pop_front();
                state.dropped += 1;
            }
            state.queue.push_back(event);
            state.waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
    }
}

impl Drop for Sink {
    fn drop(&mut self) {
        let wake = {
            let mut state = self.0.borrow_mut();
            state.closed = true;
            state.waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
    }
}

impl Events {
    pub fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Event>> {
        let mut state = self.0.borrow_mut();
        if let Some(event) = state.queue.pop_front() {
            Poll::Ready(Some(event))
        } else if state.closed {
            Poll::Ready(None)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        poll_fn(|cx| self.poll_next(cx)).await
    }

    pub fn buffered(&self) -> usize {
        self.0.borrow().queue.len()
    }

    pub fn dropped(&self) -> usize {
        self.0.borrow().dropped
    }
}

impl Drop for Events {
    fn drop(&mut self) {
        let mut state = self.0.borrow_mut();
        state.observing = false;
        state.queue.clear();
        state.waker = None;
    }
}

/// Force a real Pending boundary so captures are exercised across suspension.
pub async fn yield_once() {
    let mut yielded = false;
    poll_fn(|cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await
}

struct ThreadWake(std::thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

/// Minimal executor for fixtures only; not a runtime proposal.
pub fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::park(),
        }
    }
}
