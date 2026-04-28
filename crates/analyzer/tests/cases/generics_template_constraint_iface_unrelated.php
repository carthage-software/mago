<?php

declare(strict_types=1);

interface GenSerializableI
{
    public function serialize_me(): string;
}

/**
 * @template T of GenSerializableI
 *
 * @param T $val
 */
function gen_take_serializable(GenSerializableI $val): void
{
}

final class GenNotSerializable
{
}

/** @mago-expect analysis:possibly-invalid-argument,template-constraint-violation */
gen_take_serializable(new GenNotSerializable());
