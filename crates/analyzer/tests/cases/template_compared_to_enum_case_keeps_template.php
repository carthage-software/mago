<?php

declare(strict_types=1);

enum TemplateComparedSuit
{
    case Hearts;
    case Spades;
}

/**
 * @template T of TemplateComparedSuit
 *
 * @param T $suit
 *
 * @return T
 */
function template_compared_to_enum_case(TemplateComparedSuit $suit): TemplateComparedSuit
{
    if ($suit === TemplateComparedSuit::Hearts) {
        return $suit;
    }

    return $suit;
}

/**
 * @template T of UnitEnum
 *
 * @param T $value
 *
 * @return T
 */
function template_of_interface_compared_to_enum_case(UnitEnum $value): UnitEnum
{
    if ($value === TemplateComparedSuit::Spades) {
        return $value;
    }

    return $value;
}
