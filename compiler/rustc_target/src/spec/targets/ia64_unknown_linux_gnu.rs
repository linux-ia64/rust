use crate::spec::{Arch, PanicStrategy, Target, TargetMetadata, base};

pub(crate) fn target() -> Target {
    let mut base = base::linux_gnu::opts();
    base.max_atomic_width = Some(64);
    // EH landing-pad lowering is not implemented in the IA-64 backend yet, so
    // default to aborting panics (Phase 5 lifts this). See rust_bringup.html.
    base.panic_strategy = PanicStrategy::Abort;

    Target {
        llvm_target: "ia64-unknown-linux-gnu".into(),
        metadata: TargetMetadata {
            description: Some("IA-64 (Itanium) Linux (kernel 4.4, glibc 2.23)".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: Some(false),
        },
        pointer_width: 64,
        data_layout: "e-m:e-p:64:64-i64:64-f80:128-n8:16:32:64-S128".into(),
        arch: Arch::IA64,
        options: base,
    }
}
