<?php

namespace Fixture {
    class NotAnInterface {}

    trait AlsoNotAnInterface {}

    /** @enum-interface */
    interface EnumOnlyInterface {}

    /** @inheritors PermittedImplementor */
    interface RestrictedInterface {}

    class PermittedImplementor implements RestrictedInterface {}

    /**
     * @template T
     *
     */
    interface GenericInterface {}
}

/**
 */
namespace ImplementsNonExistent {
    class ImplementsNonExistent implements NonExistentInterface {}
}

/**
 */
namespace ImplementsClass {
    use Fixture\NotAnInterface;

    class ImplementsClass implements NotAnInterface {}
}

/**
 */
namespace ImplementsTrait {
    use Fixture\AlsoNotAnInterface;

    class ImplementsTrait implements AlsoNotAnInterface {}
}

/**
 */
namespace NonEnumImplementsEnumInterface {
    use Fixture\EnumOnlyInterface;

    class NonEnumImplementsEnumInterface implements EnumOnlyInterface {}
}

/**
 */
namespace UnpermittedImplementor {
    use Fixture\RestrictedInterface;

    class UnpermittedImplementor implements RestrictedInterface {}
}

/**
 */
namespace ImplementsWithTooFewArgs {
    use Fixture\GenericInterface;

    /** @implements GenericInterface */
    class ImplementsWithTooFewArgs implements GenericInterface {}
}

/**
 */
namespace ImplementsWithTooManyArgs {
    use Fixture\GenericInterface;

    /** @implements GenericInterface<string, int> */
    class ImplementsWithTooManyArgs implements GenericInterface {}
}
