use ownership_probe::{Plan, Scoped};
fn escaped() -> Plan<'static, (), Scoped> {
    let mut values = Vec::new();
    Plan::new().tool("record", "record", async |n: u32| {
        values.push(n);
        Ok(())
    })
}
fn main() {
    let _ = escaped();
}
