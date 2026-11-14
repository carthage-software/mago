<?php

declare(strict_types=1);

class A
{
    public function viaThis(): void
    {
        if ($this::class === B::class) {
            $this->onlyB();
        }
    }

    public function viaStatic(): void
    {
        if (static::class === B::class) {
            $this->onlyB();
        }
    }

    public function viaStaticNegated(): void
    {
        if (static::class !== B::class) {
            return;
        }

        $this->onlyB();
    }
}

final class B extends A
{
    public function onlyB(): void {}
}

/** @param array<int, string> $xs */
function first(array $xs): int
{
    if (count($xs, COUNT_NORMAL) > 0) {

        return array_key_first($xs);
    }

    return 0;
}

/** @param array<int, string> $xs */
function firstOneArgument(array $xs): int
{
    if (count($xs) > 0) {

        return array_key_first($xs);
    }

    return 0;
}
