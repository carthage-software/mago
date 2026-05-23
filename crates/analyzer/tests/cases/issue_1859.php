<?php

declare(strict_types=1);

/**
 * @template T
 */
abstract class Issue1859Column
{
    /** @var T */
    public mixed $default;

    /** @param T $default */
    public function __construct(mixed $default)
    {
        $this->default = $default;
    }
}

/** @extends Issue1859Column<int> */
final class Issue1859IntColumn extends Issue1859Column {}

/** @extends Issue1859Column<string> */
final class Issue1859StringColumn extends Issue1859Column {}

final class Issue1859Builder
{
    /**
     * @template T
     *
     * @param Issue1859Column<T> $column
     * @param T $value
     */
    public function where(Issue1859Column $column, mixed $value): void
    {
        $_ = [$column, $value];
    }
}

$builder = new Issue1859Builder();
$builder->where(new Issue1859IntColumn(0), 100);
$builder->where(new Issue1859StringColumn(''), 'ok');

/** @mago-expect analysis:invalid-argument */
$builder->where(new Issue1859IntColumn(0), 'bad');

/** @mago-expect analysis:invalid-argument */
$builder->where(new Issue1859StringColumn(''), 3);
