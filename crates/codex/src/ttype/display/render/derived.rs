use crate::ttype::TType;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::union::TUnion;

use super::Renderer;

impl Renderer<'_> {
    pub(super) fn write_derived(&self, derived: &TDerived, output: &mut Vec<u8>) {
        match derived {
            TDerived::KeyOf(ty) => self.write_utility(b"key-of", ty.get_target_type(), output),
            TDerived::ValueOf(ty) => self.write_utility(b"value-of", ty.get_target_type(), output),
            TDerived::IntMaskOf(ty) => self.write_utility(b"int-mask-of", ty.get_target_type(), output),
            TDerived::New(ty) => self.write_utility(b"new", ty.get_target_type(), output),
            TDerived::PropertiesOf(ty) => {
                if let Some(visibility) = ty.visibility() {
                    output.extend_from_slice(visibility.as_bytes());
                    output.push(b'-');
                }

                self.write_utility(b"properties-of", ty.get_target_type(), output);
            }
            TDerived::IntMask(ty) => {
                output.extend_from_slice(b"int-mask<");
                self.write_types(ty.get_values().iter(), output);
                output.push(b'>');
            }
            TDerived::IndexAccess(ty) => {
                self.write_union(ty.get_target_type(), output);
                output.push(b'[');
                self.write_union(ty.get_index_type(), output);
                output.push(b']');
            }
            TDerived::TemplateType(ty) => {
                output.extend_from_slice(b"template-type<");
                self.write_types([ty.get_object(), ty.get_class_name(), ty.get_template_name()].into_iter(), output);
                output.push(b'>');
            }
            TDerived::Intersection(ty) => {
                output.push(b'(');
                self.write_union(ty.get_base_type(), output);
                output.push(b')');
                self.write_intersections(ty.get_intersection_types(), false, output);
            }
        }
    }

    fn write_utility(&self, name: &[u8], target: &TUnion, output: &mut Vec<u8>) {
        output.extend_from_slice(name);
        output.push(b'<');
        self.write_union(target, output);
        output.push(b'>');
    }
}
