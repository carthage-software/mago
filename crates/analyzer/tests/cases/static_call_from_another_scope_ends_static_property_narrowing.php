<?php

declare(strict_types=1);

namespace StaticCallFromAnotherScope;

use RuntimeException;

class Recorder
{
    /** @var list<string> */
    public static array $calls = [];

    public static function record(string $call): void
    {
        self::$calls[] = $call;
    }
}

final class ChildRecorder extends Recorder
{
    /**
     * @throws RuntimeException
     */
    public static function check(): void
    {
        assertSame(['a'], self::$calls);
        parent::record('b');
        assertSame(['a', 'b'], self::$calls);
    }
}

final class RecorderTest
{
    /**
     * @throws RuntimeException
     */
    public function check(): void
    {
        assertSame(['a'], Recorder::$calls);
        Recorder::record('b');
        assertSame(['a', 'b'], Recorder::$calls);
    }
}

abstract class Assert
{
    public static function assertTrue(bool $_condition): void
    {
    }
}

final class FixtureTest extends Assert
{
    private static ?Recorder $recorder = null;

    public function check(): Recorder
    {
        self::$recorder ??= new Recorder();
        self::assertTrue(true);

        return self::$recorder;
    }
}

/**
 * @template T
 *
 * @param T $expected
 *
 * @psalm-assert =T $actual
 *
 * @throws RuntimeException
 */
function assertSame(mixed $expected, mixed $actual): void
{
    if ($expected !== $actual) {
        throw new RuntimeException('Not the same');
    }
}

Recorder::record('a');
assertSame(['a'], Recorder::$calls);
Recorder::record('b');
assertSame(['a', 'b'], Recorder::$calls);
