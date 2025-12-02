<?php

/**
 * @template T
 *
 * @mago-expect analysis:unused-template-parameter
 */
function unused_template(): void
{
}

/**
 * @template T
 *
 * @param T $value
 */
function used_in_param(mixed $value): void
{
}

/**
 * @template T
 *
 * @return T
 */
function used_in_return(): mixed
{
    return null;
}

/**
 * @template T
 * @template U
 *
 * @param T $value
 *
 * @mago-expect analysis:unused-template-parameter
 */
function one_used_one_not(mixed $value): void
{
}

/**
 * @template _T
 */
function underscore_prefix_is_allowed(): void
{
}

class Example
{
    /**
     * @template T
     *
     * @mago-expect analysis:unused-template-parameter
     */
    public function unused_method_template(): void
    {
    }

    /**
     * @template T
     *
     * @param T $value
     */
    public function used_in_method_param(mixed $value): void
    {
    }

    /**
     * @template T
     *
     * @return T
     */
    public function used_in_method_return(): mixed
    {
        return null;
    }

    /**
     * @template T
     * @template U
     *
     * @param T $first
     * @param U $second
     */
    public function both_used_in_method(mixed $first, mixed $second): void
    {
    }

    /**
     * @template _Unused
     */
    public function underscore_method_template(): void
    {
    }
}
