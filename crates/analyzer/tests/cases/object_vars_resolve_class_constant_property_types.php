<?php

declare(strict_types=1);

class Level
{
    private const HIGH = 3;
    private const LOW = 0;

    /** @var self::HIGH|self::LOW */
    public int $value = 3;
}

class Base
{
    public const A = 1;

    /** @var self::A|2 */
    public int $inherited = 1;
}

final class Child extends Base
{
}

function level_column_has_mid(Level $level): bool
{
    return in_array(1, array_column([$level], 'value'), true);
}

function level_var_has_mid(Level $level): bool
{
    return in_array(1, [get_object_vars($level)['value']], true);
}

function inherited_var_has_two(Child $child): bool
{
    return in_array(2, [get_object_vars($child)['inherited']], true);
}
