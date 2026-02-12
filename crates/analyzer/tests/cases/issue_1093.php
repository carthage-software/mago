<?php

declare(strict_types=1);

/**
 * @param array{valid: true, result: string}|array{valid: false, errorCode: string} $input
 *
 * @return array{valid: true, result: string}
 */
function test_array_key_narrowing(array $input)
{
    if ($input['valid'] === true) {
        return $input;
    } else {
        return ['result' => $input['errorCode'], 'valid' => true];
    }
}

interface MyInterface1
{
    public string $myProp { get; }
}

interface MyInterface2
{
    public float $myProp { get; }
}

function test_property_narrowing(MyInterface1|MyInterface2 $input): MyInterface1
{
    if (is_string($input->myProp)) {
        return $input;
    } else {
        die();
    }
}
