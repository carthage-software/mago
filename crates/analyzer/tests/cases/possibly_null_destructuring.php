<?php

declare(strict_types=1);

function take_int(int $value): void
{
    echo $value;
}

function take_string(string $value): void
{
    echo $value;
}

/**
 * @param array{int, string}|null $a
 */
function list_destructuring(?array $a): void
{
    /** @mago-expect analysis:possibly-null-destructuring-source */
    [$i, $s] = $a;

    take_int($i ?? 0);
    take_string($s ?? '');
}

/**
 * @param array{id: int, name: string}|null $a
 */
function keyed_destructuring(?array $a): void
{
    /** @mago-expect analysis:possibly-null-destructuring-source */
    ['id' => $id, 'name' => $name] = $a;

    take_int($id ?? 0);
    take_string($name ?? '');
}

/**
 * @param int|null $a
 */
function non_array_source(?int $a): void
{
    /**
     * @mago-expect analysis:invalid-destructuring-source
     * @mago-expect analysis:mixed-assignment
     */
    [$x] = $a;

    unset($x);
}

function null_source(): void
{
    $a = null;

    /** @mago-expect analysis:null-destructuring-source */
    [$x, $y] = $a;

    unset($x, $y);
}
