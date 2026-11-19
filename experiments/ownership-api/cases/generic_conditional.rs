// Isolate the type-changing-storage question without a protocol implementation.
struct GenericPlan<Tools>(Tools);
impl<Tools> GenericPlan<Tools> {
    fn tool<F>(self, f: F) -> GenericPlan<(Tools, F)> {
        GenericPlan((self.0, f))
    }
}
fn configure(enabled: bool) -> impl Sized {
    let base = GenericPlan(());
    if enabled {
        base.tool(async |n: u32| n + 1)
    } else {
        base
    }
}
fn main() {
    let _ = configure(true);
}
