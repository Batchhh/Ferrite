//! IL visibility flag helpers.

pub fn il_type_visibility(flags: u32) -> &'static str {
    match flags & 0x07 {
        0x00 => "",
        0x01 => "public ",
        0x02 => "nested public ",
        0x03 => "nested private ",
        0x04 => "nested family ",
        0x05 => "nested assembly ",
        0x06 => "nested famorassem ",
        0x07 => "nested famandassem ",
        _ => "",
    }
}

pub fn il_method_visibility(flags: u16) -> &'static str {
    match flags & 0x0007 {
        0x0001 => "private ",
        0x0002 => "famandassem ",
        0x0003 => "assembly ",
        0x0004 => "family ",
        0x0005 => "famorassem ",
        0x0006 => "public ",
        _ => "privatescope ",
    }
}

pub fn il_field_visibility(flags: u16) -> &'static str {
    il_method_visibility(flags)
}
