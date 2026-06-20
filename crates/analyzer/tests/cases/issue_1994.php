<?php

declare(strict_types=1);

final class Repro
{
    private ?string $a = null;
    private ?string $b = null;

    public function noop(): void {}

    public function takes(string $v): void {}

    public function trigger(): void
    {
        if (null === $this->a || null === $this->b) {
            $this->noop();
        }

        /** @mago-expect analysis:possibly-null-argument */
        $this->takes($this->a);

        /** @mago-expect analysis:possibly-null-argument */
        $this->takes($this->b);
    }
}
