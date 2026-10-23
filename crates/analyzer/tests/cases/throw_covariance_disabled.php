<?php

interface ThrowCovarianceDisabledContract
{
    /** @throws \RuntimeException */
    public function run(): void;
}

final class ThrowCovarianceDisabledImplementation implements ThrowCovarianceDisabledContract
{
    /** @throws \Throwable */
    public function run(): void {}
}

(new ThrowCovarianceDisabledImplementation())->run();
