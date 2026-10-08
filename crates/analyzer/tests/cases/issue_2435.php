<?php

declare(strict_types=1);

namespace Issue2435;

final class Builder
{
    public function andWhere(): self
    {
        $_args = func_get_args();
        return $this;
    }

    public function nth(): mixed
    {
        return func_get_arg(0);
    }

    public function total(): int
    {
        return func_num_args();
    }

    public function afterHook(): int
    {
        $_o = new class { public int $x { get => 1; } };

        return func_num_args();
    }

    public function plain(): self
    {
        $_n = count([]);

        return $this;
    }

    public function hooked(): object
    {
        return new class {
            public string $v = '' {
                set => (string) count(func_get_args());
            }
        };
    }

    public function nested(): \Closure
    {
        return static fn(): int => count(func_get_args());
    }
}

function qualified(): int
{
    return count(\func_get_args());
}

function run(Builder $b): void
{
    $b->andWhere('a', 'b');
    $b->nth('a', 'b');
    $b->total('a', 'b');
    echo $b->afterHook('a');
    echo qualified(1, 2);

    /** @var non-empty-list<string> $parts */
    $parts = ['a', 'b'];
    $b->andWhere(...$parts);

    $closure = static function (): int {
        return count(func_get_args());
    };
    echo $closure(1, 2);

    // @mago-expect analysis:too-many-arguments
    $b->plain('a');
    // @mago-expect analysis:too-many-arguments
    $b->nested('a');
    // @mago-expect analysis:too-many-arguments
    $b->hooked('a');
}
