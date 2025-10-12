<?php

/**
 * @template T of string|int|float
 */
interface Box
{
    /**
     * @param T $value
     */
    public function setValue(string|int|float $value): void;

    /**
     * @return T|null
     */
    public function getValue(): string|int|float|null;
}

/**
 * @extends Box<string>
 */
interface StringBox extends Box
{
}

function use_string_box(StringBox $box): null|string
{
    $box->setValue('Hello');
    return $box->getValue();
}
