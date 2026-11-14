<?php

declare(strict_types=1);

interface Contract
{
    public function extra(): void;

    public function more(int $x): void;

    public function all(int $x, int $y): void;

    public function optional(int $x = 1): void;
}

final class Implementation implements Contract
{
    public function extra(int ...$rest): void {}

    public function more(int $x, string ...$rest): void {}

    public function all(int ...$x): void {}

    public function optional(int ...$x): void {}
}

interface Strict
{
    public function needs(int $x, int $y): void;
}

final class TooFew implements Strict
{
    // @mago-expect analysis:incompatible-parameter-count
    public function needs(int $x): void {}
}
