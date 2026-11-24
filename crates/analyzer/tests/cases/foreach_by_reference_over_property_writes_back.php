<?php

declare(strict_types=1);

final class ForeachRefItem
{
    public int $count = 0;
}

final class ForeachRefHolder
{
    /** @var list<int> */
    public array $ints = [];

    /** @var array{k: list<int>} */
    public static array $nested = ['k' => []];

    /** @var array<string, ForeachRefItem> */
    private array $items = [];

    /** @var list<array{name: string, tags?: list<string>}> */
    private array $rows = [];

    /** @var array<string, int|string> */
    private array $mixed = [];

    /** @param list<string> $strings */
    public function valid(array $strings): array
    {
        foreach ($this->ints as &$v) {
            $v = $v + 1;
        }
        unset($v);

        foreach ($strings as &$s) {
            $s = 1;
        }
        unset($s);

        foreach ($this->items as &$item) {
            $item->count++;
        }
        unset($item);

        foreach ($this->rows as &$row) {
            $row['tags'] = ['a'];
        }
        unset($row);

        foreach ($this->mixed as &$value) {
            if (is_int($value)) {
                $value = (string) $value;
            }
        }
        unset($value);

        foreach (self::$nested['k'] as &$n) {
            $n *= 2;
        }
        unset($n);

        return $strings;
    }

    public function invalid(): void
    {
        foreach ($this->ints as &$v) {
            $v = 's';
        }
        unset($v);

        foreach (self::$nested['k'] as &$x) {
            if ($x > 3) {
                $x = 's';
            }
        }
        unset($x);
    }
}

final class ForeachRefWide
{
    /** @var list<ForeachRefItem|int> */
    public array $types = [];
}

final class ForeachRefNarrow
{
    /** @var list<ForeachRefItem> */
    public array $types = [];
}

function foreach_ref_keep(ForeachRefItem|int $type): ForeachRefItem|int
{
    return $type;
}

function foreach_ref_over_either_class(ForeachRefWide|ForeachRefNarrow $node): void
{
    foreach ($node->types as &$type) {
        $type = foreach_ref_keep($type);
    }
    unset($type);
}
