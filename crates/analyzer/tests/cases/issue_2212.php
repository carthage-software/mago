<?php

declare(strict_types=1);

final class Config
{
    public function isEnabled(): bool
    {
        return true;
    }
}

function takesString(string $_value): void {}

function takesMixed(mixed $_value): void {}

/** @mago-expect analysis:non-existent-class-like */
final class MissingInterface implements Vendor\Unresolvable\Contract
{
    public function __construct(private Config $config) {}

    public function run(): void
    {
        if (!$this->config->isEnabled()) {}

        /** @mago-expect analysis:invalid-argument */
        takesString(1);

        $this->unknownMethod(takesString('argument is still analyzed'));
        takesMixed($this->unknownProperty);
        self::unknownStaticMethod();
        takesMixed(self::$unknownStaticProperty);
        takesMixed(self::UNKNOWN_CONSTANT);

        $this->unknownProperty = 1;
        self::$unknownStaticProperty = 1;
        takesMixed($this->unknownMethod(...));
        takesMixed(self::unknownStaticMethod(...));
    }

    #[\Override]
    public function possiblyInherited(): void {}
}

/** @mago-expect analysis:non-existent-class-like */
class MissingParent extends Vendor\Unresolvable\BaseClass
{
    public function inheritedAccesses(): void
    {
        parent::unknownStaticMethod();
        takesMixed(parent::$unknownStaticProperty);
        takesMixed(parent::UNKNOWN_CONSTANT);
        parent::$unknownStaticProperty = 1;
        takesMixed(parent::unknownStaticMethod(...));
    }
}

final class TransitiveChild extends MissingParent
{
    public function localBody(): void
    {
        /** @mago-expect analysis:invalid-argument */
        takesString(2);

        $this->unknownMethod();
    }
}

/**
 * @template T
 * @extends Vendor\Unresolvable\GenericBase<T>
 *
 * @mago-expect analysis:non-existent-class-like
 */
class MissingGenericParent extends Vendor\Unresolvable\GenericBase {}

final class MissingTrait
{
    /** @mago-expect analysis:non-existent-class-like */
    use Vendor\Unresolvable\Behavior;

    public function localBody(): void
    {
        /** @mago-expect analysis:invalid-argument */
        takesString(3);

        $this->unknownMethod();
    }
}

trait TraitWithMissingTrait
{
    /** @mago-expect analysis:non-existent-class-like */
    use Vendor\Unresolvable\NestedBehavior;

    public function traitBody(): void
    {
        /** @mago-expect analysis:invalid-argument */
        takesString(5);
    }
}

final class TransitiveTraitUser
{
    use TraitWithMissingTrait;

    public function inheritedAccess(): void
    {
        $this->unknownMethod();
    }
}

/** @mago-expect analysis:invalid-argument */
new MissingParent(takesString(6));

/** @mago-expect analysis:non-existent-class-like */
$anonymous = new class(
    /** @mago-expect analysis:invalid-argument */
    takesString(7),
) extends Vendor\Unresolvable\AnonymousBase {
    /** @mago-expect analysis:invalid-argument */
    public function localBody(): void
    {
        takesString(4);
    }
};
