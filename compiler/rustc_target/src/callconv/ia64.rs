// IA-64 (Itanium) SysV psABI.
//
// Reference: IA-64 Software Conventions and Runtime Architecture Guide
// (`/data/documents/IA64conventions.pdf`), §8.5 "Parameter Passing" and §8.6
// "Result Return"; matches Clang's `IA64ABIInfo::classifyArgumentType` /
// `classifyReturnType` (`clang/lib/CodeGen/Targets/IA64.cpp`).
//
// Scalars occupy one output GR (extended to 64 bits) or, for floating point,
// one FP register (handled by the backend's `CC_IA64*` custom hooks). Small
// aggregates (structs/unions/enums) are passed *by value*, flattened into
// consecutive 64-bit integer slots -- there is no hidden-pointer form for an
// aggregate that fits in the eight parameter slots (<= 64 bytes). A
// homogeneous floating-point aggregate (all `f32` or all `f64`, up to eight
// members) is instead flattened into that many FP registers. Aggregate
// return values follow the same flattening into r8-r11 (up to 256 bits);
// bigger ones are returned via a caller-allocated buffer (sret).
//
// A 16-byte-aligned aggregate argument must start on an even parameter slot
// ("Next Even", psABI Table 8-1). We track the running slot offset and, if
// such an aggregate would start on an odd slot, burn that slot with a padding
// argument -- reusing `cast_to_and_pad_i32`, since IA64's calling convention
// promotes any sub-64-bit integer to a full 64-bit slot, an `i32` padding
// argument consumes exactly one parameter slot here.
//
// Not implemented: Next-Even padding ahead of a *scalar* argument (e.g. a
// 128-bit integer) that isn't part of an aggregate. IA64's LLVM calling
// convention (`IA64CallingConv.td`) has no rule for such scalars in the first
// place, and Rust has no stable by-value type that would exercise it.

use rustc_abi::{Align, HasDataLayout, RegKind, Size, TyAbiInterface, TyAndLayout};

use crate::callconv::{ArgAbi, FnAbi, Reg, Uniform};

/// Homogeneous floating-point aggregate: all leaf fields are `f32`, or all
/// are `f64`, with at most eight members (they are passed/returned in
/// F8-F15).
fn homogeneous_fp_aggregate<'a, Ty, C>(cx: &C, layout: &TyAndLayout<'a, Ty>) -> Option<Uniform>
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    layout.homogeneous_aggregate(cx).ok().and_then(|ha| ha.unit()).and_then(|unit| {
        if unit.kind != RegKind::Float {
            return None;
        }
        if layout.size > unit.size.checked_mul(8, cx).unwrap() {
            return None;
        }
        Some(Uniform::consecutive(unit, layout.size))
    })
}

fn classify_ret<'a, Ty, C>(cx: &C, ret: &mut ArgAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    if !ret.layout.is_aggregate() {
        ret.extend_integer_width_to(64);
        return;
    }

    if let Some(uniform) = homogeneous_fp_aggregate(cx, &ret.layout) {
        ret.cast_to(uniform);
        return;
    }

    // Aggregates up to 256 bits return by value in r8-r11, flattened into
    // 64-bit slots; larger ones are returned via a caller-allocated buffer
    // whose address is passed in r8 (sret).
    if ret.layout.size.bits() > 256 {
        ret.make_indirect();
        return;
    }
    ret.cast_to(Uniform::new(Reg::i64(), ret.layout.size));
}

fn classify_arg<'a, Ty, C>(cx: &C, arg: &mut ArgAbi<'a, Ty>, offset: &mut Size)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    if arg.layout.pass_indirectly_in_non_rustic_abis(cx) {
        arg.make_indirect();
        return;
    }

    if !arg.layout.is_aggregate() {
        arg.extend_integer_width_to(64);
        *offset += arg.layout.size.align_to(Align::EIGHT);
        return;
    }

    // "Next Even": a 16-byte-aligned aggregate must start on an even
    // parameter slot. If it would land on an odd one, burn that slot with a
    // padding argument.
    let pad = arg.layout.align.abi.bytes() >= 16 && offset.bytes() % 16 != 0;
    if pad {
        *offset += Size::from_bytes(8);
    }

    let cast = match homogeneous_fp_aggregate(cx, &arg.layout) {
        Some(uniform) => uniform,
        None => Uniform::new(Reg::i64(), arg.layout.size),
    };
    arg.cast_to_and_pad_i32(cast, pad);
    *offset += arg.layout.size.align_to(Align::EIGHT);
}

pub(crate) fn compute_abi_info<'a, Ty, C>(cx: &C, fn_abi: &mut FnAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    if !fn_abi.ret.is_ignore() {
        classify_ret(cx, &mut fn_abi.ret);
    }

    // Slot offset tracking for the "Next Even" alignment rule; a large
    // aggregate return value's sret buffer address travels in r8 and takes
    // no parameter slot, so the count always starts at 0.
    let mut offset = Size::ZERO;
    for arg in fn_abi.args.iter_mut() {
        if arg.is_ignore() {
            continue;
        }
        classify_arg(cx, arg, &mut offset);
    }
}
