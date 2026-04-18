<?php

/**
 * Regression: analyzing this function used to trip the
 * `combine() received an empty Vec` debug assertion inside
 * `mago_codex::ttype::get_array_parameters` for the `Keyed` arm,
 * whenever the outer array shape had a sibling whose value was the
 * empty keyed-array being assigned into from inside a loop.
 *
 * The expect pragmas below guard the panic only; the specific issue
 * codes raised are incidental.
 *
 * @mago-expect analysis:mixed-assignment
 * @mago-expect analysis:invalid-array-access
 */
function reproducer(array $headers): void
{
    $parameters = ['attrs' => []];

    foreach ($headers as $name => $value) {
        if ($name[0]) {
            $parameters['attrs'][$name] = $value;
        }
    }
}
