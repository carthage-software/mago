use crate::cst::tag::TagVendor;

pub(crate) fn remainder_after_vendor(name: &[u8], vendor: Option<TagVendor>) -> &[u8] {
    match vendor {
        Some(vendor) => name.get(vendor.prefix().len() + 1..).unwrap_or(name),
        None => name,
    }
}
