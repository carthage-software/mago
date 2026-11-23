<?php

declare(strict_types=1);

function keyed_assignment(): void
{
    $missing['b'] = 1;
}

function nested_assignment(): void
{
    $missing['x']['y'] = 1;
}

function append_assignment(): void
{
    $missing[] = 1;
}

function compound_assignment(): void
{
    // @mago-expect analysis:undefined-variable,mixed-array-access,mixed-operand
    $missing['x'] .= 'v';
}
