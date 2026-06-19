//! This module contains the implementation of the `eh_personality` lang item.
//!
//! The actual implementation is heavily dependent on the target since Rust
//! tries to use the native stack unwinding mechanism whenever possible.
//!
//! This personality function is still required with `-C panic=abort` because
//! it is used to catch foreign exceptions from `extern "C-unwind"` and turn
//! them into aborts.
//!
//! Additionally, ARM EHABI uses the personality function when generating
//! backtraces.

mod dwarf;

#[cfg(not(any(test, doctest)))]
cfg_select! {
    target_os = "emscripten" => {
        mod emcc;
    }
    any(target_env = "msvc", target_family = "wasm", target_os = "motor") => {
        // This is required by the compiler to exist (e.g., it's a lang item),
        // but it's never actually called by the compiler because
        // __CxxFrameHandler3 (msvc) / __gxx_wasm_personality_v0 (wasm) is the
        // personality function that is always used.  Hence this is just an
        // aborting stub.
        #[lang = "eh_personality"]
        fn rust_eh_personality() {
            core::intrinsics::abort()
        }
    }
    target_arch = "ia64" => {
        // IA-64 unwinds via native table-based info (.IA_64.unwind), not DWARF
        // CFI, and EH is not yet implemented (panic=abort only -- Phase 5). The
        // generic `gcc` personality decodes DWARF LSDA pointers via
        // _Unwind_Get{Text,Data}RelBase, which don't exist in ia64 libgcc_s /
        // libunwind. Until native ia64 EH lands, provide an aborting stub like
        // the msvc/wasm cases above.
        #[lang = "eh_personality"]
        fn rust_eh_personality() {
            core::intrinsics::abort()
        }
    }
    any(
        all(target_family = "windows", target_env = "gnu"),
        target_os = "psp",
        target_os = "xous",
        target_os = "solid_asp3",
        all(target_family = "unix", not(target_os = "espidf"), not(target_os = "l4re"), not(target_os = "nuttx")),
        all(target_vendor = "fortanix", target_env = "sgx"),
    ) => {
        mod gcc;
    }
    _ => {
        // Targets that don't support unwinding.
        // - os=none ("bare metal" targets)
        // - os=uefi
        // - os=espidf
        // - os=hermit
        // - nvptx64-nvidia-cuda
        // - arch=avr
    }
}
