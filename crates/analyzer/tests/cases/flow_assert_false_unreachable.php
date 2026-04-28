<?php

declare(strict_types=1);

function flow_assert_false_unreachable(): never
{
    assert(false);

    while (true) {
    }
}
