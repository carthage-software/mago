<?php

declare(strict_types=1);

namespace Issue2315;

final class Item
{
    public function __construct(
        public bool $active,
        public int $id,
        public string $label,
    ) {}
}

final class Holder
{
    public function __construct(public ?Item $item) {}
}

final readonly class Repro
{
    public function afterTrueComparison(Holder $holder): void
    {
        if ($holder->item?->active === true) {
            self::useInt($holder->item->id);
        }
    }

    public function afterFalseComparison(Holder $holder): void
    {
        if ($holder->item?->active === false) {
            self::useInt($holder->item->id);
        }
    }

    public function afterReversedTrueComparison(Holder $holder): void
    {
        if (true === $holder->item?->active) {
            self::useString($holder->item->label);
        }
    }

    public function afterReversedFalseComparison(Holder $holder): void
    {
        if (false === $holder->item?->active) {
            self::useString($holder->item->label);
        }
    }

    public function afterIntComparison(Holder $holder): void
    {
        if ($holder->item?->id === 42) {
            self::useString($holder->item->label);
        }
    }

    public function afterStringComparison(Holder $holder): void
    {
        if ($holder->item?->label === 'ok') {
            self::useInt($holder->item->id);
        }
    }

    private static function useInt(int $value): void {}

    private static function useString(string $value): void {}
}
