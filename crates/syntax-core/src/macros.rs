#[macro_export]
macro_rules! start_of_identifier {
    () => {
        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'\x80'..=b'\xff'
    };
}

#[macro_export]
macro_rules! part_of_identifier {
    () => {
        b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'\x80'..=b'\xff'
    };
}

#[macro_export]
macro_rules! start_of_number {
    () => {
        b'0'..=b'9'
    };
}

#[macro_export]
macro_rules! start_of_binary_number {
    () => {
        [b'0', b'B' | b'b', b'0' | b'1']
    };
}

#[macro_export]
macro_rules! start_of_octal_number {
    () => {
        [b'0', b'O' | b'o', b'0'..=b'7']
    };
}

#[macro_export]
macro_rules! start_of_hexadecimal_number {
    () => {
        [b'0', b'X' | b'x', b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F']
    };
}

#[macro_export]
macro_rules! start_of_octal_or_float_number {
    () => {
        [b'0', ..]
    };
}

#[macro_export]
macro_rules! start_of_float_number {
    () => {
        [b'.', ..]
    };
}

#[macro_export]
macro_rules! float_exponent {
    () => {
        [b'e' | b'E']
    };
}

#[macro_export]
macro_rules! float_separator {
    () => {
        [b'.', ..] | [b'e' | b'E', b'-' | b'+', b'0'..=b'9'] | [b'e' | b'E', b'0'..=b'9', ..]
    };
}

#[macro_export]
macro_rules! number_sign {
    () => {
        [b'-' | b'+']
    };
}

#[macro_export]
macro_rules! number_separator {
    () => {
        b'_'
    };
}
