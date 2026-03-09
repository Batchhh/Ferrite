use dotnetdll::prelude::*;

use crate::il::{OpCode, Operand};

use super::super::formatting::format_types::format_method_type;
use super::super::resolve::{
    resolve_field_source_name, resolve_method_source_call_info, resolve_method_source_name,
};
use super::super::Assembly;

/// Convert a dotnetdll resolved IL instruction to an `(OpCode, Operand)` pair.
///
/// String literals are interned into `asm.user_strings` and replaced with a synthetic token.
pub(in crate::assembly) fn convert_instruction(
    instr: &dotnetdll::resolved::il::Instruction,
    res: &Resolution,
    asm: &mut Assembly,
) -> (OpCode, Operand) {
    use dotnetdll::resolved::il::Instruction as DI;
    use dotnetdll::resolved::il::NumberSign;

    match instr {
        DI::NoOperation => (OpCode::Nop, Operand::None),
        DI::Breakpoint => (OpCode::Break, Operand::None),
        DI::LoadArgument(idx) => match idx {
            0 => (OpCode::Ldarg0, Operand::None),
            1 => (OpCode::Ldarg1, Operand::None),
            2 => (OpCode::Ldarg2, Operand::None),
            3 => (OpCode::Ldarg3, Operand::None),
            _ => (OpCode::LdargS, Operand::Var(*idx)),
        },
        DI::LoadArgumentAddress(idx) => (OpCode::LdargaS, Operand::Var(*idx)),
        DI::StoreArgument(idx) => (OpCode::StargS, Operand::Var(*idx)),
        DI::LoadLocal(idx) => match idx {
            0 => (OpCode::Ldloc0, Operand::None),
            1 => (OpCode::Ldloc1, Operand::None),
            2 => (OpCode::Ldloc2, Operand::None),
            3 => (OpCode::Ldloc3, Operand::None),
            _ => (OpCode::LdlocS, Operand::Var(*idx)),
        },
        DI::LoadLocalAddress(idx) => (OpCode::LdlocaS, Operand::Var(*idx)),
        DI::StoreLocal(idx) => match idx {
            0 => (OpCode::Stloc0, Operand::None),
            1 => (OpCode::Stloc1, Operand::None),
            2 => (OpCode::Stloc2, Operand::None),
            3 => (OpCode::Stloc3, Operand::None),
            _ => (OpCode::StlocS, Operand::Var(*idx)),
        },
        DI::LoadNull => (OpCode::Ldnull, Operand::None),
        DI::LoadConstantInt32(v) => match v {
            -1 => (OpCode::LdcI4M1, Operand::None),
            0 => (OpCode::LdcI4_0, Operand::None),
            1 => (OpCode::LdcI4_1, Operand::None),
            2 => (OpCode::LdcI4_2, Operand::None),
            3 => (OpCode::LdcI4_3, Operand::None),
            4 => (OpCode::LdcI4_4, Operand::None),
            5 => (OpCode::LdcI4_5, Operand::None),
            6 => (OpCode::LdcI4_6, Operand::None),
            7 => (OpCode::LdcI4_7, Operand::None),
            8 => (OpCode::LdcI4_8, Operand::None),
            _ => (OpCode::LdcI4, Operand::Int32(*v)),
        },
        DI::LoadConstantInt64(v) => (OpCode::LdcI8, Operand::Int64(*v)),
        DI::LoadConstantFloat32(v) => (OpCode::LdcR4, Operand::Float32(*v)),
        DI::LoadConstantFloat64(v) => (OpCode::LdcR8, Operand::Float64(*v)),
        DI::Duplicate => (OpCode::Dup, Operand::None),
        DI::Pop => (OpCode::Pop, Operand::None),
        DI::Return => (OpCode::Ret, Operand::None),

        DI::LoadString(chars) => {
            let s = String::from_utf16_lossy(chars);
            let token = asm.intern_string(s);
            (OpCode::Ldstr, Operand::StringToken(token))
        }

        DI::Branch(target) => (OpCode::Br, Operand::BranchTarget(*target as i64)),
        DI::BranchTruthy(target) => (OpCode::Brtrue, Operand::BranchTarget(*target as i64)),
        DI::BranchFalsy(target) => (OpCode::Brfalse, Operand::BranchTarget(*target as i64)),
        DI::BranchEqual(target) => (OpCode::Beq, Operand::BranchTarget(*target as i64)),
        DI::BranchNotEqual(target) => (OpCode::BneUn, Operand::BranchTarget(*target as i64)),
        DI::BranchGreater(NumberSign::Signed, target) => {
            (OpCode::Bgt, Operand::BranchTarget(*target as i64))
        }
        DI::BranchGreater(NumberSign::Unsigned, target) => {
            (OpCode::BgtUn, Operand::BranchTarget(*target as i64))
        }
        DI::BranchGreaterOrEqual(NumberSign::Signed, target) => {
            (OpCode::Bge, Operand::BranchTarget(*target as i64))
        }
        DI::BranchGreaterOrEqual(NumberSign::Unsigned, target) => {
            (OpCode::BgeUn, Operand::BranchTarget(*target as i64))
        }
        DI::BranchLess(NumberSign::Signed, target) => {
            (OpCode::Blt, Operand::BranchTarget(*target as i64))
        }
        DI::BranchLess(NumberSign::Unsigned, target) => {
            (OpCode::BltUn, Operand::BranchTarget(*target as i64))
        }
        DI::BranchLessOrEqual(NumberSign::Signed, target) => {
            (OpCode::Ble, Operand::BranchTarget(*target as i64))
        }
        DI::BranchLessOrEqual(NumberSign::Unsigned, target) => {
            (OpCode::BleUn, Operand::BranchTarget(*target as i64))
        }
        DI::Switch(targets) => {
            let t: Vec<i64> = targets.iter().map(|&t| t as i64).collect();
            (OpCode::Switch, Operand::Switch(t))
        }
        DI::Leave(target) => (OpCode::Leave, Operand::BranchTarget(*target as i64)),

        DI::Add => (OpCode::Add, Operand::None),
        DI::Subtract => (OpCode::Sub, Operand::None),
        DI::Multiply => (OpCode::Mul, Operand::None),
        DI::Divide(NumberSign::Signed) => (OpCode::Div, Operand::None),
        DI::Divide(NumberSign::Unsigned) => (OpCode::DivUn, Operand::None),
        DI::Remainder(NumberSign::Signed) => (OpCode::Rem, Operand::None),
        DI::Remainder(NumberSign::Unsigned) => (OpCode::RemUn, Operand::None),
        DI::Negate => (OpCode::Neg, Operand::None),
        DI::And => (OpCode::And, Operand::None),
        DI::Or => (OpCode::Or, Operand::None),
        DI::Xor => (OpCode::Xor, Operand::None),
        DI::Not => (OpCode::Not, Operand::None),
        DI::ShiftLeft => (OpCode::Shl, Operand::None),
        DI::ShiftRight(NumberSign::Signed) => (OpCode::Shr, Operand::None),
        DI::ShiftRight(NumberSign::Unsigned) => (OpCode::ShrUn, Operand::None),

        DI::CompareEqual => (OpCode::Ceq, Operand::None),
        DI::CompareGreater(NumberSign::Signed) => (OpCode::Cgt, Operand::None),
        DI::CompareGreater(NumberSign::Unsigned) => (OpCode::CgtUn, Operand::None),
        DI::CompareLess(NumberSign::Signed) => (OpCode::Clt, Operand::None),
        DI::CompareLess(NumberSign::Unsigned) => (OpCode::CltUn, Operand::None),

        // Method calls — store call info in "name|param_count|has_return|is_static" format
        DI::Call {
            param0: ref method_source,
            ..
        }
        | DI::CallConstrained(_, ref method_source) => {
            let (name, pc, hr, is_st) = resolve_method_source_call_info(method_source, res);
            let info = format!(
                "{}|{}|{}|{}",
                name,
                pc,
                if hr { "1" } else { "0" },
                if is_st { "1" } else { "0" }
            );
            (OpCode::Call, Operand::ResolvedName(info))
        }
        DI::CallVirtual {
            param0: ref method_source,
            ..
        }
        | DI::CallVirtualConstrained(_, ref method_source)
        | DI::CallVirtualTail(ref method_source) => {
            let (name, pc, hr, is_st) = resolve_method_source_call_info(method_source, res);
            let info = format!(
                "{}|{}|{}|{}",
                name,
                pc,
                if hr { "1" } else { "0" },
                if is_st { "1" } else { "0" }
            );
            (OpCode::Callvirt, Operand::ResolvedName(info))
        }
        DI::NewObject(ref user_method) => {
            let (name, pc, hr, is_st) =
                resolve_method_source_call_info(&MethodSource::User(*user_method), res);
            let info = format!(
                "{}|{}|{}|{}",
                name,
                pc,
                if hr { "1" } else { "0" },
                if is_st { "1" } else { "0" }
            );
            (OpCode::Newobj, Operand::ResolvedName(info))
        }
        DI::Jump(ref method_source) => {
            let name = resolve_method_source_name(method_source, res);
            (OpCode::Jmp, Operand::ResolvedName(name))
        }

        DI::LoadField { param0: ref fs, .. } | DI::LoadFieldSkipNullCheck(ref fs) => {
            let name = resolve_field_source_name(fs, res);
            (OpCode::Ldfld, Operand::ResolvedName(name))
        }
        DI::LoadFieldAddress(ref fs) => {
            let name = resolve_field_source_name(fs, res);
            (OpCode::Ldflda, Operand::ResolvedName(name))
        }
        DI::StoreField { param0: ref fs, .. } | DI::StoreFieldSkipNullCheck(ref fs) => {
            let name = resolve_field_source_name(fs, res);
            (OpCode::Stfld, Operand::ResolvedName(name))
        }
        DI::LoadStaticField { param0: ref fs, .. } => {
            let name = resolve_field_source_name(fs, res);
            (OpCode::Ldsfld, Operand::ResolvedName(name))
        }
        DI::LoadStaticFieldAddress(ref fs) => {
            let name = resolve_field_source_name(fs, res);
            (OpCode::Ldsflda, Operand::ResolvedName(name))
        }
        DI::StoreStaticField { param0: ref fs, .. } => {
            let name = resolve_field_source_name(fs, res);
            (OpCode::Stsfld, Operand::ResolvedName(name))
        }

        DI::CastClass { param0: ref mt, .. } => {
            let name = format_method_type(mt, res);
            (OpCode::Castclass, Operand::ResolvedName(name))
        }
        DI::IsInstance(ref mt) => {
            let name = format_method_type(mt, res);
            (OpCode::Isinst, Operand::ResolvedName(name))
        }
        DI::BoxValue(ref mt) => {
            let name = format_method_type(mt, res);
            (OpCode::Box, Operand::ResolvedName(name))
        }
        DI::UnboxIntoValue(ref mt) => {
            let name = format_method_type(mt, res);
            (OpCode::UnboxAny, Operand::ResolvedName(name))
        }
        DI::UnboxIntoAddress { param0: ref mt, .. } => {
            let name = format_method_type(mt, res);
            (OpCode::Unbox, Operand::ResolvedName(name))
        }
        DI::NewArray(ref mt) => {
            let name = format_method_type(mt, res);
            (OpCode::Newarr, Operand::ResolvedName(name))
        }
        DI::LoadLength => (OpCode::Ldlen, Operand::None),
        DI::InitializeForObject(ref mt) => {
            let name = format_method_type(mt, res);
            (OpCode::Initobj, Operand::ResolvedName(name))
        }
        DI::Sizeof(ref mt) => {
            let name = format_method_type(mt, res);
            (OpCode::Sizeof, Operand::ResolvedName(name))
        }

        DI::LoadElement { .. } => (OpCode::Ldelem, Operand::None),
        DI::LoadElementPrimitive { .. } => (OpCode::LdelemI4, Operand::None),
        DI::LoadElementAddress { param0: ref mt, .. } => {
            let name = format_method_type(mt, res);
            (OpCode::Ldelema, Operand::ResolvedName(name))
        }
        DI::LoadElementAddressReadonly(ref mt) => {
            let name = format_method_type(mt, res);
            (OpCode::Ldelema, Operand::ResolvedName(name))
        }
        DI::StoreElement { .. } => (OpCode::Stelem, Operand::None),
        DI::StoreElementPrimitive { .. } => (OpCode::StelemI4, Operand::None),

        DI::LoadTokenType(ref mt) => {
            let name = format_method_type(mt, res);
            (OpCode::Ldtoken, Operand::ResolvedName(name))
        }
        DI::LoadTokenField(ref fs) => {
            let name = resolve_field_source_name(fs, res);
            (OpCode::Ldtoken, Operand::ResolvedName(name))
        }
        DI::LoadTokenMethod(ref ms) => {
            let name = resolve_method_source_name(ms, res);
            (OpCode::Ldtoken, Operand::ResolvedName(name))
        }

        DI::LoadMethodPointer(ref ms) => {
            let name = resolve_method_source_name(ms, res);
            (OpCode::Ldftn, Operand::ResolvedName(name))
        }
        DI::LoadVirtualMethodPointer { param0: ref ms, .. } => {
            let name = resolve_method_source_name(ms, res);
            (OpCode::Ldvirtftn, Operand::ResolvedName(name))
        }

        DI::Convert(_)
        | DI::ConvertOverflow(_, _)
        | DI::ConvertFloat32
        | DI::ConvertFloat64
        | DI::ConvertUnsignedToFloat
        | DI::CheckFinite => (OpCode::Nop, Operand::None),

        DI::LoadObject { .. } => (OpCode::Ldobj, Operand::None),
        DI::StoreObject { .. } => (OpCode::Stobj, Operand::None),
        DI::CopyObject(_) => (OpCode::Cpobj, Operand::None),

        DI::AddOverflow(NumberSign::Signed) => (OpCode::AddOvf, Operand::None),
        DI::AddOverflow(NumberSign::Unsigned) => (OpCode::AddOvfUn, Operand::None),
        DI::SubtractOverflow(NumberSign::Signed) => (OpCode::SubOvf, Operand::None),
        DI::SubtractOverflow(NumberSign::Unsigned) => (OpCode::SubOvfUn, Operand::None),
        DI::MultiplyOverflow(NumberSign::Signed) => (OpCode::MulOvf, Operand::None),
        DI::MultiplyOverflow(NumberSign::Unsigned) => (OpCode::MulOvfUn, Operand::None),

        DI::Throw => (OpCode::Throw, Operand::None),
        DI::Rethrow => (OpCode::Rethrow, Operand::None),
        DI::EndFinally => (OpCode::Endfinally, Operand::None),
        DI::EndFilter => (OpCode::Endfilter, Operand::None),

        DI::LoadIndirect { .. } => (OpCode::LdindI4, Operand::None),
        DI::StoreIndirect { .. } => (OpCode::StindI4, Operand::None),

        DI::LocalMemoryAllocate => (OpCode::Localloc, Operand::None),
        DI::ArgumentList => (OpCode::Arglist, Operand::None),
        DI::CopyMemoryBlock { .. } => (OpCode::Cpblk, Operand::None),
        DI::InitializeMemoryBlock { .. } => (OpCode::Initblk, Operand::None),
        DI::MakeTypedReference(_) => (OpCode::Mkrefany, Operand::None),
        DI::ReadTypedReferenceType => (OpCode::Refanytype, Operand::None),
        DI::ReadTypedReferenceValue(_) => (OpCode::Refanyval, Operand::None),
        DI::CallIndirect { .. } => (OpCode::Calli, Operand::None),
    }
}
