<?php

declare(strict_types=1);

/**
 * @param object{meta: string}|null $obj
 */
function suffix_null(?object $obj): string
{
    return $obj?->meta ?? '';
}

/**
 * @param null|object{meta: string} $obj
 */
function prefix_null(?object $obj): string
{
    return $obj?->meta ?? '';
}

/**
 * @param ?object{meta: string} $obj
 */
function shorthand_null(?object $obj): string
{
    return $obj?->meta ?? '';
}

function var_annotation(?object $obj): string
{
    /** @var object{meta: string}|null $obj */
    return $obj?->meta ?? '';
}
