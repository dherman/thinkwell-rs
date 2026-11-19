use ownership_probe::Plan;
fn main() {
    let mut values = Vec::new();
    let plan = Plan::<()>::new().tool("record", "record", async |n: u32| {
        values.push(n);
        Ok(())
    });
    let _copy = plan.clone();
}
