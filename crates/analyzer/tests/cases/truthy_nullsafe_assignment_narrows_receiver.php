<?php

declare(strict_types=1);

final class Box
{
    public function get(): ?string
    {
        return null;
    }

    public function name(): string
    {
        return '';
    }
}

function assigned(?Box $box): string
{
    if ($value = $box?->get()) {
        return $value . $box->name();
    }

    return '';
}

function negated(?Box $box): string
{
    if (!($value = $box?->get())) {
        return '';
    }

    return $value . $box->name();
}

function ternary(?Box $box): string
{
    return ($value = $box?->get()) ? $value . $box->name() : '';
}

function conjunction(?Box $box): string
{
    if (($value = $box?->get()) && $box->name() !== '') {
        return $value;
    }

    return '';
}

function compared(?Box $box): string
{
    if (($value = $box?->get()) !== null) {
        return $value . $box->name();
    }

    return '';
}
