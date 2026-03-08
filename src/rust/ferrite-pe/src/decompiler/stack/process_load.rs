//! Opcode handlers: load/store constants, args, locals, fields, and calls.

use super::expressions::*;
use super::*;

impl<'a> StackSimulator<'a> {
    /// Handles: load constants, load/store args, load/store locals, address-of.
    pub(super) fn process_loads_and_stores(&mut self, instr: &Instruction) -> bool {
        match instr.opcode {
            OpCode::LdcI4M1 => self.push(Expr::Int(-1)),
            OpCode::LdcI4_0 => self.push(Expr::Int(0)),
            OpCode::LdcI4_1 => self.push(Expr::Int(1)),
            OpCode::LdcI4_2 => self.push(Expr::Int(2)),
            OpCode::LdcI4_3 => self.push(Expr::Int(3)),
            OpCode::LdcI4_4 => self.push(Expr::Int(4)),
            OpCode::LdcI4_5 => self.push(Expr::Int(5)),
            OpCode::LdcI4_6 => self.push(Expr::Int(6)),
            OpCode::LdcI4_7 => self.push(Expr::Int(7)),
            OpCode::LdcI4_8 => self.push(Expr::Int(8)),
            OpCode::LdcI4S => {
                if let Operand::Int8(v) = instr.operand {
                    self.push(Expr::Int(v as i64));
                }
            }
            OpCode::LdcI4 => {
                if let Operand::Int32(v) = instr.operand {
                    self.push(Expr::Int(v as i64));
                }
            }
            OpCode::LdcI8 => {
                if let Operand::Int64(v) = instr.operand {
                    self.push(Expr::Int(v));
                }
            }
            OpCode::LdcR4 => {
                if let Operand::Float32(v) = instr.operand {
                    self.push(Expr::Float(v as f64));
                }
            }
            OpCode::LdcR8 => {
                if let Operand::Float64(v) = instr.operand {
                    self.push(Expr::Float(v));
                }
            }
            OpCode::Ldnull => self.push(Expr::Null),
            OpCode::Ldstr => match &instr.operand {
                Operand::StringToken(tok) => {
                    let s = self.resolver.resolve_user_string(*tok);
                    self.push(Expr::String(s));
                }
                Operand::ResolvedName(s) => self.push(Expr::String(s.clone())),
                _ => {}
            },
            OpCode::Ldarg0 => {
                let e = self.load_arg(0);
                self.push(e);
            }
            OpCode::Ldarg1 => {
                let e = self.load_arg(1);
                self.push(e);
            }
            OpCode::Ldarg2 => {
                let e = self.load_arg(2);
                self.push(e);
            }
            OpCode::Ldarg3 => {
                let e = self.load_arg(3);
                self.push(e);
            }
            OpCode::LdargS | OpCode::Ldarg => {
                if let Operand::Var(idx) = instr.operand {
                    let e = self.load_arg(idx);
                    self.push(e);
                }
            }
            OpCode::Ldloc0 => {
                let e = self.load_local(0);
                self.push(e);
            }
            OpCode::Ldloc1 => {
                let e = self.load_local(1);
                self.push(e);
            }
            OpCode::Ldloc2 => {
                let e = self.load_local(2);
                self.push(e);
            }
            OpCode::Ldloc3 => {
                let e = self.load_local(3);
                self.push(e);
            }
            OpCode::LdlocS | OpCode::Ldloc => {
                if let Operand::Var(idx) = instr.operand {
                    let e = self.load_local(idx);
                    self.push(e);
                }
            }
            OpCode::Stloc0 => self.store_local(0),
            OpCode::Stloc1 => self.store_local(1),
            OpCode::Stloc2 => self.store_local(2),
            OpCode::Stloc3 => self.store_local(3),
            OpCode::StlocS | OpCode::Stloc => {
                if let Operand::Var(idx) = instr.operand {
                    self.store_local(idx);
                }
            }
            OpCode::StargS | OpCode::Starg => {
                if let Operand::Var(idx) = instr.operand {
                    self.store_arg(idx);
                }
            }
            OpCode::LdlocaS | OpCode::Ldloca => {
                if let Operand::Var(idx) = instr.operand {
                    let local = self.load_local(idx);
                    self.push(Expr::AddressOf(Box::new(local)));
                }
            }
            OpCode::LdargaS | OpCode::Ldarga => {
                if let Operand::Var(idx) = instr.operand {
                    let arg = self.load_arg(idx);
                    self.push(Expr::AddressOf(Box::new(arg)));
                }
            }
            _ => return false,
        }
        true
    }

    /// Handles: field loads/stores, calls, newobj.
    pub(super) fn process_fields_and_calls(&mut self, instr: &Instruction) -> bool {
        match instr.opcode {
            OpCode::Ldfld => {
                let full_name = resolve_operand_name(self.resolver, &instr.operand);
                let obj = self.pop();
                let field_name = strip_type_prefix(&full_name);
                self.push(Expr::Field(Box::new(obj), field_name));
            }
            OpCode::Stfld => {
                let full_name = resolve_operand_name(self.resolver, &instr.operand);
                let value = self.pop();
                let obj = self.pop();
                let field_name = strip_type_prefix(&full_name);
                let target = Expr::Field(Box::new(obj), field_name);
                self.emit(Statement::Assign(target, value));
            }
            OpCode::Ldsfld => {
                let name = resolve_operand_name(self.resolver, &instr.operand);
                let (type_name, field_name) = split_qualified(&name);
                self.push(Expr::StaticField(type_name, field_name));
            }
            OpCode::Stsfld => {
                let name = resolve_operand_name(self.resolver, &instr.operand);
                let value = self.pop();
                let (type_name, field_name) = split_qualified(&name);
                let target = Expr::StaticField(type_name, field_name);
                self.emit(Statement::Assign(target, value));
            }
            OpCode::Ldflda => {
                let full_name = resolve_operand_name(self.resolver, &instr.operand);
                let obj = self.pop();
                let field_name = strip_type_prefix(&full_name);
                self.push(Expr::AddressOf(Box::new(Expr::Field(
                    Box::new(obj),
                    field_name,
                ))));
            }
            OpCode::Ldsflda => {
                let name = resolve_operand_name(self.resolver, &instr.operand);
                let (type_name, field_name) = split_qualified(&name);
                self.push(Expr::AddressOf(Box::new(Expr::StaticField(
                    type_name, field_name,
                ))));
            }
            OpCode::Call | OpCode::Callvirt => {
                let (name, param_count, has_return, is_static_call) =
                    resolve_call_info_from_operand(self.resolver, &instr.operand);
                let mut args: Vec<Expr> = (0..param_count).map(|_| self.pop()).collect();
                args.reverse();
                let expr = if is_static_call {
                    let (type_name, method_name) = split_qualified(&name);
                    Expr::StaticCall(type_name, method_name, args)
                } else {
                    let obj = self.pop();
                    let (_, method_name) = split_qualified(&name);
                    Expr::Call(Some(Box::new(obj)), method_name, args)
                };
                if has_return {
                    self.push(expr);
                } else {
                    self.emit(Statement::Expr(expr));
                }
            }
            OpCode::Newobj => {
                let (name, param_count, _, _) =
                    resolve_call_info_from_operand(self.resolver, &instr.operand);
                let mut args: Vec<Expr> = (0..param_count).map(|_| self.pop()).collect();
                args.reverse();
                let (type_name, _) = split_qualified(&name);
                self.push(Expr::NewObj(type_name, args));
            }
            _ => return false,
        }
        true
    }
}
