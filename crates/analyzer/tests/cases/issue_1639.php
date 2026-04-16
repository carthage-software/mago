<?php

declare(strict_types=1);

function test(): void
{
    /** @var array<int,array<string,array<int<0,3>,int>>> $info */
    $info = [];
    while ($k = mt_rand(0, max: 10)) {
        if (!array_key_exists($k, $info)) {
            $info[$k] = ['s' => [0, 0, 0, 0]];
        }

        $v = mt_rand(0, max: 3);
        /**
         * @mago-expect analysis:mismatched-array-index
         * @mago-expect analysis:invalid-operand
         */
        $info[$k][$v]++;
    }
}

/**
 * @return array{'str': 'hello', 0: 1}
 */
function test2(): array
{
    $info = ['str' => 'hello'];
    $info[] = 1;

    return $info;
}

/**
 * @return array{'str': 'hello', 5: 1}
 */
function test3(): array
{
    $info = ['str' => 'hello'];
    $info[5] = 1;

    return $info;
}

/**
 * @return array{'str': 'hello', ...<int, 1>}
 */
function test4(int $k): array
{
    $info = ['str' => 'hello'];
    $info[$k] = 1;

    return $info;
}

/**
 * @return array{'str': 'hello', ...<int|string, 1|'other'>}
 */
function test5(string $v, int $k): array
{
    $info = ['str' => 'hello', $v => 'other'];
    $info[$k] = 1;

    return $info;
}

/**
 * @return list{array{'str': 'hello', 0: 1}}
 */
function test22(): array
{
    $info = [['str' => 'hello']];
    $info[0][] = 1;

    return $info;
}

/**
 * @return list{array{'str': 'hello', 5: 1}}
 */
function test32(): array
{
    $info = [['str' => 'hello']];
    $info[0][5] = 1;

    return $info;
}

/**
 * @return list{array{'str': 'hello', ...<int, 1>}}
 */
function test42(int $k): array
{
    $info = [['str' => 'hello']];
    $info[0][$k] = 1;

    return $info;
}

/**
 * @return list{array{'str': 'hello', ...<int|string, 1|'other'>}}
 */
function test52(string $v, int $k): array
{
    $info = [['str' => 'hello', $v => 'other']];
    $info[0][$k] = 1;

    return $info;
}

/**
 * @param list<array{string, string}> $packages
 * @return non-empty-list<non-empty-string>
 */
function test62(array $packages): array
{
    $repos = array_map(
        /**
         * @param array{string, string} $package
         * @return non-empty-string
         */
        fn(array $package): string => $package[0] . '/' . $package[1],
        $packages,
    );

    $repos[] = 'foo/main';

    foreach ($repos as $repo) {
        if ($repo === 'foo/main') {
            echo 'this is main repo';
        } else {
            echo 'this is another repo';
        }
    }

    return $repos;
}
