<?php

declare(strict_types=1);

function flow_switch_basic_narrow(int|string $v): string
{
    switch (true) {
        case is_int($v):
            return (string) $v;
        case is_string($v):
            return $v;
    }

    return '';
}
