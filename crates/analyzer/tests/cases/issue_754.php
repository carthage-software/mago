<?php

class Monitor
{
}

/** @template TBuiltObject of object */
trait BuilderWithIdentifier
{
    /**
     * @param positive-int $identifier
     *
     * @return static<TBuiltObject>
     */
    abstract public static function forIdentifier(int $identifier): static;

    /**
     * @return TBuiltObject
     */
    abstract public function build(): object;
}

final readonly class MonitorBuilder
{
    /** @use BuilderWithIdentifier<Monitor> */
    use BuilderWithIdentifier;

    /**
     * @param positive-int $identifier
     */
    #[Override]
    public static function forIdentifier(int $identifier): static
    {
        return new self();
    }

    public function build(): Monitor
    {
        return new Monitor();
    }
}
