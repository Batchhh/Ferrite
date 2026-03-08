use super::*;
use crate::decompiler::ast::*;
use crate::decompiler::emit::emit_expr;
use accessors::{rewrite_prop_expr, rewrite_prop_stmt, rewrite_property_accessors};
use booleans::coerce_booleans;
use compiler::simplify_self_references;
use conditions::simplify_expr;
use delegates::propagate_delegate_assignments;
use lambdas::{extract_method_name_from_ref, inline_lambda_expr, is_delegate_type};
use loops_for::{detect_for_loops, expr_contains_local_idx};

#[test]
fn test_cleanup_trailing_return() {
    let stmts = vec![
        Statement::Expr(Expr::Call(
            Some(Box::new(Expr::This)),
            "DoSomething".into(),
            vec![],
        )),
        Statement::Return(None),
    ];
    let result = apply_patterns(stmts, "", &LambdaMap::new());
    assert_eq!(result.len(), 1);
}

#[test]
fn test_double_negation_removal() {
    let expr = Expr::Unary(
        UnaryOp::LogicalNot,
        Box::new(Expr::Unary(
            UnaryOp::LogicalNot,
            Box::new(Expr::Local(0, "x".into())),
        )),
    );
    let result = simplify_expr(expr);
    match result {
        Expr::Local(0, ref name) if name == "x" => {}
        other => panic!("Expected Local(0, x), got {:?}", other),
    }
}

#[test]
fn test_eq_zero_boolean_simplification() {
    let expr = Expr::Binary(
        Box::new(Expr::Binary(
            Box::new(Expr::Local(0, "x".into())),
            BinOp::Gt,
            Box::new(Expr::Local(1, "y".into())),
        )),
        BinOp::Eq,
        Box::new(Expr::Int(0)),
    );
    let result = simplify_expr(expr);
    match result {
        Expr::Unary(UnaryOp::LogicalNot, _) => {}
        other => panic!("Expected LogicalNot, got {:?}", other),
    }
}

#[test]
fn test_ne_zero_boolean_simplification() {
    let expr = Expr::Binary(
        Box::new(Expr::Binary(
            Box::new(Expr::Local(0, "x".into())),
            BinOp::Gt,
            Box::new(Expr::Local(1, "y".into())),
        )),
        BinOp::Ne,
        Box::new(Expr::Int(0)),
    );
    let result = simplify_expr(expr);
    match result {
        Expr::Binary(_, BinOp::Gt, _) => {}
        other => panic!("Expected Gt comparison, got {:?}", other),
    }
}

#[test]
fn test_null_coalescing_detection() {
    let stmts = vec![Statement::If(
        Expr::Binary(
            Box::new(Expr::Local(0, "x".into())),
            BinOp::Ne,
            Box::new(Expr::Null),
        ),
        vec![Statement::Assign(
            Expr::Local(2, "result".into()),
            Expr::Local(0, "x".into()),
        )],
        Some(vec![Statement::Assign(
            Expr::Local(2, "result".into()),
            Expr::Local(1, "y".into()),
        )]),
    )];
    let result = apply_patterns(stmts, "", &LambdaMap::new());
    assert_eq!(result.len(), 1);
    match &result[0] {
        Statement::Assign(_, Expr::Binary(_, BinOp::NullCoalesce, _)) => {}
        other => panic!("Expected null coalescing assignment, got {:?}", other),
    }
}

#[test]
fn test_null_coalescing_reversed() {
    let stmts = vec![Statement::If(
        Expr::Binary(
            Box::new(Expr::Local(0, "x".into())),
            BinOp::Eq,
            Box::new(Expr::Null),
        ),
        vec![Statement::Assign(
            Expr::Local(2, "result".into()),
            Expr::Local(1, "y".into()),
        )],
        Some(vec![Statement::Assign(
            Expr::Local(2, "result".into()),
            Expr::Local(0, "x".into()),
        )]),
    )];
    let result = apply_patterns(stmts, "", &LambdaMap::new());
    assert_eq!(result.len(), 1);
    match &result[0] {
        Statement::Assign(_, Expr::Binary(_, BinOp::NullCoalesce, _)) => {}
        other => panic!("Expected null coalescing assignment, got {:?}", other),
    }
}

#[test]
fn test_empty_if_removal() {
    let stmts = vec![
        Statement::If(Expr::Bool(true), vec![], None),
        Statement::Return(Some(Expr::Int(42))),
    ];
    let result = apply_patterns(stmts, "", &LambdaMap::new());
    assert_eq!(result.len(), 1);
    match &result[0] {
        Statement::Return(Some(Expr::Int(42))) => {}
        other => panic!("Expected return 42, got {:?}", other),
    }
}

#[test]
fn test_foreach_detection() {
    let dll_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../DummyDll/Assembly-CSharp.dll"
    );
    if let Ok(data) = std::fs::read(dll_path) {
        let asm = crate::assembly::Assembly::parse(&data).unwrap();

        let mut found_foreach = false;
        for td in &asm.types {
            if let Ok(code) = crate::decompiler::decompile_type(&asm, td.token) {
                if code.contains("foreach") {
                    found_foreach = true;
                    println!(
                        "Found foreach in {}:\n{}",
                        td.name,
                        &code[..code.len().min(500)]
                    );
                    break;
                }
            }
        }
        if !found_foreach {
            println!("No foreach patterns found (expected for some DLLs)");
        }
    } else {
        println!("Assembly-CSharp.dll not found, skipping foreach detection test");
    }
}

#[test]
fn test_patterns_dont_break_existing() {
    let dll_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../DummyDll/System.dll");
    let data = std::fs::read(dll_path).unwrap();
    let asm = crate::assembly::Assembly::parse(&data).unwrap();

    let mut success = 0;
    for td in &asm.types {
        if crate::decompiler::decompile_type(&asm, td.token).is_ok() {
            success += 1;
        }
    }
    assert!(success > 0);
}

#[test]
fn test_using_detection_unit() {
    let stmts = vec![
        Statement::LocalDecl(
            "StreamReader".into(),
            "reader".into(),
            Some(Expr::NewObj(
                "StreamReader".into(),
                vec![Expr::Local(0, "path".into())],
            )),
        ),
        Statement::Try(
            vec![Statement::Expr(Expr::Call(
                Some(Box::new(Expr::Local(1, "reader".into()))),
                "ReadToEnd".into(),
                vec![],
            ))],
            vec![],
            Some(vec![Statement::Expr(Expr::Call(
                Some(Box::new(Expr::Local(1, "reader".into()))),
                "Dispose".into(),
                vec![],
            ))]),
        ),
    ];
    let result = apply_patterns(stmts, "", &LambdaMap::new());
    assert_eq!(result.len(), 1);
    match &result[0] {
        Statement::Using(_, _) => {}
        other => panic!("Expected Using statement, got {:?}", other),
    }
}

// --- Property accessor rewriting tests ---

#[test]
fn test_property_getter_rewrite() {
    let expr = Expr::Call(
        Some(Box::new(Expr::Local(0, "obj".into()))),
        "get_Name".into(),
        vec![],
    );
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::Field(obj, ref name) if name == "Name" => {
            assert!(matches!(*obj, Expr::Local(0, _)));
        }
        other => panic!("Expected Field access, got {:?}", other),
    }
}

#[test]
fn test_property_setter_rewrite() {
    let stmt = Statement::Expr(Expr::Call(
        Some(Box::new(Expr::This)),
        "set_isOn".into(),
        vec![Expr::Int(0)],
    ));
    let result = rewrite_prop_stmt(stmt);
    match result {
        Statement::Assign(Expr::Field(obj, ref name), Expr::Int(0)) if name == "isOn" => {
            assert!(matches!(*obj, Expr::This));
        }
        other => panic!("Expected Assign to Field, got {:?}", other),
    }
}

#[test]
fn test_static_property_getter_rewrite() {
    let expr = Expr::StaticCall("MyType".into(), "get_Instance".into(), vec![]);
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::StaticField(ref ty, ref name) if ty == "MyType" && name == "Instance" => {}
        other => panic!("Expected StaticField, got {:?}", other),
    }
}

#[test]
fn test_static_property_setter_rewrite() {
    let stmt = Statement::Expr(Expr::StaticCall(
        "Config".into(),
        "set_Value".into(),
        vec![Expr::Int(42)],
    ));
    let result = rewrite_prop_stmt(stmt);
    match result {
        Statement::Assign(Expr::StaticField(ref ty, ref name), Expr::Int(42))
            if ty == "Config" && name == "Value" => {}
        other => panic!("Expected Assign to StaticField, got {:?}", other),
    }
}

#[test]
fn test_op_equality_static_rewrite() {
    let expr = Expr::StaticCall(
        "String".into(),
        "op_Equality".into(),
        vec![Expr::Local(0, "a".into()), Expr::Local(1, "b".into())],
    );
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::Binary(_, BinOp::Eq, _) => {}
        other => panic!("Expected Binary Eq, got {:?}", other),
    }
}

#[test]
fn test_op_inequality_static_rewrite() {
    let expr = Expr::StaticCall(
        "String".into(),
        "op_Inequality".into(),
        vec![Expr::Local(0, "a".into()), Expr::Local(1, "b".into())],
    );
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::Binary(_, BinOp::Ne, _) => {}
        other => panic!("Expected Binary Ne, got {:?}", other),
    }
}

#[test]
fn test_op_equality_call_none_rewrite() {
    let expr = Expr::Call(
        None,
        "op_Equality".into(),
        vec![Expr::Local(0, "a".into()), Expr::Local(1, "b".into())],
    );
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::Binary(_, BinOp::Eq, _) => {}
        other => panic!("Expected Binary Eq, got {:?}", other),
    }
}

#[test]
fn test_getter_nested_in_binary() {
    let expr = Expr::Binary(
        Box::new(Expr::Call(
            Some(Box::new(Expr::Local(0, "list".into()))),
            "get_Count".into(),
            vec![],
        )),
        BinOp::Gt,
        Box::new(Expr::Int(0)),
    );
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::Binary(left, BinOp::Gt, _) => {
            assert!(matches!(*left, Expr::Field(_, ref n) if n == "Count"));
        }
        other => panic!("Expected Binary with Field, got {:?}", other),
    }
}

#[test]
fn test_getter_in_call_args() {
    let expr = Expr::Call(
        None,
        "foo".into(),
        vec![Expr::Call(
            Some(Box::new(Expr::Local(0, "obj".into()))),
            "get_Name".into(),
            vec![],
        )],
    );
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::Call(None, ref name, ref args) if name == "foo" => {
            assert!(matches!(&args[0], Expr::Field(_, ref n) if n == "Name"));
        }
        other => panic!("Expected Call with Field arg, got {:?}", other),
    }
}

#[test]
fn test_setter_in_if_block() {
    let stmts = vec![Statement::If(
        Expr::Bool(true),
        vec![Statement::Expr(Expr::Call(
            Some(Box::new(Expr::This)),
            "set_X".into(),
            vec![Expr::Int(1)],
        ))],
        None,
    )];
    let result = rewrite_property_accessors(stmts);
    match &result[0] {
        Statement::If(_, then_block, _) => match &then_block[0] {
            Statement::Assign(Expr::Field(_, ref name), Expr::Int(1)) if name == "X" => {}
            other => panic!("Expected Assign in if body, got {:?}", other),
        },
        other => panic!("Expected If, got {:?}", other),
    }
}

#[test]
fn test_non_property_call_unchanged() {
    let expr = Expr::Call(Some(Box::new(Expr::This)), "DoSomething".into(), vec![]);
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::Call(Some(_), ref name, _) if name == "DoSomething" => {}
        other => panic!("Expected unchanged Call, got {:?}", other),
    }
}

#[test]
fn test_get_with_args_unchanged() {
    let expr = Expr::Call(
        Some(Box::new(Expr::This)),
        "get_Item".into(),
        vec![Expr::Int(0)],
    );
    let result = rewrite_prop_expr(expr);
    match result {
        Expr::Call(Some(_), ref name, ref args) if name == "get_Item" && args.len() == 1 => {}
        other => panic!("Expected unchanged Call with args, got {:?}", other),
    }
}

// --- A1: Self-field prefix removal tests ---

#[test]
fn test_self_static_field_becomes_instance_field() {
    let stmts = vec![Statement::Expr(Expr::StaticField(
        "MyClass".into(),
        "m_field".into(),
    ))];
    let result = simplify_self_references(stmts, "MyClass");
    match &result[0] {
        Statement::Expr(Expr::Field(obj, ref name)) if name == "m_field" => {
            assert!(matches!(obj.as_ref(), Expr::This));
        }
        other => panic!("Expected Field(This, m_field), got {:?}", other),
    }
}

#[test]
fn test_self_static_call_becomes_instance_call() {
    let stmts = vec![Statement::Expr(Expr::StaticCall(
        "MyClass".into(),
        "DoThing".into(),
        vec![Expr::Int(1)],
    ))];
    let result = simplify_self_references(stmts, "MyClass");
    match &result[0] {
        Statement::Expr(Expr::Call(Some(obj), ref name, ref args))
            if name == "DoThing" && args.len() == 1 =>
        {
            assert!(matches!(obj.as_ref(), Expr::This));
        }
        other => panic!("Expected Call(This, DoThing), got {:?}", other),
    }
}

#[test]
fn test_different_type_static_field_unchanged() {
    let stmts = vec![Statement::Expr(Expr::StaticField(
        "OtherClass".into(),
        "m_field".into(),
    ))];
    let result = simplify_self_references(stmts, "MyClass");
    match &result[0] {
        Statement::Expr(Expr::StaticField(ref ty, ref name))
            if ty == "OtherClass" && name == "m_field" => {}
        other => panic!("Expected unchanged StaticField, got {:?}", other),
    }
}

#[test]
fn test_self_reference_empty_enclosing_type_noop() {
    let stmts = vec![Statement::Expr(Expr::StaticField(
        "MyClass".into(),
        "field".into(),
    ))];
    let result = simplify_self_references(stmts, "");
    match &result[0] {
        Statement::Expr(Expr::StaticField(..)) => {}
        other => panic!("Expected unchanged StaticField, got {:?}", other),
    }
}

// --- A3: Boolean literal coercion tests ---

#[test]
fn test_bool_coerce_in_if_condition() {
    let stmts = vec![
        Statement::If(Expr::Int(0), vec![Statement::Break], None),
        Statement::If(Expr::Int(1), vec![Statement::Break], None),
    ];
    let result = coerce_booleans(stmts);
    match &result[0] {
        Statement::If(Expr::Bool(false), _, _) => {}
        other => panic!("Expected if(false), got {:?}", other),
    }
    match &result[1] {
        Statement::If(Expr::Bool(true), _, _) => {}
        other => panic!("Expected if(true), got {:?}", other),
    }
}

#[test]
fn test_bool_coerce_in_while_condition() {
    let stmts = vec![Statement::While(Expr::Int(1), vec![])];
    let result = coerce_booleans(stmts);
    match &result[0] {
        Statement::While(Expr::Bool(true), _) => {}
        other => panic!("Expected while(true), got {:?}", other),
    }
}

#[test]
fn test_bool_coerce_boolean_field_assignment() {
    let stmts = vec![Statement::Assign(
        Expr::Field(Box::new(Expr::This), "isEnabled".into()),
        Expr::Int(0),
    )];
    let result = coerce_booleans(stmts);
    match &result[0] {
        Statement::Assign(Expr::Field(_, ref name), Expr::Bool(false)) if name == "isEnabled" => {}
        other => panic!("Expected Assign(isEnabled, false), got {:?}", other),
    }
}

#[test]
fn test_bool_coerce_non_boolean_field_unchanged() {
    let stmts = vec![Statement::Assign(
        Expr::Field(Box::new(Expr::This), "count".into()),
        Expr::Int(0),
    )];
    let result = coerce_booleans(stmts);
    match &result[0] {
        Statement::Assign(Expr::Field(_, ref name), Expr::Int(0)) if name == "count" => {}
        other => panic!("Expected Assign(count, 0), got {:?}", other),
    }
}

#[test]
fn test_bool_coerce_m_allow_prefix() {
    let stmts = vec![Statement::Assign(
        Expr::Field(Box::new(Expr::This), "m_AllowSwitchOff".into()),
        Expr::Int(1),
    )];
    let result = coerce_booleans(stmts);
    match &result[0] {
        Statement::Assign(_, Expr::Bool(true)) => {}
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn test_bool_coerce_do_while() {
    let stmts = vec![Statement::DoWhile(vec![], Expr::Int(0))];
    let result = coerce_booleans(stmts);
    match &result[0] {
        Statement::DoWhile(_, Expr::Bool(false)) => {}
        other => panic!("Expected DoWhile with Bool(false), got {:?}", other),
    }
}

// -----------------------------------------------------------------------
// For-loop reconstruction tests
// -----------------------------------------------------------------------

#[test]
fn test_for_loop_basic_assign_init() {
    let stmts = vec![
        Statement::Assign(Expr::Local(0, "i".into()), Expr::Int(0)),
        Statement::While(
            Expr::Binary(
                Box::new(Expr::Local(0, "i".into())),
                BinOp::Lt,
                Box::new(Expr::Field(Box::new(Expr::This), "Count".into())),
            ),
            vec![
                Statement::Expr(Expr::Raw("body".into())),
                Statement::Assign(
                    Expr::Local(0, "i".into()),
                    Expr::Binary(
                        Box::new(Expr::Local(0, "i".into())),
                        BinOp::Add,
                        Box::new(Expr::Int(1)),
                    ),
                ),
            ],
        ),
    ];
    let result = detect_for_loops(stmts);
    assert_eq!(result.len(), 1);
    match &result[0] {
        Statement::For(init, cond, update, body) => {
            match init.as_ref() {
                Statement::Assign(Expr::Local(0, _), Expr::Int(0)) => {}
                other => panic!("Expected Assign init, got {:?}", other),
            }
            assert!(
                expr_contains_local_idx(cond, 0),
                "Condition should reference local 0"
            );
            match update.as_ref() {
                Statement::Assign(Expr::Local(0, _), Expr::Binary(_, BinOp::Add, _)) => {}
                other => panic!("Expected increment update, got {:?}", other),
            }
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected For loop, got {:?}", other),
    }
}

#[test]
fn test_for_loop_decrement() {
    let stmts = vec![
        Statement::Assign(Expr::Local(0, "i".into()), Expr::Int(10)),
        Statement::While(
            Expr::Binary(
                Box::new(Expr::Local(0, "i".into())),
                BinOp::Gt,
                Box::new(Expr::Int(0)),
            ),
            vec![
                Statement::Expr(Expr::Raw("body".into())),
                Statement::Assign(
                    Expr::Local(0, "i".into()),
                    Expr::Binary(
                        Box::new(Expr::Local(0, "i".into())),
                        BinOp::Sub,
                        Box::new(Expr::Int(1)),
                    ),
                ),
            ],
        ),
    ];
    let result = detect_for_loops(stmts);
    assert_eq!(result.len(), 1);
    match &result[0] {
        Statement::For(_, _, update, _) => match update.as_ref() {
            Statement::Assign(_, Expr::Binary(_, BinOp::Sub, _)) => {}
            other => panic!("Expected decrement update, got {:?}", other),
        },
        other => panic!("Expected For loop, got {:?}", other),
    }
}

#[test]
fn test_for_loop_local_decl_init() {
    let stmts = vec![
        Statement::LocalDecl("int".into(), "i".into(), Some(Expr::Int(0))),
        Statement::While(
            Expr::Binary(
                Box::new(Expr::Local(0, "i".into())),
                BinOp::Lt,
                Box::new(Expr::Local(1, "n".into())),
            ),
            vec![
                Statement::Expr(Expr::Raw("body".into())),
                Statement::Assign(
                    Expr::Local(0, "i".into()),
                    Expr::Binary(
                        Box::new(Expr::Local(0, "i".into())),
                        BinOp::Add,
                        Box::new(Expr::Int(1)),
                    ),
                ),
            ],
        ),
    ];
    let result = detect_for_loops(stmts);
    assert_eq!(result.len(), 1);
    match &result[0] {
        Statement::For(init, _, _, body) => {
            match init.as_ref() {
                Statement::LocalDecl(ty, name, Some(Expr::Int(0)))
                    if ty == "int" && name == "i" => {}
                other => panic!("Expected LocalDecl init, got {:?}", other),
            }
            assert_eq!(body.len(), 1, "Body should exclude the increment");
        }
        other => panic!("Expected For loop, got {:?}", other),
    }
}

#[test]
fn test_for_loop_no_match_without_increment() {
    let stmts = vec![
        Statement::Assign(Expr::Local(0, "i".into()), Expr::Int(0)),
        Statement::While(
            Expr::Binary(
                Box::new(Expr::Local(0, "i".into())),
                BinOp::Lt,
                Box::new(Expr::Int(10)),
            ),
            vec![Statement::Expr(Expr::Raw("body".into()))],
        ),
    ];
    let result = detect_for_loops(stmts);
    assert_eq!(result.len(), 2, "Should remain as two separate statements");
    assert!(matches!(&result[0], Statement::Assign(..)));
    assert!(matches!(&result[1], Statement::While(..)));
}

#[test]
fn test_for_loop_nested_in_if() {
    let stmts = vec![Statement::If(
        Expr::Bool(true),
        vec![
            Statement::Assign(Expr::Local(0, "i".into()), Expr::Int(0)),
            Statement::While(
                Expr::Binary(
                    Box::new(Expr::Local(0, "i".into())),
                    BinOp::Lt,
                    Box::new(Expr::Int(10)),
                ),
                vec![
                    Statement::Expr(Expr::Raw("body".into())),
                    Statement::Assign(
                        Expr::Local(0, "i".into()),
                        Expr::Binary(
                            Box::new(Expr::Local(0, "i".into())),
                            BinOp::Add,
                            Box::new(Expr::Int(1)),
                        ),
                    ),
                ],
            ),
        ],
        None,
    )];
    let result = detect_for_loops(stmts);
    assert_eq!(result.len(), 1);
    match &result[0] {
        Statement::If(_, then_body, _) => {
            assert_eq!(then_body.len(), 1);
            assert!(
                matches!(&then_body[0], Statement::For(..)),
                "Expected For in if-body, got {:?}",
                &then_body[0]
            );
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

#[test]
fn test_for_loop_condition_not_referencing_var() {
    let stmts = vec![
        Statement::Assign(Expr::Local(0, "i".into()), Expr::Int(0)),
        Statement::While(
            Expr::Bool(true),
            vec![
                Statement::Expr(Expr::Raw("body".into())),
                Statement::Assign(
                    Expr::Local(0, "i".into()),
                    Expr::Binary(
                        Box::new(Expr::Local(0, "i".into())),
                        BinOp::Add,
                        Box::new(Expr::Int(1)),
                    ),
                ),
            ],
        ),
    ];
    let result = detect_for_loops(stmts);
    assert_eq!(
        result.len(),
        2,
        "Should not convert when cond doesn't ref var"
    );
}

// --- B6: Lambda decompilation tests ---

#[test]
fn test_lambda_inline_newobj_with_func_ref() {
    let mut lm = LambdaMap::new();
    lm.insert(
        "<AnyTogglesOn>b__10_0".into(),
        (
            vec![("ActiveToggle".into(), "x".into())],
            vec![Statement::Return(Some(Expr::Field(
                Box::new(Expr::Arg(0, "x".into())),
                "isOn".into(),
            )))],
        ),
    );

    let new_obj = Expr::NewObj(
        "Predicate<ActiveToggle>".into(),
        vec![
            Expr::StaticField("<>c".into(), "<>9".into()),
            Expr::Raw("&<>c::<AnyTogglesOn>b__10_0".into()),
        ],
    );

    let result = inline_lambda_expr(new_obj, &lm);
    let emitted = emit_expr(&result);
    assert_eq!(emitted, "(ActiveToggle x) => x.isOn");
}

#[test]
fn test_lambda_propagate_delegate_assignment() {
    use crate::decompiler::emit::emit_statements;

    let lambda = Expr::Lambda(
        vec![("ActiveToggle".into(), "x".into())],
        Box::new(LambdaBody::Expr(Expr::Field(
            Box::new(Expr::Arg(0, "x".into())),
            "isOn".into(),
        ))),
    );

    let stmts = vec![
        Statement::Assign(Expr::StaticField("<>c".into(), "<>9__10_0".into()), lambda),
        Statement::Return(Some(Expr::Binary(
            Box::new(Expr::Call(
                Some(Box::new(Expr::Field(
                    Box::new(Expr::This),
                    "m_Toggles".into(),
                ))),
                "Find".into(),
                vec![Expr::StaticField("<>c".into(), "<>9__10_0".into())],
            )),
            BinOp::Ne,
            Box::new(Expr::Null),
        ))),
    ];

    let result = propagate_delegate_assignments(stmts);
    assert_eq!(result.len(), 1, "Assignment should be eliminated");

    let emitted = emit_statements(&result, 0);
    assert!(
        emitted.contains("(ActiveToggle x) => x.isOn"),
        "Lambda should be inlined: {}",
        emitted
    );
    assert!(
        !emitted.contains("<>c"),
        "Compiler-generated names should be gone: {}",
        emitted
    );
}

#[test]
fn test_lambda_emit_expression_bodied() {
    let lambda = Expr::Lambda(
        vec![("int".into(), "x".into()), ("int".into(), "y".into())],
        Box::new(LambdaBody::Expr(Expr::Binary(
            Box::new(Expr::Arg(0, "x".into())),
            BinOp::Add,
            Box::new(Expr::Arg(1, "y".into())),
        ))),
    );
    assert_eq!(emit_expr(&lambda), "(int x, int y) => x + y");
}

#[test]
fn test_lambda_emit_block_bodied() {
    let lambda = Expr::Lambda(
        vec![("string".into(), "s".into())],
        Box::new(LambdaBody::Block(vec![
            Statement::Expr(Expr::Call(
                None,
                "Console.WriteLine".into(),
                vec![Expr::Arg(0, "s".into())],
            )),
            Statement::Return(Some(Expr::Bool(true))),
        ])),
    );
    let emitted = emit_expr(&lambda);
    assert!(emitted.starts_with("(string s) => {\n"));
    assert!(emitted.contains("Console.WriteLine(s)"));
    assert!(emitted.contains("return true;"));
}

#[test]
fn test_extract_method_name_from_ref() {
    assert_eq!(
        extract_method_name_from_ref("&<>c::<AnyTogglesOn>b__10_0"),
        Some("<AnyTogglesOn>b__10_0")
    );
    assert_eq!(
        extract_method_name_from_ref("&SomeClass::Method"),
        Some("Method")
    );
    assert_eq!(extract_method_name_from_ref("noAmpersand"), None);
}

#[test]
fn test_is_delegate_type() {
    assert!(is_delegate_type("Predicate<ActiveToggle>"));
    assert!(is_delegate_type("Func<int, bool>"));
    assert!(is_delegate_type("Action<string>"));
    assert!(is_delegate_type("Comparison<int>"));
    assert!(is_delegate_type("UnityAction"));
    assert!(!is_delegate_type("List<int>"));
    assert!(!is_delegate_type("Dictionary<string, int>"));
}

#[test]
fn test_array_initializer_detection() {
    let stmts = vec![
        Statement::Assign(
            Expr::ArrayElement(
                Box::new(Expr::ArrayNew("Object".into(), Box::new(Expr::Int(2)))),
                Box::new(Expr::Int(0)),
            ),
            Expr::Arg(1, "toggle".into()),
        ),
        Statement::Assign(
            Expr::ArrayElement(
                Box::new(Expr::ArrayNew("Object".into(), Box::new(Expr::Int(2)))),
                Box::new(Expr::Int(1)),
            ),
            Expr::This,
        ),
        Statement::Throw(Some(Expr::NewObj(
            "ArgumentException".into(),
            vec![Expr::StaticCall(
                "String".into(),
                "Format".into(),
                vec![
                    Expr::String("Toggle {0} is not part of ToggleGroup {1}".into()),
                    Expr::ArrayNew("Object".into(), Box::new(Expr::Int(2))),
                ],
            )],
        ))),
    ];

    let result = apply_patterns(stmts, "", &LambdaMap::new());
    assert_eq!(
        result.len(),
        1,
        "Expected 1 statement, got {}: {:?}",
        result.len(),
        result
    );
    match &result[0] {
        Statement::Throw(Some(Expr::NewObj(_, args))) => match &args[0] {
            Expr::StaticCall(_, _, format_args) => {
                let arr = &format_args[1];
                let s = emit_expr(arr);
                assert!(
                    s.contains("new Object[]"),
                    "Expected array init, got: {}",
                    s
                );
                assert!(
                    s.contains("toggle"),
                    "Expected toggle in array init, got: {}",
                    s
                );
                assert!(
                    s.contains("this"),
                    "Expected this in array init, got: {}",
                    s
                );
            }
            other => panic!("Expected StaticCall, got: {:?}", other),
        },
        other => panic!("Expected Throw(NewObj(...)), got: {:?}", other),
    }
}
