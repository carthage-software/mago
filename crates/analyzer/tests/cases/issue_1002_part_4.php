<?php

declare(strict_types=1);

interface A {
    public function a(): void;
}

interface B {
    public function b(): void;
}

enum C implements A, B {
    case X;

    public function a(): void {}
    public function b(): void {}
}

function x(B&A&UnitEnum $c): void {
    $c::cases();
}

x(C::X);
