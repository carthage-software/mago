<?php

declare(strict_types=1);

interface I1
{
    public function f(int $x, int &$y): void;
}

final class C1 implements I1
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function f(int ...$x): void {}
}

interface I2
{
    public function g(int &$y): void;
}

final class C2 implements I2
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function g(int $y): void {}
}

interface I
{
    public function f(int $x, int $y): void;
}

final class C implements I
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function f(int &...$x): void {}
}

interface I3
{
    public function h(array &$items, int ...$rest): void;
}

final class C3 implements I3
{
    public function h(array &$items, int ...$rest): void {}
}
