use mago_interner::ThreadedInterner;
use mago_reflection::class_like::ClassLikeReflection;
use mago_reflection::identifier::Name;
use mago_reflection::Reflection;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;

const CODE: &str = "reflection";

#[inline]
pub fn report_missing_parent_class(
    interner: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    class_name: &Name,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_kind = reflection.name.get_kind();
    let class_like_name = reflection.name.get_key(interner);
    let class_name_value = interner.lookup(&class_name.value);

    reflection.issues.push(
        Issue::error(format!(
            "{} `{}` extends undefined class `{}`.",
            class_like_kind, class_like_name, class_name_value
        ))
        .with_code(CODE)
        .with_annotation(
            Annotation::primary(class_name.span).with_message(format!("Class `{}` does not exist.", class_name_value)),
        )
        .with_help(format!("Ensure the class `{}` is defined or imported before extending it.", class_name_value)),
    );
}

#[inline]
pub fn report_parent_not_class(interner: &ThreadedInterner, reflection: &mut ClassLikeReflection, class_name: &Name) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_kind = reflection.name.get_kind();
    let class_like_name = reflection.name.get_key(interner);
    let class_name_value = interner.lookup(&class_name.value);

    reflection.issues.push(
        Issue::error(format!("{} `{}` extends non-class `{}`.", class_like_kind, class_like_name, class_name_value))
            .with_code(CODE)
            .with_annotation(
                Annotation::primary(class_name.span).with_message(format!("`{}` is not a class.", class_name_value)),
            )
            .with_help(format!("Ensure the class `{}` is defined or imported before extending it.", class_name_value)),
    );
}

#[inline]
pub fn report_parent_class_is_final(
    interner: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    class_name: &Name,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_kind = reflection.name.get_kind();
    let class_like_name = reflection.name.get_key(interner);
    let class_name_value = interner.lookup(&class_name.value);

    reflection.issues.push(
        Issue::error(format!("{} `{}` extends final class `{}`.", class_like_kind, class_like_name, class_name_value))
            .with_code(CODE)
            .with_annotation(
                Annotation::primary(class_name.span).with_message(format!("Class `{}` is final.", class_name_value)),
            )
            .with_help(format!("Ensure the class `{}` is not final or remove the `extends` clause.", class_name_value)),
    );
}

#[inline]
pub fn report_parent_class_is_readonly(
    interner: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    parent_name: &Name,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_name = reflection.name.get_key(interner);
    let parent_name_value = interner.lookup(&parent_name.value);

    reflection.issues.push(
        Issue::error(format!(
            "Extending readonly class `{}` from non-readonly class `{}`.",
            parent_name_value, class_like_name
        ))
        .with_code(CODE)
        .with_annotation(
            Annotation::primary(parent_name.span).with_message(format!("Class `{}` is readonly.", parent_name_value)),
        )
        .with_annotation(
            Annotation::secondary(reflection.name.span())
                .with_message(format!("Class `{}` is not readonly.", class_like_name)),
        )
        .with_help(format!("Mark the class `{}` as readonly or remove the `extends` clause.", class_like_name)),
    );
}

#[inline]
pub fn report_parent_class_is_not_readonly(
    interner: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    parent_name: &Name,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_name = reflection.name.get_key(interner);
    let parent_name_value = interner.lookup(&parent_name.value);

    reflection.issues.push(
        Issue::error(format!(
            "Cannot extend non-readonly class `{}` from readonly class `{}`.",
            parent_name_value, class_like_name
        ))
        .with_code(CODE)
        .with_annotation(
            Annotation::primary(parent_name.span)
                .with_message(format!("Class `{}` is not readonly.", parent_name_value)),
        )
        .with_annotation(
            Annotation::secondary(reflection.name.span())
                .with_message(format!("Class `{}` is readonly.", class_like_name)),
        )
        .with_help(format!("Ensure the class `{}` is readonly or remove the `extends` clause.", parent_name_value)),
    );
}

pub fn report_parent_class_circular_reference(
    interner: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    parent_name: &Name,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_name_str = reflection.name.get_key(interner);
    let parent_name_str = interner.lookup(&parent_name.value);

    reflection.issues.push(
        Issue::error(format!("Circular inheritance detected between `{}` and `{}`.", class_name_str, parent_name_str))
            .with_code(CODE)
            .with_annotation(
                Annotation::primary(parent_name.span)
                    .with_message(format!("Class `{}` already extends `{}`.", parent_name_str, class_name_str)),
            )
            .with_help(format!(
                "Ensure there is no circular inheritance between `{}` and `{}`.",
                class_name_str, parent_name_str
            )),
    );
}

#[inline]
pub fn report_missing_parent_interface(
    interner: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    interface_name: &Name,
    implemented: bool,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_kind = reflection.name.get_kind();
    let class_like_name = reflection.name.get_key(interner);
    let interface_name_value = interner.lookup(&interface_name.value);

    reflection.issues.push(
        Issue::error(format!(
            "{} `{}` {} undefined interface `{}`.",
            class_like_kind,
            class_like_name,
            if implemented { "implements" } else { "extends" },
            interface_name_value
        ))
        .with_code(CODE)
        .with_annotation(
            Annotation::primary(interface_name.span)
                .with_message(format!("Interface `{}` does not exist.", interface_name_value)),
        )
        .with_help(format!(
            "Ensure the interface `{}` is defined or imported before {} it.",
            interface_name_value,
            if implemented { "implementing" } else { "extending" }
        )),
    );
}

pub fn report_parent_not_interface(
    interner: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    interface_name: &Name,
    implemented: bool,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_kind = reflection.name.get_kind();
    let class_like_name = reflection.name.get_key(interner);
    let interface_name_value = interner.lookup(&interface_name.value);

    reflection.issues.push(
        Issue::error(format!(
            "{} `{}` {} non-interface `{}`.",
            class_like_kind,
            class_like_name,
            if implemented { "implements" } else { "extends" },
            interface_name_value
        ))
        .with_code(CODE)
        .with_annotation(
            Annotation::primary(interface_name.span)
                .with_message(format!("`{}` is not an interface.", interface_name_value)),
        )
        .with_help(format!(
            "Ensure the interface `{}` is defined or imported before {} it.",
            interface_name_value,
            if implemented { "implementing" } else { "extending" }
        )),
    );
}

pub fn report_parent_interface_circular_reference(
    interner: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    interface_name: &Name,
    implemented: bool,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_name_str = reflection.name.get_key(interner);
    let interface_name_str = interner.lookup(&interface_name.value);

    reflection.issues.push(
        Issue::error(format!(
            "Circular inheritance detected between `{}` and `{}`.",
            class_name_str, interface_name_str
        ))
        .with_code(CODE)
        .with_annotation(Annotation::primary(interface_name.span).with_message(format!(
            "Interface `{}` already {} `{}`.",
            interface_name_str,
            if implemented { "implements" } else { "extends" },
            class_name_str
        )))
        .with_help(format!(
            "Ensure there is no circular inheritance between `{}` and `{}`.",
            class_name_str, interface_name_str
        )),
    );
}

#[inline]
pub fn report_missing_trait(interner: &ThreadedInterner, reflection: &mut ClassLikeReflection, trait_name: &Name) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_kind = reflection.name.get_kind();
    let class_like_name = reflection.name.get_key(interner);
    let trait_name_value = interner.lookup(&trait_name.value);

    reflection.issues.push(
        Issue::error(format!("{} `{}` uses undefined trait `{}`.", class_like_kind, class_like_name, trait_name_value))
            .with_code(CODE)
            .with_annotation(
                Annotation::primary(trait_name.span)
                    .with_message(format!("Trait `{}` does not exist.", trait_name_value)),
            )
            .with_help(format!("Ensure the trait `{}` is defined or imported before using it.", trait_name_value)),
    );
}

#[inline]
pub fn report_not_trait(interner: &ThreadedInterner, reflection: &mut ClassLikeReflection, trait_name: &Name) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_like_kind = reflection.name.get_kind();
    let class_like_name = reflection.name.get_key(interner);
    let trait_name_value = interner.lookup(&trait_name.value);

    reflection.issues.push(
        Issue::error(format!("{} `{}` uses non-trait `{}`.", class_like_kind, class_like_name, trait_name_value))
            .with_code(CODE)
            .with_annotation(
                Annotation::primary(trait_name.span).with_message(format!("`{}` is not a trait.", trait_name_value)),
            )
            .with_help(format!("Ensure the trait `{}` is defined or imported before using it.", trait_name_value)),
    );
}

#[inline]
pub fn report_trait_circular_reference(
    inter: &ThreadedInterner,
    reflection: &mut ClassLikeReflection,
    trait_name: &Name,
) {
    if !reflection.is_user_defined() {
        return;
    }

    let class_name_str = reflection.name.get_key(inter);
    let trait_name_str = inter.lookup(&trait_name.value);

    reflection.issues.push(
        Issue::error(format!("Circular inheritance detected between `{}` and `{}`.", class_name_str, trait_name_str))
            .with_code(CODE)
            .with_annotation(
                Annotation::primary(trait_name.span)
                    .with_message(format!("Trait `{}` already uses `{}`.", trait_name_str, class_name_str)),
            )
            .with_help(format!(
                "Ensure there is no circular inheritance between `{}` and `{}`.",
                class_name_str, trait_name_str
            )),
    );
}
