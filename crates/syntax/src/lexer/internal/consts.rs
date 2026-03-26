use crate::token::TokenKind;

pub const CAST_TYPES: [(&[u8], TokenKind); 13] = [
    (b"(int)", TokenKind::IntCast),
    (b"(integer)", TokenKind::IntegerCast),
    (b"(double)", TokenKind::DoubleCast),
    (b"(float)", TokenKind::FloatCast),
    (b"(real)", TokenKind::RealCast),
    (b"(bool)", TokenKind::BoolCast),
    (b"(boolean)", TokenKind::BooleanCast),
    (b"(string)", TokenKind::StringCast),
    (b"(binary)", TokenKind::BinaryCast),
    (b"(array)", TokenKind::ArrayCast),
    (b"(object)", TokenKind::ObjectCast),
    (b"(unset)", TokenKind::UnsetCast),
    (b"(void)", TokenKind::VoidCast),
];
