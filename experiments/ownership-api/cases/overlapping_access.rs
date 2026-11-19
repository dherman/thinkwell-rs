use ownership_probe::{Plan, Script, block_on};
fn main() {
    let mut values = Vec::new();
    let plan = Plan::<()>::new().tool("record", "record", async |n: u32| {
        values.push(n);
        Ok(())
    });
    values.push(99);
    block_on(plan.run(Script::new(vec![], ()))).unwrap();
}
