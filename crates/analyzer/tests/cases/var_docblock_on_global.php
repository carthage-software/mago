<?php

declare(strict_types=1);

class GlobalValue
{
    public function value(): string
    {
        return '';
    }
}

function named_var(): string
{
    /** @var GlobalValue $value */
    global $value;

    return $value->value();
}

function unnamed_var(): string
{
    /** @var GlobalValue */
    global $value;

    return $value->value();
}

function multiple_vars(): string
{
    /**
     * @var GlobalValue $first
     * @var GlobalValue $second
     */
    global $first, $second;

    return $first->value() . $second->value();
}

function constrains_assignments(): void
{
    /** @var GlobalValue $value */
    global $value;

    /** @mago-expect analysis:reference-constraint-violation */
    $value = 'not an object';
}

function narrows_superglobal(): string
{
    /** @var array<string, string> $_SERVER */
    global $_SERVER;

    return $_SERVER['REQUEST_URI'] ?? '';
}

function without_var(): string
{
    global $value;

    /**
     * @mago-expect analysis:mixed-method-access
     * @mago-expect analysis:mixed-return-statement
     */
    return $value->value();
}
