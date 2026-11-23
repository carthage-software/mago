<?php

declare(strict_types=1);

interface Marker {}

class Holder
{
    public ?int $v = null;

    /** @psalm-assert int $this->v */
    public function set(int $v): void
    {
        $this->v = $v;
    }

    /** @psalm-assert-if-true Marker $this */
    public function isMarker(): bool
    {
        return $this instanceof Marker;
    }

    /** @psalm-assert-if-false int $this->v */
    public function hasNoValue(): bool
    {
        return $this->v === null;
    }
}

class Child extends Holder
{
    public function via_parent(): int
    {
        parent::set(1);

        return $this->v;
    }

    public function via_self(): int
    {
        self::set(1);

        return $this->v;
    }

    public function via_condition(): int
    {
        if (!self::hasNoValue()) {
            return $this->v;
        }

        return 0;
    }

    public function parent_marker(): ?Marker
    {
        if (parent::isMarker()) {
            return $this;
        }

        return null;
    }

    public function static_marker(): ?Marker
    {
        if (static::isMarker()) {
            return $this;
        }

        return null;
    }
}

interface Logger
{
    public function log(string $m): void;
}

final class KeepsNarrowedPropertyAcrossStaticCalls
{
    public function __construct(
        private ?Logger $logger = null,
    ) {}

    public function on(string $e): void
    {
        if (null === $this->logger) {
            return;
        }

        if (!$s = self::input($e)) {
            $this->logger->log('none');

            return;
        }

        $this->logger->log($s);
    }

    private static function input(string $e): ?string
    {
        return $e === '' ? null : $e;
    }
}
