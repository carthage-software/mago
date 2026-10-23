<?php

declare(strict_types=1);

namespace TemplateArgumentBounds;

/**
 * @template T of object
 * @template U of T
 */
class Pair
{
    /**
     * @param T $first
     * @param U $second
     */
    public function __construct(public object $first, public object $second) {}
}

class ParentType {}

final class ChildType extends ParentType {}

final class OtherType {}

/** @param Pair<ParentType, ChildType> $pair */
function validDependentBound(Pair $pair): void {}

// @mago-expect analysis:invalid-template-parameter
/** @param Pair<ParentType, OtherType> $pair */
function invalidDependentBound(Pair $pair): void {}

/**
 * @template T of ParentType
 * @param Pair<ParentType, T> $pair
 */
function validTemplateArgument(Pair $pair): void {}

// @mago-expect analysis:invalid-template-parameter
/**
 * @template T of object
 * @param Pair<ParentType, T> $pair
 */
function invalidTemplateArgument(Pair $pair): void {}

/**
 * @template-covariant T
 * @template-covariant TFlag of bool = false
 */
class Defaulted
{
    /**
     * @param T $value
     * @param TFlag $flag
     */
    public function __construct(public mixed $value, public bool $flag = false) {}
}

/** @param Defaulted<string> $value */
function validDefault(Defaulted $value): void {}

/** @param Defaulted<string, true> $value */
function validExplicit(Defaulted $value): void {}

/** @param Defaulted<string, *> $value */
function validWildcard(Defaulted $value): void {}

// @mago-expect analysis:invalid-template-parameter
/** @param Defaulted<string, int> $value */
function invalidExplicit(Defaulted $value): void {}

/**
 * @template T of \Countable&\Stringable
 */
class IntersectionBound
{
    /** @param T $value */
    public function __construct(public object $value) {}
}

/** @param IntersectionBound<\Countable&\Stringable> $value */
function validIntersection(IntersectionBound $value): void {}

// @mago-expect analysis:invalid-template-parameter
/** @param IntersectionBound<\Countable> $value */
function invalidIntersection(IntersectionBound $value): void {}

/** @param \Generator<string> $value */
function generatorShorthand(\Generator $value): void {}

/** @param \Iterator<string> $value */
function iteratorShorthand(\Iterator $value): void {}

/** @return Defaulted<string, true> */
function defaultedNativeReturn(): Defaulted
{
    return new Defaulted('value', true);
}

/**
 * @template-covariant T
 * @template-covariant TFlag of bool = true
 */
class AllDefaulted
{
    /**
     * @param T $value
     * @param TFlag $flag
     */
    public function __construct(public mixed $value, public bool $flag = true) {}
}

/** @param AllDefaulted<string> $value */
function defaultedArgument(AllDefaulted $value): void {}

defaultedArgument(new AllDefaulted('value'));
