<?php

declare(strict_types=1);

class NullValue
{
    /**
     * @template T of mixed
     *
     * @param T|NullValue|null $value
     *
     * @return ($value is NullValue ? null : ($value is null ? null : T))
     */
    public static function nullOrValue(mixed $value): mixed
    {
        return $value instanceof NullValue || $value === null ? null : $value;
    }
}

class Test
{
    public ?string $value = null;
}

/** @var string|NullValue|null $a */
$a = null;
$test = new Test();
$test->value = NullValue::nullOrValue($a);
