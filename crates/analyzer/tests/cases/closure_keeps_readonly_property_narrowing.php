<?php

declare(strict_types=1);

final class Box
{
    public function __construct(public readonly ?string $value = null) {}
}

function f(Box $box): Closure
{
    if ($box->value === null) {
        return static fn(): string => '';
    }

    return static fn(): string => $box->value;
}

final class Holder
{
    public function __construct(private readonly ?string $box) {}

    /**
     * @return Closure(): string
     */
    public function arrow(): Closure
    {
        if ($this->box === null) {
            return static fn(): string => '';
        }

        return fn(): string => $this->box;
    }

    public function closure(): Closure
    {
        if ($this->box === null) {
            return static fn(): string => '';
        }

        return function (): string {
            return $this->box;
        };
    }
}


final class Mut
{
    public ?string $value = null;

    public function reset(): void
    {
        $this->value = null;
    }
}

function lengthLater(Mut $mut): Closure
{
    if ($mut->value === null) {
        return static fn(): int => 0;
    }

    return static function () use ($mut): int {
        // @mago-expect analysis:possibly-null-argument
        return strlen($mut->value);
    };
}

