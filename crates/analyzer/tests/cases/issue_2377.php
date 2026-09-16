<?php

declare(strict_types=1);

namespace Issue2377;

final readonly class Container
{
    public function __construct(
        public ?string $storedValue,
    ) {}

    /** @pure */
    public function getStoredValue(): ?string
    {
        return $this->storedValue;
    }

    /** @phpstan-assert-if-true !null $this->getStoredValue() */
    public function hasStoredValue(): bool
    {
        return $this->storedValue !== null;
    }

    /** @phpstan-assert-if-false !null $this->getStoredValue() */
    public function lacksStoredValue(): bool
    {
        return $this->storedValue === null;
    }
}

final readonly class Repro
{
    public function invertedGuardThenReturn(Container $holder): string
    {
        if (!$holder->hasStoredValue()) {
            return 'none';
        }

        return $holder->getStoredValue();
    }

    public function invertedGuardThenContinue(Container $holder): string
    {
        foreach ([$holder] as $item) {
            if (!$item->hasStoredValue()) {
                continue;
            }

            return $item->getStoredValue();
        }

        return 'none';
    }

    public function matchOnGuardBoolean(Container $holder): string
    {
        return match ($holder->hasStoredValue()) {
            true => $holder->getStoredValue(),
            false => 'none',
        };
    }

    public function matchOnTrueWithGuardCondition(Container $holder): string
    {
        return match (true) {
            $holder->hasStoredValue() => $holder->getStoredValue(),
            default => 'none',
        };
    }

    public function switchOnGuard(Container $holder): string
    {
        switch ($holder->hasStoredValue()) {
            case true:
                return $holder->getStoredValue();
            default:
                return 'none';
        }
    }

    public function guardResultStoredInVariable(Container $holder): string
    {
        $hasStoredValue = $holder->hasStoredValue();
        if ($hasStoredValue) {
            return $holder->getStoredValue();
        }

        return 'none';
    }

    public function assertIfFalseMatchFalseArm(Container $holder): string
    {
        return match ($holder->lacksStoredValue()) {
            false => $holder->getStoredValue(),
            true => 'none',
        };
    }
}
