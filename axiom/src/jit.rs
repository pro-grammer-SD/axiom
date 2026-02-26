/// Axiom JIT — Entry Point Verification
///
/// Lightweight module that inspects parsed items and optionally
/// locates the `main()` function. Having a `main()` is NOT required;
/// top-level statements are executed directly.

use miette::Result;

/// Check whether a `main` function is present in the item list.
/// Always succeeds — `main()` is optional in Axiom.
pub fn prepare_jit_entry(items: &[crate::ast::Item]) -> Result<()> {
    let _has_main = items.iter().any(|item| {
        if let crate::ast::Item::FunctionDecl { name, .. } = item {
            name == "main"
        } else {
            false
        }
    });
    // main() is optional; top-level statements execute regardless.
    Ok(())
}
