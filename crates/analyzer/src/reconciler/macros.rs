#[macro_export]
macro_rules! intersect_simple {
    (
        $(|)? $( $subtype_pattern:pat_param )|+ $( if $subtype_guard: expr )? $(,)?,
        $(|)? $( $supertype_pattern:pat_param )|+ $( if $supertype_guard: expr )? $(,)?,
        $context:expr,
        $max_type:expr,
        $assertion:expr,
        $existing_var_type:expr,
        $key:expr,
        $negated:expr,
        $span:expr,
        $is_equality:expr,
    ) => {
        {
            let mut acceptable_types = Vec::new();
            let mut did_remove_type = false;

            for atomic in $existing_var_type.types.as_ref() {
                if matches!(atomic, $( $subtype_pattern )|+ $( if $subtype_guard )?) {
                    acceptable_types.push(atomic.clone());
                } else if matches!(atomic, $( $supertype_pattern )|+ $( if $supertype_guard )?) {
                    return Some($max_type);
                } else if let TAtomic::Variable(_) = atomic {
                    did_remove_type = true;
                    acceptable_types.push(atomic.clone());
                } else {
                    did_remove_type = true;
                }
            }

            if acceptable_types.is_empty() || (!did_remove_type && !$is_equality) {
                if let Some(k) = $key {
                    if let Some(span) = $span {
                        trigger_issue_for_impossible(
                            $context,
                            $existing_var_type.get_id(),
                            &k,
                            $assertion,
                            !did_remove_type,
                            $negated,
                            span,
                        );
                    }
                }
            }

            if !acceptable_types.is_empty() {
                return Some(TUnion::from_vec(acceptable_types));
            }

            Some(get_never())
        }
    }
}
