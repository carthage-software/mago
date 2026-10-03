<?php

declare(strict_types=1);

namespace Issue2419;

class Base
{
    private const PRIVATE_VALUE = 1;
    protected const PROTECTED_VALUE = 2;
    public const PUBLIC_VALUE = 3;

    public function own(): int
    {
        return self::PRIVATE_VALUE + self::PROTECTED_VALUE + Child::PROTECTED_VALUE;
    }

    public function childPrivate(): int
    {
        // @mago-expect analysis:non-existent-class-constant,never-return
        return Child::PRIVATE_VALUE;
    }
}

class Child extends Base
{
    public function inherited(): int
    {
        return parent::PROTECTED_VALUE + self::PROTECTED_VALUE + Sibling::PROTECTED_VALUE;
    }

    public function parentPrivate(): int
    {
        // @mago-expect analysis:invalid-constant-access
        return parent::PRIVATE_VALUE;
    }

    public function childPrivate(): int
    {
        // @mago-expect analysis:non-existent-class-constant,never-return
        return self::PRIVATE_VALUE;
    }
}

class Sibling extends Base {}

class RedeclaresProtected extends Base
{
    protected const PROTECTED_VALUE = 6;
}

class ReadsSibling extends Base
{
    public function redeclared(): int
    {
        // @mago-expect analysis:invalid-constant-access
        return RedeclaresProtected::PROTECTED_VALUE;
    }
}

class RedeclaresPrivate extends Base
{
    private const PRIVATE_VALUE = 7;

    public function ownPrivate(): int
    {
        return self::PRIVATE_VALUE;
    }
}

class Unrelated
{
    public function inaccessible(): int
    {
        // @mago-expect analysis:invalid-constant-access
        return Base::PROTECTED_VALUE;
    }
}

/** @require-extends Base */
trait RequiresBase
{
    public function requiredProtected(): int
    {
        return self::PROTECTED_VALUE;
    }

    public function requiredPrivate(): int
    {
        // @mago-expect analysis:invalid-constant-access
        return self::PRIVATE_VALUE;
    }
}

class UsesRequirement extends Base
{
    use RequiresBase;
}

trait Constants
{
    private const TRAIT_PRIVATE = 4;
    protected const TRAIT_PROTECTED = 5;

    public function traitOwn(): int
    {
        return self::TRAIT_PRIVATE + static::TRAIT_PROTECTED;
    }
}

class UsesConstants
{
    use Constants;

    public function own(): int
    {
        return self::TRAIT_PRIVATE + self::TRAIT_PROTECTED;
    }
}

class InheritsConstants extends UsesConstants
{
    public function inherited(): int
    {
        return parent::TRAIT_PROTECTED;
    }

    public function parentPrivate(): int
    {
        // @mago-expect analysis:invalid-constant-access
        return parent::TRAIT_PRIVATE;
    }
}

class ReusesConstants extends UsesConstants
{
    use Constants;

    public function ownPrivate(): int
    {
        return self::TRAIT_PRIVATE;
    }
}

class OtherUsesConstants
{
    use Constants;

    public function otherPrivate(): int
    {
        // @mago-expect analysis:invalid-constant-access
        return UsesConstants::TRAIT_PRIVATE;
    }
}

function outside(): void
{
    echo Base::PUBLIC_VALUE;
    echo Child::PUBLIC_VALUE;
    // @mago-expect analysis:invalid-constant-access
    echo Base::PRIVATE_VALUE;
    // @mago-expect analysis:invalid-constant-access
    echo Base::PROTECTED_VALUE;
    // @mago-expect analysis:invalid-constant-access
    echo Child::PROTECTED_VALUE;
    // @mago-expect analysis:non-existent-class-constant,no-value
    echo Child::PRIVATE_VALUE;
    // @mago-expect analysis:invalid-constant-access
    echo UsesConstants::TRAIT_PRIVATE;
    // @mago-expect analysis:invalid-constant-access
    echo UsesConstants::TRAIT_PROTECTED;
}

function dynamic(Base $object): int
{
    $class = Base::class;
    $constant = 'PRIVATE_VALUE';
    // @mago-expect analysis:invalid-constant-access
    echo $class::PRIVATE_VALUE;
    // @mago-expect analysis:invalid-constant-access
    echo Base::{$constant};
    // @mago-expect analysis:invalid-constant-access
    return $object::PROTECTED_VALUE;
}

#[\Attribute(\Attribute::TARGET_CLASS)]
class Marker
{
    public function __construct(public int $value) {}
}

#[Marker(OwnAttribute::VALUE)]
class OwnAttribute
{
    private const VALUE = 8;
}

// @mago-expect analysis:invalid-constant-access
#[Marker(Base::PRIVATE_VALUE)]
class OtherAttribute {}

enum State
{
    case Ready;

    private const VALUE = 9;

    public function own(): int
    {
        return self::VALUE;
    }
}

function enumConstant(): void
{
    echo State::Ready->name;
    // @mago-expect analysis:invalid-constant-access
    echo State::VALUE;
}
