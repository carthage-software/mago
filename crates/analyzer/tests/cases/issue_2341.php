<?php

declare(strict_types=1);

/** @param 'a' $_ */
function expectA(string $_): void {}

/** @param 'b' $_ */
function expectB(string $_): void {}

/** @param null $_ */
function expectNull(mixed $_): void {}

/**
 * @param list<string> $values
 *
 * @return array{0: string|null, 1: string|null, ...<int, string|null>}
 */
function padAtLeastTwo(array $values): array
{
    return array_pad($values, 2, null);
}

[$a, $b] = array_pad(['a'], 2, null);
expectA($a);
expectNull($b);

[$c, $d] = array_pad(['a', 'b'], 2, null);
expectA($c);
expectB($d);

[$e, $f] = array_pad(['a'], -2, null);
expectNull($e);
expectA($f);

[$g, $h] = array_pad(['a', 'b'], 1, null);
expectA($g);
expectB($h);
