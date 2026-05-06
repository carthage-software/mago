<?php

namespace Fixture {
    class ValidBaseClass {}

    interface ValidBaseInterface {}

    trait SomeTrait {}

    /** @deprecated */
    class DeprecatedBaseClass {}

    final class FinalClass {}

    readonly class ReadonlyBaseClass {}

    interface RequiredInterface {}

    /** @require-implements RequiredInterface */
    class ParentRequiresInterface {}

    /** @inheritors AllowedChild */
    class RestrictedParent {}

    class AllowedChild extends RestrictedParent {}

    /**
     * @template T
     *
     */
    class GenericParent {}

    /**
     * @template T of ValidBaseClass
     *
     */
    class ConstrainedGenericParent {}
}

/**
 */
namespace ExtendingNonExistentType {
    class ExtendsNonExistent extends NonExistentClass {}
}

/**
 */
namespace ClassExtendingAnInterface {
    use Fixture\ValidBaseInterface;

    class ExtendsInterface extends ValidBaseInterface {}
}

/**
 */
namespace ClassExtendingTrait {
    use Fixture\SomeTrait;

    class ExtendsTrait extends SomeTrait {}
}

/**
 */
namespace InterfaceExtendingClass {
    use Fixture\ValidBaseClass;

    interface InterfaceExtendsClass extends ValidBaseClass {}
}

/**
 */
namespace InterfaceExtendsTrait {
    use Fixture\SomeTrait;

    interface InterfaceExtendsTrait extends SomeTrait {}
}

/**
 */
namespace ClassExtendingFinalClass {
    use Fixture\FinalClass;

    class ExtendsFinal extends FinalClass {}
}

/**
 */
namespace NonReadonlyExtendsReadonly {
    use Fixture\ReadonlyBaseClass;

    class ExtendsReadonly extends ReadonlyBaseClass {}
}

/**
 */
namespace ExtendingDeprecatedClass {
    use Fixture\DeprecatedBaseClass;

    class ExtendsDeprecated extends DeprecatedBaseClass {}
}

/**
 */
namespace ChildDoesNotImplementRequiredInterface {
    use Fixture\ParentRequiresInterface;

    class MissingRequiredInterface extends ParentRequiresInterface {}
}

/**
 */
namespace UnpermittedChildExtends {
    use Fixture\RestrictedParent;

    class UnpermittedChild extends RestrictedParent {}
}

/**
 */
namespace TooFewTemplateArgs {
    use Fixture\GenericParent;

    /** @extends GenericParent */
    class TooFewTemplateArgs extends GenericParent {}
}

/**
 */
namespace TooManyTemplateArgs {
    use Fixture\GenericParent;

    /** @extends GenericParent<string, int> */
    class TooManyTemplateArgs extends GenericParent {}
}

/**
 */
namespace IncompatibleTemplateArgumentType {
    use Fixture\ConstrainedGenericParent;
    use Fixture\ValidBaseClass;

    /** @extends ConstrainedGenericParent<int> */
    class IncompatibleTemplateArg extends ConstrainedGenericParent {}
}
