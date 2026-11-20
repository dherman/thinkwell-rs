struct GenericPlan<Tools>(Tools);
impl<Tools> GenericPlan<Tools> {
    fn tool<F>(self, f: F) -> GenericPlan<(Tools, F)> {
        GenericPlan((self.0, f))
    }
}
// A helper can return an opaque, changed tool-chain type.
fn add<T>(plan: GenericPlan<T>) -> GenericPlan<impl Sized> {
    plan.tool(async |n: u32| n + 1)
}
fn main() {
    let _ = add(GenericPlan(())).tool(async |s: String| s.len());
}
