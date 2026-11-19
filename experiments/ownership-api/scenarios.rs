use ownership_probe::{Call, Error, Event, Plan, Result, Scoped, Script, block_on, yield_once};
use std::{future::poll_fn, task::Poll};

#[derive(Debug, PartialEq)]
struct Summary(String);

#[test]
fn borrowed_read_only_tool_and_typed_result() {
    let prefix = String::from("observed");
    let mut calls = 0;
    let plan = Plan::<Summary>::new()
        .text("Summarize:")
        .quote("input", "document")
        .tool("inspect", "Inspect a value", async |value: String| {
            yield_once().await;
            calls += 1;
            Ok(format!("{prefix}: {value}"))
        });
    assert_eq!(plan.prompt(), "Summarize:<document>input</document>");
    let result = block_on(plan.run(Script::new(
        vec![Call::new("inspect", String::from("sample"))],
        Summary("fixture".into()),
    )));
    assert_eq!(result, Ok(Summary("fixture".into())));
    assert_eq!(calls, 1);
    assert_eq!(prefix, "observed");
}

#[test]
fn exclusive_capture_survives_suspension_and_returns_to_caller() {
    let mut findings = Vec::new();
    let plan = Plan::<()>::new().tool("record", "Record a finding", async |value: u32| {
        findings.push(value);
        yield_once().await;
        findings.push(value + 1);
        Ok(())
    });
    block_on(plan.run(Script::new(
        vec![Call::new("record", 10_u32), Call::new("record", 20_u32)],
        (),
    )))
    .unwrap();
    assert_eq!(findings, [10, 11, 20, 21]);
}

/// A generic formatting helper preserves either reusable or scoped state.
fn instruction<T, State>(plan: Plan<'_, T, State>) -> Plan<'_, T, State> {
    plan.text(" Be concise.")
}

/// Normalize once to Scoped; both branches now have an identical public type.
fn optional_counter<'a>(
    plan: Plan<'a, (), Scoped>,
    enabled: bool,
    count: &'a mut u32,
) -> Plan<'a, (), Scoped> {
    if enabled {
        plan.tool("count", "Count a unit", async |(): ()| {
            *count += 1;
            yield_once().await;
            Ok(())
        })
    } else {
        plan
    }
}

#[test]
fn clone_base_and_execute_variations_concurrently() {
    let base = Plan::<()>::new().text("Analyze.");
    let mut left_values = Vec::new();
    let mut right_values = Vec::new();
    let left = instruction(base.clone()).tool("left", "Left tool", async |n: u32| {
        yield_once().await;
        left_values.push(n);
        Ok(())
    });
    let right = base
        .clone()
        .text(" Explain.")
        .tool("right", "Right tool", async |n: u32| {
            yield_once().await;
            right_values.push(n);
            Ok(())
        });
    assert_eq!(base.prompt(), "Analyze.");
    assert_eq!(left.prompt(), "Analyze. Be concise.");
    assert_eq!(right.prompt(), "Analyze. Explain.");
    block_on(async {
        let mut left = std::pin::pin!(left.run(Script::new(vec![Call::new("left", 1_u32)], ())));
        let mut right = std::pin::pin!(right.run(Script::new(vec![Call::new("right", 2_u32)], ())));
        let mut left_done = false;
        let mut right_done = false;
        poll_fn(|cx| {
            if !left_done && let Poll::Ready(result) = left.as_mut().poll(cx) {
                result.unwrap();
                left_done = true;
            }
            if !right_done && let Poll::Ready(result) = right.as_mut().poll(cx) {
                result.unwrap();
                right_done = true;
            }
            if left_done && right_done {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    });
    assert_eq!(left_values, [1]);
    assert_eq!(right_values, [2]);
}

#[test]
fn conditional_helpers_and_heterogeneous_tools() {
    for enabled in [false, true] {
        let mut count = 0;
        let plan = optional_counter(Plan::<()>::new().scoped(), enabled, &mut count).tool(
            "label",
            "Accept a different input type",
            async |s: String| Ok(s.len()),
        );
        let mut calls = vec![Call::new("label", String::from("hello"))];
        if enabled {
            calls.push(Call::new("count", ()));
        }
        block_on(plan.run(Script::new(calls, ()))).unwrap();
        assert_eq!(count, u32::from(enabled));
    }
}

#[test]
fn typed_errors_and_cleanup() {
    let mut calls = 0;
    let plan = Plan::<()>::new().tool("fail", "Deliberate error", async |(): ()| -> Result<()> {
        calls += 1;
        yield_once().await;
        Err(Error::Tool("fixture failure"))
    });
    assert_eq!(
        block_on(plan.run(Script::new(vec![Call::new("fail", ())], ()))),
        Err(Error::Tool("fixture failure"))
    );
    assert_eq!(calls, 1);
    let plan = Plan::<()>::new().tool("number", "Typed input", async |_: u32| Ok(()));
    assert_eq!(
        block_on(plan.run(Script::new(vec![Call::new("number", "wrong")], ()))),
        Err(Error::InputType)
    );
    assert_eq!(
        block_on(Plan::<u32>::new().run(Script::new(vec![], "wrong"))),
        Err(Error::ResultType)
    );
    assert_eq!(
        block_on(Plan::<()>::new().run(Script::new(vec![Call::new("absent", ())], ()))),
        Err(Error::UnknownTool("absent".into()))
    );
}

#[test]
fn observe_events_while_borrowed_tool_is_running() {
    let mut values = Vec::new();
    let execution = Plan::<u32>::new()
        .text("inspect")
        .tool("record", "Record", async |n: u32| {
            yield_once().await;
            values.push(n);
            Ok(())
        })
        .execute(Script::new(vec![Call::new("record", 7_u32)], 42_u32));
    let (mut result, mut events) = execution.into_parts();
    let mut seen = Vec::new();
    let mut output = None;
    block_on(poll_fn(|cx| {
        if output.is_none()
            && let Poll::Ready(value) = result.as_mut().poll(cx)
        {
            output = Some(value);
        }
        while let Poll::Ready(event) = events.poll_next(cx) {
            match event {
                Some(event) => seen.push(event),
                None => return Poll::Ready(()),
            }
        }
        Poll::Pending
    }));
    drop(result); // A completed future may retain captures until it is dropped.
    assert_eq!(output, Some(Ok(42)));
    assert_eq!(values, [7]);
    assert_eq!(
        seen,
        [
            Event::Prompt("inspect".into()),
            Event::Started("record".into()),
            Event::Finished("record".into())
        ]
    );
    assert_eq!(events.dropped(), 0);
}

#[test]
fn dropping_observation_mid_run_does_not_cancel() {
    let mut calls = 0;
    let (mut result, mut events) = Plan::<()>::new()
        .tool("count", "Count", async |(): ()| {
            yield_once().await;
            calls += 1;
            Ok(())
        })
        .execute(Script::new(
            vec![Call::new("count", ()), Call::new("count", ())],
            (),
        ))
        .into_parts();
    block_on(poll_fn(|cx| {
        assert!(result.as_mut().poll(cx).is_pending());
        match events.poll_next(cx) {
            Poll::Ready(Some(Event::Started(_))) => Poll::Ready(()),
            _ => Poll::Pending,
        }
    }));
    drop(events);
    block_on(result).unwrap();
    assert_eq!(calls, 2);
}

#[test]
fn unread_events_are_bounded_and_end_after_result() {
    let (result, mut events) = Plan::<()>::new()
        .tool("noop", "Noop", async |(): ()| Ok(()))
        .execute(Script::new(
            (0..10).map(|_| Call::new("noop", ())).collect(),
            (),
        ))
        .into_parts();
    block_on(result).unwrap();
    assert_eq!(events.buffered(), 2);
    assert_eq!(events.dropped(), 19); // 1 prompt + 20 tool events, capacity 2.
    assert!(block_on(events.next()).is_some());
    assert!(block_on(events.next()).is_some());
    assert_eq!(block_on(events.next()), None);
}

#[test]
fn output_inference_and_direct_await() {
    let summary: Summary = block_on(async {
        Plan::new()
            .text("Infer the result type from its use.")
            .execute(Script::new(vec![], Summary("inferred".into())))
            .await
            .unwrap()
    });
    assert_eq!(summary, Summary("inferred".into()));
}
