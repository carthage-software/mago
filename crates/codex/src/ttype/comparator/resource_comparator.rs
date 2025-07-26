use crate::ttype::atomic::TAtomic;

pub(crate) fn is_contained_by(input_type_part: &TAtomic, container_type_part: &TAtomic) -> bool {
    let TAtomic::Resource(container_resource) = container_type_part else {
        return false;
    };

    let TAtomic::Resource(input_resource) = input_type_part else {
        return false;
    };

    let Some(is_closed) = container_resource.closed else {
        return true; // all resources are accepted if the container is not closed/open
    };

    if is_closed { input_resource.is_closed() } else { input_resource.is_open() }
}
