// IA-64 (Itanium) SysV psABI.
//
// NOTE: this is a *minimal placeholder* sufficient to compile and emit for
// scalar-only signatures (Phase 1 of the IA-64 bring-up). It passes every
// aggregate indirectly, which is NOT the real psABI: the actual convention
// flattens small structs / homogeneous FP aggregates into up to 8 register
// slots with "Next Even" alignment, matching clang's IA-64 TargetInfo. That
// accurate classification is Phase 2 — see ../../../../llvm-project-misc/rust_bringup.html
// and the clang `ia64-abi.c` test cases.

use rustc_abi::TyAbiInterface;

use crate::callconv::{ArgAbi, FnAbi};

fn classify_ret<Ty>(ret: &mut ArgAbi<'_, Ty>) {
    if ret.layout.is_aggregate() {
        ret.make_indirect();
    } else {
        ret.extend_integer_width_to(64);
    }
}

fn classify_arg<'a, Ty, C>(cx: &C, arg: &mut ArgAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
{
    if arg.layout.pass_indirectly_in_non_rustic_abis(cx) {
        arg.make_indirect();
        return;
    }
    if arg.layout.is_aggregate() {
        arg.make_indirect();
    } else {
        arg.extend_integer_width_to(64);
    }
}

pub(crate) fn compute_abi_info<'a, Ty, C>(cx: &C, fn_abi: &mut FnAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
{
    if !fn_abi.ret.is_ignore() {
        classify_ret(&mut fn_abi.ret);
    }

    for arg in fn_abi.args.iter_mut() {
        if arg.is_ignore() {
            continue;
        }
        classify_arg(cx, arg);
    }
}
