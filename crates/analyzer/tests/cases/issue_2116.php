<?php

declare(strict_types=1);

enum C: int
{
    case A = 1;
    case B = 2;
}

final class Label implements Stringable
{
    public function __toString(): string
    {
        return 'label';
    }
}

function rejects_enum_values_in_first_array(): array
{
    // @mago-expect analysis:template-constraint-violation
    // @mago-expect analysis:template-constraint-violation
    // @mago-expect analysis:possibly-invalid-argument
    // @mago-expect analysis:possibly-invalid-argument
    return array_diff([C::A, C::B], [C::B]);
}

function rejects_enum_values_in_variadic_array(): array
{
    // @mago-expect analysis:template-constraint-violation
    // @mago-expect analysis:invalid-argument
    return array_diff(['label'], [C::A]);
}

function accepts_stringable_and_null(): array
{
    return array_diff([null, new Label()], ['label']);
}

function accepts_different_scalar_types(): array
{
    return array_diff([1], ['1', true]);
}

function accepts_resource(): array
{
    $stream = fopen('php://memory', 'r');
    if (!is_resource($stream)) {
        return [];
    }

    return array_diff([$stream], []);
}
