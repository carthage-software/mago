<?php

function test_concat_with_union_types(): void
{
    $types = ["option_1", "option_2"];

    foreach ($types as $type) {
        $result = 'Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type . '
            Test ' . $type;
    }
}

function test_concat_with_coalesced_nullable(?string $a, ?string $b, ?string $c, ?string $d, ?string $e): string
{
    return 'Values: ' . ($a ?? 'default') . ', ' . ($b ?? 'default') . ', ' . ($c ?? 'default') . ', ' . ($d ?? 'default') . ', ' . ($e ?? 'default');
}

function test_small_concat_works(): string
{
    /** @var 'a'|'b' $type */
    $type = 'a';
    return 'prefix_' . $type . '_suffix';
}
