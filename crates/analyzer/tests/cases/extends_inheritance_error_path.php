<?php

namespace Fixture {
    class ValidBaseClass
    {
    }

    interface ValidBaseInterface
    {
    }

    trait SomeTrait
    {
    }

    /** @deprecated */
    class DeprecatedBaseClass
    {
    }

    final class FinalClass
    {
    }

    readonly class ReadonlyBaseClass
    {
    }

    /** @mutation-free */
    class MutationFreeClass
    {
    }

    /** @external-mutation-free */
    class ExternalMutationFreeClass
    {
    }

    interface RequiredInterface
    {
    }

    /** @require-implements RequiredInterface */
    class ParentRequiresInterface
    {
    }

    /** @inheritors AllowedChild */
    class RestrictedParent
    {
    }

    class AllowedChild extends RestrictedParent
    {
    }

    /** @template T */
    class GenericParent
    {
    }

    /** @template T of ValidBaseClass */
    class ConstrainedGenericParent
    {
    }
}

/**
 * @mago-expect analysis:non-existent-class-like
 */
namespace ExtendingNonExistentType {
    class ExtendsNonExistent extends NonExistentClass
    {
    }
}

/**
 * @mago-expect analysis:invalid-extend
 */
namespace ClassExtendingAnInterface {
    use Fixture\ValidBaseInterface;

    class ExtendsInterface extends ValidBaseInterface
    {
    }
}

/**
 * @mago-expect analysis:invalid-extend
 */
namespace ClassExtendingTrait {
    use Fixture\SomeTrait;

    class ExtendsTrait extends SomeTrait
    {
    }
}

/**
 * @mago-expect analysis:invalid-extend
 */
namespace InterfaceExtendingClass {
    use Fixture\ValidBaseClass;

    interface InterfaceExtendsClass extends ValidBaseClass
    {
    }
}

/**
 * @mago-expect analysis:invalid-extend
 */
namespace InterfaceExtendsTrait {
    use Fixture\SomeTrait;

    interface InterfaceExtendsTrait extends SomeTrait
    {
    }
}

/**
 * @mago-expect analysis:extend-final-class
 */
namespace ClassExtendingFinalClass {
    use Fixture\FinalClass;

    class ExtendsFinal extends FinalClass
    {
    }
}

/**
 * @mago-expect analysis:invalid-extend
 */
namespace NonReadonlyExtendsReadonly {
    use Fixture\ReadonlyBaseClass;

    class ExtendsReadonly extends ReadonlyBaseClass
    {
    }
}

/**
 * @mago-expect analysis:deprecated-class
 */
namespace ExtendingDeprecatedClass {
    use Fixture\DeprecatedBaseClass;

    class ExtendsDeprecated extends DeprecatedBaseClass
    {
    }
}

/**
 * @mago-expect analysis:invalid-extend
 */
namespace MutableExtendsMutationFree {
    use Fixture\MutationFreeClass;

    class ExtendsMutationFree extends MutationFreeClass
    {
    }
}

/**
 * @mago-expect analysis:invalid-extend
 */
namespace MutableExtendsExternalMutationFree {
    use Fixture\ExternalMutationFreeClass;

    class ExtendsExternalMutationFree extends ExternalMutationFreeClass
    {
    }
}

/**
 * @mago-expect analysis:missing-required-interface
 */
namespace ChildDoesNotImplementRequiredInterface {
    use Fixture\ParentRequiresInterface;

    class MissingRequiredInterface extends ParentRequiresInterface
    {
    }
}

/**
 * @mago-expect analysis:invalid-extend
 */
namespace UnpermittedChildExtends {
    use Fixture\RestrictedParent;

    class UnpermittedChild extends RestrictedParent
    {
    }
}

/**
 * @mago-expect analysis:missing-template-parameter
 */
namespace TooFewTemplateArgs {
    use Fixture\GenericParent;

    /** @extends GenericParent */
    class TooFewTemplateArgs extends GenericParent
    {
    }
}

/**
 * @mago-expect analysis:excess-template-parameter
 */
namespace TooManyTemplateArgs {
    use Fixture\GenericParent;

    /** @extends GenericParent<string, int> */
    class TooManyTemplateArgs extends GenericParent
    {
    }
}

/**
 * @mago-expect analysis:invalid-template-parameter
 */
namespace IncompatibleTemplateArgumentType {
    use Fixture\ConstrainedGenericParent;
    use Fixture\ValidBaseClass;

    /** @extends ConstrainedGenericParent<int> */
    class IncompatibleTemplateArg extends ConstrainedGenericParent
    {
    }
}
