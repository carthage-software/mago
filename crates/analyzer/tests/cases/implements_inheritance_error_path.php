<?php

namespace Fixture {
    class NotAnInterface
    {
    }

    trait AlsoNotAnInterface
    {
    }

    /** @enum-interface */
    interface EnumOnlyInterface
    {
    }

    /** @inheritors PermittedImplementor */
    interface RestrictedInterface
    {
    }

    class PermittedImplementor implements RestrictedInterface
    {
    }

    /** @template T */
    interface GenericInterface
    {
    }
}

/**
 * @mago-expect analysis:non-existent-class-like
 */
namespace ImplementsNonExistent {
    class ImplementsNonExistent implements NonExistentInterface
    {
    }
}

/**
 * @mago-expect analysis:invalid-implement
 */
namespace ImplementsClass {
    use Fixture\NotAnInterface;

    class ImplementsClass implements NotAnInterface
    {
    }
}

/**
 * @mago-expect analysis:invalid-implement
 */
namespace ImplementsTrait {
    use Fixture\AlsoNotAnInterface;

    class ImplementsTrait implements AlsoNotAnInterface
    {
    }
}

/**
 * @mago-expect analysis:invalid-implement
 */
namespace NonEnumImplementsEnumInterface {
    use Fixture\EnumOnlyInterface;

    class NonEnumImplementsEnumInterface implements EnumOnlyInterface
    {
    }
}

/**
 * @mago-expect analysis:invalid-implement
 */
namespace UnpermittedImplementor {
    use Fixture\RestrictedInterface;

    class UnpermittedImplementor implements RestrictedInterface
    {
    }
}

/**
 * @mago-expect analysis:missing-template-parameter
 */
namespace ImplementsWithTooFewArgs {
    use Fixture\GenericInterface;

    /** @implements GenericInterface */
    class ImplementsWithTooFewArgs implements GenericInterface
    {
    }
}

/**
 * @mago-expect analysis:excess-template-parameter
 */
namespace ImplementsWithTooManyArgs {
    use Fixture\GenericInterface;

    /** @implements GenericInterface<string, int> */
    class ImplementsWithTooManyArgs implements GenericInterface
    {
    }
}
