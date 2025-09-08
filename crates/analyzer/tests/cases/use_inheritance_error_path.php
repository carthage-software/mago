<?php

namespace Fixture {
    class NotATrait
    {
    }

    interface AlsoNotATrait
    {
    }

    /** @deprecated */
    trait DeprecatedTrait
    {
    }

    #[\Deprecated]
    trait DeprecatedTraitFromAttribute
    {
    }

    interface RequiredInterface
    {
    }

    /** @require-implements RequiredInterface */
    trait RequiresInterfaceTrait
    {
    }

    class RequiredClass
    {
    }

    /** @require-extends RequiredClass */
    trait RequiresClassTrait
    {
    }

    /** @inheritors PermittedUser */
    trait RestrictedTrait
    {
    }

    class PermittedUser
    {
        use RestrictedTrait;
    }

    /** @template T */
    trait GenericTrait
    {
    }
}

/**
 * @mago-expect analysis:non-existent-class-like
 */
namespace UsesNonExistent {
    class UsesNonExistent
    {
        use NonExistentTrait;
    }
}

/**
 * @mago-expect analysis:invalid-trait-use
 */
namespace UsesClass {
    use Fixture\NotATrait;

    class UsesClass
    {
        use NotATrait;
    }
}

/**
 * @mago-expect analysis:invalid-trait-use
 */
namespace UsesInterface {
    use Fixture\AlsoNotATrait;

    class UsesInterface
    {
        use AlsoNotATrait;
    }
}

/**
 * @mago-expect analysis:deprecated-trait
 */
namespace UsesDeprecated {
    use Fixture\DeprecatedTrait;

    class UsesDeprecated
    {
        use DeprecatedTrait;
    }
}

/**
 * @mago-expect analysis:deprecated-trait
 */
namespace UsesDeprecatedFromAttribute {
    use Fixture\DeprecatedTraitFromAttribute;

    class UsesDeprecatedFromAttribute
    {
        use DeprecatedTraitFromAttribute;
    }
}

/**
 * @mago-expect analysis:missing-required-interface
 */
namespace MissingRequiredInterface {
    use Fixture\RequiresInterfaceTrait;

    class MissingRequiredInterface
    {
        use RequiresInterfaceTrait;
    }
}

/**
 * @mago-expect analysis:missing-required-parent
 */
namespace MissingRequiredClass {
    use Fixture\RequiresClassTrait;

    class MissingRequiredClass
    {
        use RequiresClassTrait;
    }
}

/**
 * @mago-expect analysis:invalid-trait-use
 */
namespace UnpermittedUser {
    use Fixture\RestrictedTrait;

    class UnpermittedUser
    {
        use RestrictedTrait;
    }
}

/**
 * @mago-expect analysis:missing-template-parameter
 */
namespace UsesWithTooFewArgs {
    use Fixture\GenericTrait;

    class UsesWithTooFewArgs
    {
        use GenericTrait;
    }
}

/**
 * @mago-expect analysis:excess-template-parameter
 */
namespace UsesWithTooManyArgs {
    use Fixture\GenericTrait;

    class UsesWithTooManyArgs
    {
        /**
         * @use GenericTrait<string, int>
         */
        use GenericTrait;
    }
}
