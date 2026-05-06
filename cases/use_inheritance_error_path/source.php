<?php

namespace Fixture {
    class NotATrait {}

    interface AlsoNotATrait {}

    /** @deprecated */
    trait DeprecatedTrait {}

    #[\Deprecated]
    trait DeprecatedTraitFromAttribute {}

    interface RequiredInterface {}

    /** @require-implements RequiredInterface */
    trait RequiresInterfaceTrait {}

    class RequiredClass {}

    /** @require-extends RequiredClass */
    trait RequiresClassTrait {}

    /** @inheritors PermittedUser */
    trait RestrictedTrait {}

    class PermittedUser
    {
        use RestrictedTrait;
    }

    /**
     * @template T
     *
     */
    trait GenericTrait {}
}

/**
 */
namespace UsesNonExistent {
    class UsesNonExistent
    {
        use NonExistentTrait;
    }
}

/**
 */
namespace UsesClass {
    use Fixture\NotATrait;

    class UsesClass
    {
        use NotATrait;
    }
}

/**
 */
namespace UsesInterface {
    use Fixture\AlsoNotATrait;

    class UsesInterface
    {
        use AlsoNotATrait;
    }
}

/**
 */
namespace UsesDeprecated {
    use Fixture\DeprecatedTrait;

    class UsesDeprecated
    {
        use DeprecatedTrait;
    }
}

/**
 */
namespace UsesDeprecatedFromAttribute {
    use Fixture\DeprecatedTraitFromAttribute;

    class UsesDeprecatedFromAttribute
    {
        use DeprecatedTraitFromAttribute;
    }
}

/**
 */
namespace MissingRequiredInterface {
    use Fixture\RequiresInterfaceTrait;

    class MissingRequiredInterface
    {
        use RequiresInterfaceTrait;
    }
}

/**
 */
namespace MissingRequiredClass {
    use Fixture\RequiresClassTrait;

    class MissingRequiredClass
    {
        use RequiresClassTrait;
    }
}

/**
 */
namespace UnpermittedUser {
    use Fixture\RestrictedTrait;

    class UnpermittedUser
    {
        use RestrictedTrait;
    }
}

/**
 */
namespace UsesWithTooFewArgs {
    use Fixture\GenericTrait;

    class UsesWithTooFewArgs
    {
        use GenericTrait;
    }
}

/**
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
