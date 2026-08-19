<?php

declare(strict_types=1);

function keyed_assignment(): void
{
    // @mago-expect analysis:undefined-variable,mixed-array-assignment
    $missing['b'] = 1;
}

function nested_assignment(): void
{
    // @mago-expect analysis:undefined-variable,mixed-array-assignment,mixed-array-assignment
    $missing['x']['y'] = 1;
}

function append_assignment(): void
{
    // @mago-expect analysis:undefined-variable,mixed-array-assignment
    $missing[] = 1;
}

function compound_assignment(): void
{
    // @mago-expect analysis:undefined-variable,mixed-array-access,mixed-array-assignment,mixed-operand
    $missing['x'] .= 'v';
}
