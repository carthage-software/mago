use std::io::Write;

use crate::ttype::TType;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::float::TFloat;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TStringLiteral;

pub(super) fn write_scalar(scalar: &TScalar, output: &mut Vec<u8>) {
    match scalar {
        TScalar::String(string) => {
            if let Some(TStringLiteral::Value(value)) = string.literal {
                output.extend_from_slice(b"string('");
                output.extend_from_slice(value.as_bytes());
                output.extend_from_slice(b"')");
            } else {
                output.extend_from_slice(string.get_id().as_bytes());
            }
        }
        TScalar::Integer(integer) => match integer {
            TInteger::Literal(value) => {
                let _ = write!(output, "int({value})");
            }
            TInteger::From(value) if !matches!(*value, 0 | 1) => {
                let _ = write!(output, "int<{value}, max>");
            }
            TInteger::To(value) if !matches!(*value, -1 | 0) => {
                let _ = write!(output, "int<min, {value}>");
            }
            TInteger::Range(from, to) => {
                let _ = write!(output, "int<{from}, {to}>");
            }
            _ => output.extend_from_slice(integer.get_id().as_bytes()),
        },
        TScalar::Float(TFloat::Literal(value)) => {
            output.extend_from_slice(b"float(");
            output.extend_from_slice(ryu::Buffer::new().format(**value).as_bytes());
            output.push(b')');
        }
        _ => output.extend_from_slice(scalar.get_id().as_bytes()),
    }
}
