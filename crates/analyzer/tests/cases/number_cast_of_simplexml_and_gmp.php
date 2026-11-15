<?php

declare(strict_types=1);

final class Element extends SimpleXMLElement
{
}

function attribute_as_float(SimpleXMLElement $xml): float
{
    return (float) $xml['value'];
}

function element_as_float(Element $element): float
{
    return (float) $element;
}

/** @return int<2, max>|int<min, 0> */
function element_as_int(SimpleXMLElement $xml): int
{
    $value = (int) $xml;
    if ($value === 1) {
        return 0;
    }

    return $value;
}

function gmp_as_float(GMP $number): float
{
    return (float) $number;
}
