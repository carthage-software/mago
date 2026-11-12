<?php

declare(strict_types=1);

namespace Issue2625;

/**
 * @return never
 * @throws \RuntimeException
 */
function fail(): void
{
    throw new \RuntimeException('stop');
}

/** @throws \RuntimeException */
function functionCall(bool $c): int
{
    if ($c) {
        fail();
    } else {
        $a = 1;
    }

    return $a;
}

final class Failure
{
    /** @return never */
    public static function stop(): void
    {
        exit;
    }

    /** @return never */
    public function abort(): void
    {
        exit;
    }
}

function staticMethodCall(bool $c): int
{
    if ($c) {
        Failure::stop();
    } else {
        $a = 1;
    }

    return $a;
}

function instanceMethodCall(bool $c, Failure $failure): int
{
    if ($c) {
        $failure->abort();
    } else {
        $a = 1;
    }

    return $a;
}

/** @return never */
function objectReturn(): object
{
    exit;
}

function objectReturnCall(bool $c): int
{
    if ($c) {
        objectReturn();
    } else {
        $a = 1;
    }

    return $a;
}

/** @return never */
function arrayReturn(): array
{
    exit;
}

function arrayReturnCall(bool $c): int
{
    if ($c) {
        arrayReturn();
    } else {
        $a = 1;
    }

    return $a;
}

/** @return never */
function untypedReturn()
{
    exit;
}

function untypedReturnCall(bool $c): int
{
    if ($c) {
        untypedReturn();
    } else {
        $a = 1;
    }

    return $a;
}

function nativeNever(): never
{
    exit;
}

function nativeNeverCall(bool $c): int
{
    if ($c) {
        nativeNever();
    } else {
        $a = 1;
    }

    return $a;
}

function plainVoid(): void {}

function plainVoidCall(bool $c): int
{
    if ($c) {
        plainVoid();
    } else {
        $a = 1;
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $a;
}

/**
 * @return never
 * @mago-expect analysis:missing-return-statement
 */
function fallsThrough(): void {}

/**
 * @return never
 * @mago-expect analysis:missing-return-statement
 */
function sometimesReturns(bool $c): void
{
    if ($c) {
        exit;
    }
}

/** @return never */
function explicitReturn(): void
{
    // @mago-expect analysis:invalid-return-statement
    return;
}
