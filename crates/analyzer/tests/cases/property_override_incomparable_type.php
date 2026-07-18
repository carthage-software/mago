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
