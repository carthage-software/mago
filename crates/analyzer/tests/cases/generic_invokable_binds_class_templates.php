<?php

declare(strict_types=1);

/** @template T */
abstract class InvokeBase
{
    /**
     * @param T $value
     * @return T
     */
    public function __invoke(mixed $value): mixed
    {
        return $value;
    }
}

/** @extends InvokeBase<stdClass> */
final class InvokeFixed extends InvokeBase {}

/**
 * @template U
 * @extends InvokeBase<list<U>>
 */
final class InvokeComposed extends InvokeBase {}

interface InvokeNamed {}

/** @template T */
final class InvokeBox
{
    /** @param T $value */
    public function __construct(private mixed $value) {}

    /** @return T */
    public function __invoke(string $_key = ''): mixed
    {
        return $this->value;
    }
}

/** @param callable(): int $_callback */
function invoke_takes_int_callback(callable $_callback): void {}

/** @param callable(list<DateTime>): list<DateTime> $_callback */
function invoke_takes_dates_callback(callable $_callback): void {}

/** @param list<int> $_values */
function invoke_takes_ints(array $_values): void {}

function invoke_takes_std(stdClass $_value): void {}

/**
 * @param InvokeBox<int> $box
 * @param InvokeNamed&InvokeBox<int> $named
 * @param InvokeComposed<DateTime> $composed
 * @param list<string> $keys
 */
function invoke_objects(InvokeFixed $fixed, InvokeBox $box, InvokeNamed $named, InvokeComposed $composed, array $keys): int
{
    invoke_takes_std($fixed(new stdClass()));
    $fixed(new DateTime());

    invoke_takes_int_callback($box);
    invoke_takes_int_callback($named);
    invoke_takes_dates_callback($composed);
    invoke_takes_ints(array_map($box, $keys));
    usort($keys, $box);

    return $box() + $named();
}
