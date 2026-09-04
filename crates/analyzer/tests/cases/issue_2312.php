<?php

declare(strict_types=1);

namespace Issue2312;

final readonly class Scalar
{
    public static function fromFloat(float $value): self
    {
        return new self($value);
    }

    private function __construct(public float $value) {}
}

final readonly class Repro
{
    public function directIf(?float $value): ?Scalar
    {
        $hasValue = $value !== null;

        if ($hasValue) {
            return Scalar::fromFloat($value);
        }

        return null;
    }

    public function directComparison(?float $value): ?Scalar
    {
        $hasValue = $value !== null;

        if ($hasValue === true) {
            return Scalar::fromFloat($value);
        }

        return null;
    }

    public function elseif(?float $value, bool $skip): ?Scalar
    {
        $hasValue = $value !== null;

        if ($skip) {
            return null;
        } elseif ($hasValue) {
            return Scalar::fromFloat($value);
        }

        return null;
    }

    public function matchDiscriminant(?float $value): ?Scalar
    {
        $hasValue = $value !== null;

        return match ($hasValue) {
            true => Scalar::fromFloat($value),
            false => null,
        };
    }

    public function matchArm(?float $value): ?Scalar
    {
        $hasValue = $value !== null;

        return match (true) {
            $hasValue => Scalar::fromFloat($value),
            default => null,
        };
    }
}
