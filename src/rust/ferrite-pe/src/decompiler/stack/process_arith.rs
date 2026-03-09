//! Opcode handlers: arithmetic, type ops, array ops, control flow, and misc.

use super::expressions::resolve_operand_name;
use super::*;

impl<'a> StackSimulator<'a> {
    /// Handles: arithmetic, unary, comparisons, type/object operations, ldtoken.
    pub(super) fn process_arith_and_types(&mut self, instr: &Instruction) -> bool {
        match instr.opcode {
            OpCode::Add => self.binary_op(BinOp::Add),
            OpCode::AddOvf | OpCode::AddOvfUn => self.binary_op(BinOp::AddChecked),
            OpCode::Sub => self.binary_op(BinOp::Sub),
            OpCode::SubOvf | OpCode::SubOvfUn => self.binary_op(BinOp::SubChecked),
            OpCode::Mul => self.binary_op(BinOp::Mul),
            OpCode::MulOvf | OpCode::MulOvfUn => self.binary_op(BinOp::MulChecked),
            OpCode::Div | OpCode::DivUn => self.binary_op(BinOp::Div),
            OpCode::Rem | OpCode::RemUn => self.binary_op(BinOp::Rem),
            OpCode::And => self.binary_op(BinOp::And),
            OpCode::Or => self.binary_op(BinOp::Or),
            OpCode::Xor => self.binary_op(BinOp::Xor),
            OpCode::Shl => self.binary_op(BinOp::Shl),
            OpCode::Shr | OpCode::ShrUn => self.binary_op(BinOp::Shr),
            OpCode::Neg => {
                let val = self.pop();
                self.push(Expr::Unary(UnaryOp::Neg, Box::new(val)));
            }
            OpCode::Not => {
                let val = self.pop();
                self.push(Expr::Unary(UnaryOp::Not, Box::new(val)));
            }
            OpCode::Ceq => self.binary_op(BinOp::Eq),
            OpCode::Cgt | OpCode::CgtUn => self.binary_op(BinOp::Gt),
            OpCode::Clt | OpCode::CltUn => self.binary_op(BinOp::Lt),
            OpCode::Castclass => {
                let type_name = resolve_operand_name(self.resolver, &instr.operand);
                let val = self.pop();
                self.push(Expr::Cast(type_name, Box::new(val)));
            }
            OpCode::Isinst => {
                let type_name = resolve_operand_name(self.resolver, &instr.operand);
                let val = self.pop();
                self.push(Expr::AsInst(Box::new(val), type_name));
            }
            OpCode::Box => {
                let type_name = resolve_operand_name(self.resolver, &instr.operand);
                let val = self.pop();
                self.push(Expr::Box(type_name, Box::new(val)));
            }
            OpCode::Unbox => {
                let type_name = resolve_operand_name(self.resolver, &instr.operand);
                let val = self.pop();
                self.push(Expr::Unbox(type_name, Box::new(val)));
            }
            OpCode::UnboxAny => {
                let type_name = resolve_operand_name(self.resolver, &instr.operand);
                let val = self.pop();
                self.push(Expr::Cast(type_name, Box::new(val)));
            }
            OpCode::Newarr => {
                let type_name = resolve_operand_name(self.resolver, &instr.operand);
                let size = self.pop();
                self.push(Expr::ArrayNew(type_name, Box::new(size)));
            }
            OpCode::Ldlen => {
                let arr = self.pop();
                self.push(Expr::ArrayLength(Box::new(arr)));
            }
            OpCode::Sizeof => {
                let type_name = resolve_operand_name(self.resolver, &instr.operand);
                self.push(Expr::Sizeof(type_name));
            }
            OpCode::Initobj => {
                let type_name = resolve_operand_name(self.resolver, &instr.operand);
                let _addr = self.pop();
                self.push(Expr::Default(type_name));
            }
            OpCode::Ldtoken => {
                let name = resolve_operand_name(self.resolver, &instr.operand);
                self.push(Expr::Typeof(name));
            }
            _ => return false,
        }
        true
    }

    /// Handles: array element ops, return/throw, stack manipulation, branches,
    /// conversions, indirect ops, no-ops, and misc/catch-all.
    pub(super) fn process_arrays_and_control(&mut self, instr: &Instruction) {
        match instr.opcode {
            OpCode::LdelemI1
            | OpCode::LdelemU1
            | OpCode::LdelemI2
            | OpCode::LdelemU2
            | OpCode::LdelemI4
            | OpCode::LdelemU4
            | OpCode::LdelemI8
            | OpCode::LdelemR4
            | OpCode::LdelemR8
            | OpCode::LdelemI
            | OpCode::LdelemRef
            | OpCode::Ldelem => {
                let index = self.pop();
                let arr = self.pop();
                self.push(Expr::ArrayElement(Box::new(arr), Box::new(index)));
            }
            OpCode::Ldelema => {
                let index = self.pop();
                let arr = self.pop();
                self.push(Expr::AddressOf(Box::new(Expr::ArrayElement(
                    Box::new(arr),
                    Box::new(index),
                ))));
            }
            OpCode::StelemI
            | OpCode::StelemI1
            | OpCode::StelemI2
            | OpCode::StelemI4
            | OpCode::StelemI8
            | OpCode::StelemR4
            | OpCode::StelemR8
            | OpCode::StelemRef
            | OpCode::Stelem => {
                let value = self.pop();
                let index = self.pop();
                let arr = self.pop();
                let target = Expr::ArrayElement(Box::new(arr), Box::new(index));
                self.emit(Statement::Assign(target, value));
            }
            OpCode::Ret => {
                let value = if !self.stack.is_empty() {
                    Some(self.pop())
                } else {
                    None
                };
                self.emit(Statement::Return(value));
            }
            OpCode::Throw => {
                let val = self.pop();
                self.emit(Statement::Throw(Some(val)));
            }
            OpCode::Rethrow => {
                self.emit(Statement::Throw(None));
            }
            OpCode::Dup => {
                let val = self.pop();
                self.push(val.clone());
                self.push(val);
            }
            OpCode::Pop => {
                let val = self.pop();
                if matches!(
                    &val,
                    Expr::Call(..) | Expr::StaticCall(..) | Expr::NewObj(..)
                ) {
                    self.emit(Statement::Expr(val));
                }
            }
            OpCode::Br | OpCode::BrS => {}
            OpCode::Brfalse | OpCode::BrfalseS | OpCode::Brtrue | OpCode::BrtrueS => {
                let _cond = self.pop();
            }
            OpCode::Beq
            | OpCode::BeqS
            | OpCode::Bge
            | OpCode::BgeS
            | OpCode::Bgt
            | OpCode::BgtS
            | OpCode::Ble
            | OpCode::BleS
            | OpCode::Blt
            | OpCode::BltS
            | OpCode::BneUn
            | OpCode::BneUnS
            | OpCode::BgeUn
            | OpCode::BgeUnS
            | OpCode::BgtUn
            | OpCode::BgtUnS
            | OpCode::BleUn
            | OpCode::BleUnS
            | OpCode::BltUn
            | OpCode::BltUnS => {
                let _v2 = self.pop();
                let _v1 = self.pop();
            }
            OpCode::Switch => {
                let _val = self.pop();
            }
            OpCode::ConvI1
            | OpCode::ConvI2
            | OpCode::ConvI4
            | OpCode::ConvI8
            | OpCode::ConvR4
            | OpCode::ConvR8
            | OpCode::ConvU4
            | OpCode::ConvU8
            | OpCode::ConvI
            | OpCode::ConvU
            | OpCode::ConvRUn
            | OpCode::ConvU2_D1
            | OpCode::ConvU1_D2
            | OpCode::ConvOvfI1
            | OpCode::ConvOvfU1
            | OpCode::ConvOvfI2
            | OpCode::ConvOvfU2
            | OpCode::ConvOvfI4
            | OpCode::ConvOvfU4
            | OpCode::ConvOvfI8
            | OpCode::ConvOvfU8
            | OpCode::ConvOvfI
            | OpCode::ConvOvfU
            | OpCode::ConvOvfI1Un
            | OpCode::ConvOvfI2Un
            | OpCode::ConvOvfI4Un
            | OpCode::ConvOvfI8Un
            | OpCode::ConvOvfU1Un
            | OpCode::ConvOvfU2Un
            | OpCode::ConvOvfU4Un
            | OpCode::ConvOvfU8Un
            | OpCode::ConvOvfIUn
            | OpCode::ConvOvfUUn => {}
            OpCode::LdindI1
            | OpCode::LdindU1
            | OpCode::LdindI2
            | OpCode::LdindU2
            | OpCode::LdindI4
            | OpCode::LdindU4
            | OpCode::LdindI8
            | OpCode::LdindI
            | OpCode::LdindR4
            | OpCode::LdindR8
            | OpCode::LdindRef => {}
            OpCode::StindRef
            | OpCode::StindI1
            | OpCode::StindI2
            | OpCode::StindI4
            | OpCode::StindI8
            | OpCode::StindR4
            | OpCode::StindR8
            | OpCode::StindI => {
                let value = self.pop();
                let addr = self.pop();
                self.emit(Statement::Assign(addr, value));
            }
            OpCode::Stobj => {
                let value = self.pop();
                let addr = self.pop();
                self.emit(Statement::Assign(addr, value));
            }
            OpCode::Ldobj => {}
            OpCode::Nop
            | OpCode::Break
            | OpCode::Leave
            | OpCode::LeaveS
            | OpCode::Endfinally
            | OpCode::Endfilter
            | OpCode::Volatile
            | OpCode::TailCall
            | OpCode::Unaligned
            | OpCode::Constrained
            | OpCode::Readonly => {}
            OpCode::Localloc => {
                let size = self.pop();
                self.push(Expr::Stackalloc("byte".into(), Box::new(size)));
            }
            OpCode::Cpobj => {
                let _src = self.pop();
                let _dst = self.pop();
            }
            OpCode::Cpblk | OpCode::Initblk => {
                let _count = self.pop();
                let _val = self.pop();
                let _addr = self.pop();
            }
            OpCode::Ckfinite => {}
            OpCode::Mkrefany | OpCode::Refanyval | OpCode::Refanytype => {
                self.push(Expr::Raw(format!("/* {:?} */", instr.opcode)));
            }
            OpCode::Arglist => {
                self.push(Expr::Raw("__arglist".into()));
            }
            OpCode::Ldftn | OpCode::Ldvirtftn => {
                let name = resolve_operand_name(self.resolver, &instr.operand);
                self.push(Expr::Raw(format!("&{}", name)));
            }
            OpCode::Calli => {
                self.push(Expr::Raw("/* calli */".into()));
            }
            OpCode::Jmp => {
                let name = resolve_operand_name(self.resolver, &instr.operand);
                self.push(Expr::Raw(format!("/* jmp {} */", name)));
            }
            // Catch-all
            #[allow(unreachable_patterns)]
            _ => {
                self.push(Expr::Raw(format!("/* {:?} */", instr.opcode)));
            }
        }
    }
}
