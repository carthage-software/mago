<?php

declare(strict_types=1);

interface DenormalizerInterface
{
    /**
     * @template TObject of object
     *
     * @param mixed                        $data
     * @param class-string<TObject>|string $type
     *
     * @return ($type is class-string<TObject> ? TObject : mixed)
     */
    public function denormalize(mixed $data, string $type): mixed;
}

class X {}

function x(DenormalizerInterface $denormalizer) {
    $x = $denormalizer->denormalize('', X::class);

    accept_x($x);
}

function accept_x(X $_) {}
