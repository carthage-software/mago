<?php

declare(strict_types=1);

/** @param 'first'|'second' $firstOrSecond */
function take_one_of_them(string $firstOrSecond): void
{
    echo $firstOrSecond;
}

$ao = new ArrayObject(['first', 'second']);
[$first, $second] = $ao;
$assoc = new ArrayObject(['a' => 'first', 'b' => 'second']);
['a' => $a, 'b' => $b] = $assoc;
assert($a === $first && $b === $second);

/** @mago-expect analysis:possibly-invalid-argument */
take_one_of_them($first);
/** @mago-expect analysis:possibly-invalid-argument */
take_one_of_them($second);
/** @mago-expect analysis:possibly-invalid-argument */
take_one_of_them($a);
/** @mago-expect analysis:possibly-invalid-argument */
take_one_of_them($b);
