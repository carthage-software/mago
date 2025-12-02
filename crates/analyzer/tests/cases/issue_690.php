<?php

declare(strict_types=1);

class Cons
{
    public const string TYPE_PURCHASE = 'I';
    public const string TYPE_SALES = 'V';
}

/**
 * @param Cons::TYPE_* $out
 **/
function takes(string $out)
{
    echo $out;
}

/**
 * @var string
 */
$x = 'V';

// This works
if ($x === Cons::TYPE_PURCHASE || $x === Cons::TYPE_SALES) {
    takes(out: $x);
}

// But somehow this doesn't ?, only different is yoda style
if (Cons::TYPE_PURCHASE === $x || Cons::TYPE_SALES === $x) {
    takes(out: $x);
}
