<?php

declare(strict_types=1);

function string_loop(?string $value): void
{
    while ($value != null) { // @mago-expect analysis:possibly-null-operand,null-operand
        echo $value;
        break;
    }
}

function int_loop(?int $value): void
{
    while (null != $value) { // @mago-expect analysis:possibly-null-operand,null-operand
        echo $value;
        break;
    }
}
