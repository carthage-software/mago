<?php

declare(strict_types=1);

final class EmptyObject {}

final class Defaults
{
    public int $count = 0;
}

class LoudObject
{
    public function __construct()
    {
        echo 'created';
    }
}

final class LoudChild extends LoudObject {}

class CleanupObject
{
    public function __destruct()
    {
        echo 'destroyed';
    }
}

final class CleanupChild extends CleanupObject {}

trait Cleanup
{
    public function __destruct()
    {
        echo 'destroyed';
    }
}

final class CleanupFromTrait
{
    use Cleanup;
}

final class ValidatedObject
{
    /**
     * @pure
     *
     * @throws InvalidArgumentException
     */
    public function __construct(int $value)
    {
        if ($value < 0) {
            throw new InvalidArgumentException('negative');
        }
    }
}

final class Point
{
    /** @psalm-external-mutation-free */
    public function __construct(
        public int $x,
        public int $y,
    ) {}
}

abstract class AbstractObject {}

function log_message(): string
{
    echo 'logging';

    return 'message';
}

/** @mago-expect analysis:unused-statement */
new EmptyObject();
/** @mago-expect analysis:unused-statement */
new Defaults();
/** @mago-expect analysis:unused-statement */
new stdClass();
/** @mago-expect analysis:unused-statement */
new Point(1, 2);
/** @mago-expect analysis:unused-statement */
new InvalidArgumentException('forgot to throw');
/** @mago-expect analysis:unused-statement */
new class {};

new LoudObject();
new LoudChild();
new CleanupObject();
new CleanupChild();
new CleanupFromTrait();
new ValidatedObject(1);
new InvalidArgumentException(log_message());
new ArrayObject();
new class {
    public function __construct()
    {
        echo 'created';
    }
};
new class extends CleanupObject {};

/** @mago-expect analysis:abstract-instantiation */
new AbstractObject();
