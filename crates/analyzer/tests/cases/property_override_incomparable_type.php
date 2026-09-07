<?php

declare(strict_types=1);

namespace PropertyOverrideIncomparableType;

class Wide
{
}

class Unrelated
{
}

class Base
{
    public ?Wide $native = null;
}

// @mago-expect analysis:incompatible-property-type
class NativeIncomparable extends Base
{
    public ?Unrelated $native = null;
}

class DocblockBase
{
    /** @var Wide|null */
    public mixed $value = null;
}

// @mago-expect analysis:incompatible-property-type
class DocblockIncomparable extends DocblockBase
{
    /** @var Unrelated|null */
    public mixed $value = null;
}

class VirtualBase
{
    public ?Wide $virtual {
        get => null;
    }
}

// A get-only virtual property allows covariant overrides, but an
// incomparable type is not covariant.
// @mago-expect analysis:incompatible-property-type
class VirtualIncomparable extends VirtualBase
{
    public ?Unrelated $virtual {
        get => null;
    }
}

/**
 * @template TValue of int|string
 */
abstract class GenericBase
{
    /** @var array<TValue, string> */
    protected array $choices = [];
}

/**
 * @extends GenericBase<string>
 */
final class GenericChild extends GenericBase
{
    /** @var array<string, string> */
    protected array $choices = ['a' => 'A'];
}

/**
 * The parent's type is localized before comparison, so an override that does
 * not match the substituted type is still reported.
 *
 * @extends GenericBase<int>
 */
// @mago-expect analysis:incompatible-property-type
final class GenericChildIncomparable extends GenericBase
{
    /** @var array<string, string> */
    protected array $choices = ['a' => 'A'];
}
