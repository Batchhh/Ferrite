use super::*;

#[test]
fn test_custom_attributes_extracted() {
    let dll_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../com.muf.runtime_1.dll");
    if let Ok(data) = std::fs::read(dll_path) {
        let asm = Assembly::parse(&data).unwrap();
        let mut type_attrs = 0;
        let mut field_attrs = 0;
        let mut method_attrs = 0;
        for td in &asm.types {
            type_attrs += td.custom_attributes.len();
            for f in &td.fields {
                field_attrs += f.custom_attributes.len();
            }
            for m in &td.methods {
                method_attrs += m.custom_attributes.len();
            }
        }
        println!(
            "Attributes — type: {}, field: {}, method: {}",
            type_attrs, field_attrs, method_attrs
        );
        assert!(type_attrs > 0, "Should have type-level attributes");
    } else {
        println!("User DLL not found, skipping test");
    }
}
