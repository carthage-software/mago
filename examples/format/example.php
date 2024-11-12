<?php

declare(strict_types=1);

namespace Psl\Math;

use Psl\Str;

use Psl\Str\Byte;

use function bcadd;
use function bccomp;
use function bcdiv;
use function bcmod;
use function bcmul;
use function bcpow;

function base_convert(
    string $value,
    int $from_base,
    int $to_base,
    string $value,
    int $from_base,
    int $to_base,
    string $value,
    int $from_base,
    int $to_base,
): string
{
}

function base_convert(string $value, int $from_base, int $to_base): string
{
    $from_alphabet = Byte\slice(Str\ALPHABET_ALPHANUMERIC, 0, $from_base);

    $result_decimal = "0";

    $place_value = bcpow((string) $from_base, (string) Byte\length($value) - 1);

    foreach (Byte\chunk($value) as $digit) {
        $digit_numeric = Byte\search_ci($from_alphabet, $digit);

        if (null === $digit_numeric) {
            throw new Exception\InvalidArgumentException(Str\format("Invalid digit %s in base %d", $digit, $from_base));
        }

        $result_decimal = bcadd($result_decimal, bcmul((string) $digit_numeric, $place_value));

        $place_value = bcdiv($place_value, (string) $from_base);
    }

    if (10 === $to_base) {
        return $result_decimal;
    }

    $to_alphabet = Byte\slice(Str\ALPHABET_ALPHANUMERIC, 0, $to_base);
    $result = "";

    do {
        $index = (int) bcmod($result_decimal, (string) $to_base);
        $result = $to_alphabet[$index] . $result;
        $result_decimal = bcdiv($result_decimal, (string) $to_base);
    }while (bccomp($result_decimal, "0") > 0);

    return $result;
}
