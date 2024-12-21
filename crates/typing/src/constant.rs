use mago_ast::Identifier;
use mago_interner::ThreadedInterner;
use mago_names::Names;
use mago_reflection::r#type::kind::*;
use mago_reflection::CodebaseReflection;
use mago_trinary::Trinary;
use ordered_float::OrderedFloat;

pub struct ConstantTypeResolver<'i, 'c> {
    interner: &'i ThreadedInterner,
    names: &'c Names,
    codebase: Option<&'c CodebaseReflection>,
}

impl<'i, 'c> ConstantTypeResolver<'i, 'c> {
    pub fn new(interner: &'i ThreadedInterner, names: &'c Names, codebase: Option<&'c CodebaseReflection>) -> Self {
        Self { interner, names, codebase }
    }

    pub fn resolve(&self, constant: &Identifier) -> TypeKind {
        let (short_name, full_name) = if self.names.is_imported(constant) {
            let name = self.interner.lookup(self.names.get(constant));

            (name, name)
        } else {
            let short_name = self.interner.lookup(&constant.value());
            let imported_name = self.interner.lookup(self.names.get(constant));

            if let Some(stripped) = short_name.strip_prefix('\\') {
                (stripped, imported_name)
            } else {
                (short_name, imported_name)
            }
        };

        match short_name {
            "PHP_VERSION" => non_empty_string_kind(),
            "PHP_MAJOR_VERSION" => integer_range_kind(7, 9),
            "PHP_MINOR_VERSION" => non_negative_integer_kind(),
            "PHP_RELEASE_VERSION" => non_negative_integer_kind(),
            "PHP_VERSION_ID" => integer_range_kind(70000, 99999),
            "PHP_ZTS" => union_kind(vec![value_integer_kind(0), value_integer_kind(1)]),
            "PHP_DEBUG" => union_kind(vec![value_integer_kind(0), value_integer_kind(1)]),
            "PHP_MAXPATHLEN" => positive_integer_kind(),
            "PHP_OS" => non_empty_string_kind(),
            "PHP_OS_FAMILY" => union_kind(vec![
                value_string_kind(
                    self.interner.intern("Windows"),
                    7,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
                value_string_kind(
                    self.interner.intern("BSD"),
                    3,
                    Trinary::True,
                    Trinary::True,
                    Trinary::False,
                    Trinary::False,
                ),
                value_string_kind(
                    self.interner.intern("Darwin"),
                    6,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
                value_string_kind(
                    self.interner.intern("Linux"),
                    5,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
                value_string_kind(
                    self.interner.intern("Solaris"),
                    7,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
                value_string_kind(
                    self.interner.intern("Unknown"),
                    7,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
            ]),
            "PHP_SAPI" => union_kind(vec![
                value_string_kind(
                    self.interner.intern("apache"),
                    6,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("apache2handler"),
                    14,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("cgi"),
                    3,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("cli"),
                    3,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("cli-server"),
                    10,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("embed"),
                    5,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("fpm-fcgi"),
                    8,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("litespeed"),
                    9,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("phpdbg"),
                    6,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                non_empty_string_kind(),
            ]),
            "PHP_EOL" => union_kind(vec![
                value_string_kind(
                    self.interner.intern("\n"),
                    1,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
                value_string_kind(
                    self.interner.intern("\r\n"),
                    2,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
            ]),
            "PHP_INT_MAX" => union_kind(vec![value_integer_kind(9223372036854775807), value_integer_kind(2147483647)]),
            "PHP_INT_MIN" => {
                union_kind(vec![value_integer_kind(-9223372036854775808), value_integer_kind(-2147483648)])
            }
            "PHP_INT_SIZE" => union_kind(vec![value_integer_kind(4), value_integer_kind(8)]),
            "PHP_FLOAT_DIG" => positive_integer_kind(),
            "PHP_FLOAT_EPSILON" => union_kind(vec![
                value_float_kind(OrderedFloat(2.220_446_049_250_313e-16)),
                value_float_kind(OrderedFloat(1.19209290e-7)),
            ]),
            "PHP_EXTENSION_DIR" => non_empty_string_kind(),
            "PHP_PREFIX" => non_empty_string_kind(),
            "PHP_BINDIR" => non_empty_string_kind(),
            "PHP_BINARY" => non_empty_string_kind(),
            "PHP_MANDIR" => non_empty_string_kind(),
            "PHP_LIBDIR" => non_empty_string_kind(),
            "PHP_DATADIR" => non_empty_string_kind(),
            "PHP_SYSCONFDIR" => non_empty_string_kind(),
            "PHP_LOCALSTATEDIR" => non_empty_string_kind(),
            "PHP_CONFIG_FILE_PATH" => non_empty_string_kind(),
            "PHP_SHLIB_SUFFIX" => union_kind(vec![
                value_string_kind(
                    self.interner.intern("so"),
                    2,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
                value_string_kind(
                    self.interner.intern("dll"),
                    3,
                    Trinary::False,
                    Trinary::False,
                    Trinary::True,
                    Trinary::True,
                ),
            ]),
            "PHP_FD_SETSIZE" => positive_integer_kind(),
            "PHP_WINDOWS_VERSION_MAJOR" => union_kind(vec![
                value_integer_kind(4), // NT4/Me/98/95
                value_integer_kind(5), // XP/2003 R2/2003/2000
                value_integer_kind(6), // Vista/2008/7/8/8.1
            ]),
            "PHP_WINDOWS_VERSION_MINOR" => union_kind(vec![
                value_integer_kind(0),  // Vista/2008/2000/NT4/95
                value_integer_kind(1),  // XP
                value_integer_kind(2),  // 2003 R2/2003/XP x64
                value_integer_kind(10), // 98
                value_integer_kind(90), // Me
            ]),
            "PHP_WINDOWS_VERSION_BUILD" => positive_integer_kind(),
            "DIRECTORY_SEPARATOR" => union_kind(vec![
                value_string_kind(
                    self.interner.intern("/"),
                    1,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
                value_string_kind(
                    self.interner.intern("\\"),
                    1,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
            ]),
            "PATH_SEPARATOR" => union_kind(vec![
                value_string_kind(
                    self.interner.intern(";"),
                    1,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
                value_string_kind(
                    self.interner.intern(":"),
                    1,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                    Trinary::False,
                ),
            ]),
            "ICONV_IMPL" => non_empty_string_kind(),
            "LIBXML_VERSION" => positive_integer_kind(),
            "LIBXML_DOTTED_VERSION" => non_empty_string_kind(),
            "OPENSSL_VERSION_NUMBER" => positive_integer_kind(),
            "PCRE_VERSION" => non_empty_string_kind(),
            "STDIN" | "STDOUT" | "STDERR" => resource_kind(),
            "NAN" => value_float_kind(OrderedFloat(f64::NAN)),
            "INF" => value_float_kind(OrderedFloat(f64::INFINITY)),
            _ => {
                if let Some(codebase) = self.codebase {
                    let short_name_id = self.interner.intern(short_name);
                    let full_name_id = self.interner.intern(full_name);

                    let Some(constant) =
                        codebase.get_constant(&full_name_id).or_else(|| codebase.get_constant(&short_name_id))
                    else {
                        return mixed_kind(false);
                    };

                    constant.type_reflection.kind.clone()
                } else {
                    mixed_kind(false)
                }
            }
        }
    }
}
