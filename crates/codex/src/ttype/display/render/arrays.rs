use std::io::Write;

use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;

use super::Renderer;

impl Renderer<'_> {
    pub(super) fn write_array(&self, array: &TArray, output: &mut Vec<u8>) {
        match array {
            TArray::List(list) => {
                if let Some(elements) = &list.known_elements {
                    output.extend_from_slice(b"list{");
                    let include_index = list.has_known_optional_elements()
                        || elements.first_key_value().is_some_and(|(index, _)| *index != 0);
                    for (index, (key, (optional, ty))) in elements.iter().enumerate() {
                        if index > 0 {
                            output.extend_from_slice(b", ");
                        }

                        if include_index {
                            let _ = write!(output, "{key}");
                            if *optional {
                                output.push(b'?');
                            }

                            output.extend_from_slice(b": ");
                        }

                        self.write_union(ty, output);
                    }

                    if !list.element_type.is_never() {
                        if !elements.is_empty() {
                            output.extend_from_slice(b", ");
                        }

                        output.extend_from_slice(b"...<");
                        self.write_union(&list.element_type, output);
                        output.push(b'>');
                    }

                    output.push(b'}');
                } else {
                    output.extend_from_slice(if list.is_non_empty() { b"non-empty-list<" } else { b"list<" });
                    self.write_union(&list.element_type, output);
                    output.push(b'>');
                }
            }
            TArray::Keyed(array) => {
                if let Some(items) = &array.known_items {
                    output.extend_from_slice(b"array{");
                    for (index, (key, (optional, ty))) in items.iter().enumerate() {
                        if index > 0 {
                            output.extend_from_slice(b", ");
                        }

                        write_key(key, output);
                        if *optional {
                            output.push(b'?');
                        }

                        output.extend_from_slice(b": ");
                        self.write_union(ty, output);
                    }

                    if let Some((key, value)) = &array.parameters {
                        if !items.is_empty() {
                            output.extend_from_slice(b", ");
                        }

                        output.extend_from_slice(b"...");
                        if !key.is_array_key() || !value.is_mixed() {
                            output.push(b'<');
                            self.write_types([key.as_ref(), value.as_ref()].into_iter(), output);
                            output.push(b'>');
                        }
                    }

                    output.push(b'}');
                } else if let Some((key, value)) = &array.parameters {
                    output.extend_from_slice(if array.is_non_empty() { b"non-empty-array<" } else { b"array<" });
                    self.write_types([key.as_ref(), value.as_ref()].into_iter(), output);
                    output.push(b'>');
                } else {
                    output.extend_from_slice(b"array{}");
                }
            }
        }
    }
}

fn write_key(key: &ArrayKey, output: &mut Vec<u8>) {
    match key {
        ArrayKey::Integer(value) => {
            let _ = write!(output, "{value}");
        }
        ArrayKey::String(value) => {
            output.push(b'\'');
            output.extend_from_slice(value.as_bytes());
            output.push(b'\'');
        }
        ArrayKey::ClassLikeConstant { class_like_name, constant_name } => {
            output.extend_from_slice(class_like_name.as_bytes());
            output.extend_from_slice(b"::");
            output.extend_from_slice(constant_name.as_bytes());
        }
    }
}
