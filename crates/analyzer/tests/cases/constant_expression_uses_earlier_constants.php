<?php

declare(strict_types=1);

const BASE = 3;
const NEXT = BASE + 1;

define('DEFINED_NEXT', BASE * 2);

enum Level: int
{
    case First = NEXT;
}

/**
 * @return 4
 */
function next_value(): int
{
    return NEXT;
}

/**
 * @return 6
 */
function defined_value(): int
{
    return DEFINED_NEXT;
}

function level_value(): int
{
    return Level::First->value;
}
