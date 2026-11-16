<?php

declare(strict_types=1);

interface Codec
{
}

final class JsonCodec implements Codec
{
}

final class Registry
{
    /** @var array<class-string, Codec> */
    private array $codecs;

    /** @param iterable<class-string, Codec> $codecs */
    public function __construct(iterable $codecs = [])
    {
        if (! is_array($codecs)) {
            $codecs = iterator_to_array($codecs);
        }

        $this->codecs = $codecs + [JsonCodec::class => new JsonCodec()];
    }

    public function get(string $name): ?Codec
    {
        return $this->codecs[$name] ?? null;
    }
}
