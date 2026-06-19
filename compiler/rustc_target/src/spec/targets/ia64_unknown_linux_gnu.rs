use crate::spec::{Arch, Cc, LinkerFlavor, Lld, PanicStrategy, Target, TargetMetadata, base, cvs};

pub(crate) fn target() -> Target {
    let mut base = base::linux_gnu::opts();
    base.max_atomic_width = Some(64);
    // The IA-64 GNU `as`/`ld` is the old ia64-only binutils (dropped upstream),
    // whose `ld` is a strict single-pass-over-archives linker. rustc injects the
    // panic runtime (panic_abort) to the LEFT of libstd, but libstd has a
    // backward reference into it (__rust_start_panic / __rust_panic_cleanup, both
    // #[rustc_std_internal_symbol]); a single-pass ld can't resolve that. Bracket
    // the whole object/rlib/native-lib span in --start-group/--end-group so ld
    // iterates the archives to a fixpoint.
    base.add_pre_link_args(LinkerFlavor::Gnu(Cc::Yes, Lld::No), &["-Wl,--start-group"]);
    base.add_late_link_args(LinkerFlavor::Gnu(Cc::Yes, Lld::No), &["-Wl,--end-group"]);
    // EH landing-pad lowering is not implemented in the IA-64 backend yet, so
    // default to aborting panics (Phase 5 lifts this). See rust_bringup.html.
    base.panic_strategy = PanicStrategy::Abort;
    // The IA-64 LLVM backend has no integrated assembler (no MC object writer), so
    // rustc emits assembly and shells out to GNU `as`. Program defaults to
    // `ia64-unknown-linux-gnu-as` from PATH; override with `-Cassembler=`.
    base.need_external_assembler = true;
    // GNU `as` for IA-64 defaults to "auto" template mode, which re-bundles
    // instructions and *ignores* the explicit stop bits (`;;`) the backend
    // emits — corrupting dependency ordering (and tripping a symbols.c assert).
    // `-x` selects explicit mode so our bundling/stops are honored, exactly as
    // clang's IA-64 path drives `as`.
    base.asm_args = cvs!["-x"];

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
