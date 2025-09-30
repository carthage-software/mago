<?php

declare(strict_types=1);

/**
 * @method static Money USD(numeric-string|int $amount)
 */
trait MoneyFactory
{
    public static function __callStatic($name, $arguments): Money
    {
        return new Money();
    }
}

final class Money
{
    use MoneyFactory;
}

$money = Money::USD('100');
