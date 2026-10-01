<?php

declare(strict_types=1);

namespace Issue2403;

/** @return array{x: 1, y: 2} */
function initialize(): array
{
    static $bar = ['x' => 1];

    if (!isset($bar['y'])) {
        echo "init\n";
        $bar['y'] = 2;
    }

    return $bar;
}

/** @return array{x: 1, y: 2} */
function documented(): array
{
    /** @var array{x: 1, y?: 2} */
    static $bar = ['x' => 1];

    if (!isset($bar['y'])) {
        $bar['y'] = 2;
    }

    return $bar;
}

function remove(bool $remove): void
{
    static $bar = ['x' => 1];

    if (isset($bar['x'])) {
        takesOne($bar['x']);
    }

    if ($remove) {
        unset($bar['x']);
    }
}

function unchanged(): void
{
    static $bar = ['x' => 1];

    // @mago-expect analysis:redundant-condition
    if (isset($bar['x'])) {
        takesOne($bar['x']);
    }
}

function counter(): int
{
    static $count = 0;

    return ++$count;
}

/** @return array{x: 1, count: int} */
function nestedCounter(): array
{
    static $state = ['x' => 1, 'count' => 0];
    ++$state['count'];

    return $state;
}

/** @return list<1> */
function append(): array
{
    static $values = [];
    $values[] = 1;

    return $values;
}

/** @return non-empty-string */
function stringCounter(): string
{
    static $value = 'a';
    $current = $value;
    $value .= 'b';

    return $current;
}

/** @return 1 */
function growingTree(bool $grow): int
{
    static $constant = 1;
    static $tree = [];

    if ($grow) {
        $tree = ['child' => $tree];
    }

    return $constant;
}

function cached(bool $initialize): ?int
{
    static $cache = [];

    if (isset($cache['value'])) {
        return $cache['value'];
    }

    if ($initialize) {
        $cache['value'] = 2;

        return $cache['value'];
    }

    return null;
}

function unguarded(bool $initialize): ?int
{
    static $cache = [];

    if ($initialize) {
        $cache['value'] = 2;
    }

    // @mago-expect analysis:possibly-undefined-string-array-index
    return $cache['value'];
}

function closureCache(): \Closure
{
    return function (): int {
        static $cache = [];

        if (!isset($cache['value'])) {
            $cache['value'] = 2;
        }

        return $cache['value'];
    };
}

/** @param 1 $_ */
function takesOne(int $_): void {}

initialize();
initialize();
documented();
remove(true);
remove(false);
unchanged();
counter();
nestedCounter();
append();
stringCounter();
growingTree(true);
cached(true);
cached(false);
unguarded(true);
closureCache()();
