<?php

declare(strict_types=1);

namespace Psl\Math;

use Psl\Str;

use Psl\Str\Byte;

function from_base(string $number, int $from_base): int
{
    $limit = div(INT64_MAX, $from_base);

    $result = 0;

    foreach (Byte\chunk($number) as $digit) {
        $oval = Byte\ord($digit);

        if (($oval <= 57) && ($oval >= 48)) {
            $dval = $oval - 48;
        } else if (($oval >= 97) && ($oval <= 122)) {
            $dval = $oval - 87;
        } else if (($oval >= 65) && ($oval <= 90)) {
            $dval = $oval - 55;
        } else {
            $dval = 99;
        }

        if ($from_base < $dval) {
            throw new Exception\InvalidArgumentException(Str\format("Invalid digit %s in base %d", $digit, $from_base));
        }

        $oldval = $result;

        $result = $from_base * $result + $dval;

        if (($oldval > $limit) || ($oldval > $result)) {
            throw new Exception\OverflowException(
                Str\format("Unexpected integer overflow parsing %s from base %d", $number, $from_base),
            );
        }
    }

    return $result;
}
