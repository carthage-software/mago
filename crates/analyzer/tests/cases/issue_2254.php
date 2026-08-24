<?php

declare(strict_types=1);

namespace Issue2254;

/** @return numeric-string */
function validateNumber(string $value): string
{
    if ((int) $value != $value || $value > 100) {
        return '0';
    }

    return $value;
}

function compareLiteralValues(): void
{
    $nonNumeric = 'abc';
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ((int) $nonNumeric != $nonNumeric) {
    }

    $integer = '42';
    // @mago-expect analysis:redundant-comparison,impossible-condition
    if ((int) $integer != $integer) {
    }

    $leadingZero = '042';
    // @mago-expect analysis:redundant-comparison,impossible-condition
    if ((int) $leadingZero != $leadingZero) {
    }

    $decimal = '42.0';
    // @mago-expect analysis:redundant-comparison,impossible-condition
    if ((int) $decimal != $decimal) {
    }

    $exponent = '4.2e1';
    // @mago-expect analysis:redundant-comparison,impossible-condition
    if ((int) $exponent != $exponent) {
    }

    $whitespace = ' 42 ';
    // @mago-expect analysis:redundant-comparison,impossible-condition
    if ((int) $whitespace != $whitespace) {
    }

    $fraction = '42.5';
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ((int) $fraction != $fraction) {
    }

    $numericPrefix = '42foo';
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ((int) $numericPrefix != $numericPrefix) {
    }
}
