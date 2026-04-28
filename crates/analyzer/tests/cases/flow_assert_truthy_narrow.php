<?php

declare(strict_types=1);

function flow_assert_truthy_narrow(null|string $v): int
{
    assert($v !== null && $v !== '');

    return strlen($v);
}
