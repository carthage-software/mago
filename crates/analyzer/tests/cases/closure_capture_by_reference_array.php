<?php

declare(strict_types=1);

$seen = [];
$isDuplicate = static function (string $key) use (&$seen): bool {
    if (isset($seen[$key])) {
        return true;
    }

    $seen[$key] = true;
    return false;
};

$isDuplicate('a');
$isDuplicate('a');

/**
 * @param list<string> $path
 */
function print_path(array $path): void
{
    echo implode(' -> ', $path) . "\n";
}

/**
 * The keys written by the closure must stay `string`, they must not widen to `array-key`.
 *
 * @param array<string, list<string>> $edges
 */
function print_cycle(array $edges, string $start): void
{
    $visited = [];
    $stack = [];

    $dfs = static function (string $node) use ($edges, &$visited, &$stack, &$dfs): void {
        $visited[$node] = true;
        $stack[$node] = true;

        foreach ($edges[$node] ?? [] as $neighbor) {
            if (isset($stack[$neighbor])) {
                $path = [];
                foreach ($stack as $name => $_) {
                    $path[] = $name;
                }

                $path[] = $neighbor;
                print_path($path);

                return;
            }

            if (!isset($visited[$neighbor])) {
                $dfs($neighbor);
            }
        }

        unset($stack[$node]);
    };

    $dfs($start);
}

print_cycle(['a' => ['b'], 'b' => ['a']], 'a');

/**
 * The closure only reads the array, the enclosing scope writes to it after the closure is created.
 */
function has_after_outer_write(): bool
{
    $known = [];
    $has = static function (string $key) use (&$known): bool {
        if (isset($known[$key])) {
            return true;
        }

        return false;
    };

    $known['a'] = true;

    return $has('a');
}

has_after_outer_write();
