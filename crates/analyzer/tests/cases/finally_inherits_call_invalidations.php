<?php

namespace Fixture;

final class Counter
{
    /** @var array<string, int<0, max>> */
    private array $ongoing = [];

    /** @param positive-int $limit */
    public function __construct(private readonly int $limit) {}

    public function tick(string $key): void
    {
        $this->ongoing[$key] ??= 0;
        if ($this->ongoing[$key] !== $this->limit) {
            return;
        }

        $this->ongoing[$key]++;
        try {
            $this->work();
        } finally {
            /** @mago-expect analysis:invalid-property-assignment-value */
            $this->ongoing[$key]--;

            if (0 === $this->ongoing[$key]) {
                unset($this->ongoing[$key]);
            }
        }
    }

    private function work(): void
    {
        // Could mutate $this->ongoing.
    }
}
