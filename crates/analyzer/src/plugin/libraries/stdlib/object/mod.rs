mod get_called_class;
mod get_object_vars;

pub use get_called_class::GetCalledClassProvider;
pub use get_object_vars::GetObjectVarsProvider;
pub(crate) use get_object_vars::record_property_reads as record_get_object_vars_property_reads;
