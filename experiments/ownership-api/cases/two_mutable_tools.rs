use ownership_probe::{Plan, Script, block_on};
fn main() {
    let mut values = Vec::new();
    let plan = Plan::<()>::new()
        .tool("first", "first", async |n: u32| {
            values.push(n);
            Ok(())
        })
        .tool("second", "second", async |n: u32| {
            values.push(n);
            Ok(())
        });
    block_on(plan.run(Script::new(vec![], ()))).unwrap();
}
