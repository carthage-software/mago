<?php

declare(strict_types=1);

/** @template T of object */
final class StaticBox
{
    /** @param T $value */
    public function __construct(
        public object $value,
    ) {}
}

class StaticBase
{
    /** @return StaticBox<StaticMid> */
    public function box(): StaticBox
    {
        return new StaticBox(new StaticMid());
    }
}

class StaticMid extends StaticBase
{
    /**
     * @return StaticBox<static>
     * @mago-expect analysis:incompatible-return-type
     */
    #[Override]
    final public function box(): StaticBox
    {
        return new StaticBox(new static());
    }

    final public function __construct() {}
}

final class StaticSub extends StaticMid {}

/** @param StaticBox<StaticSub> $box */
function static_box_check(StaticBox $box): void
{
    // @mago-expect analysis:less-specific-argument
    static_box_accept($box);
}

/** @param StaticBox<StaticMid> $box */
function static_box_accept(StaticBox $box): void
{
    $box->value = new StaticMid();
}
