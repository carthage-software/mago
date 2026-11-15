<?php

declare(strict_types=1);

abstract class Constraint
{
    public const CLASS_CONSTRAINT = 'class';
    public const PROPERTY_CONSTRAINT = 'property';

    /** @return self::CLASS_CONSTRAINT|self::PROPERTY_CONSTRAINT|array<self::CLASS_CONSTRAINT|self::PROPERTY_CONSTRAINT> */
    public function getTargets(): string|array
    {
        return self::PROPERTY_CONSTRAINT;
    }
}

final class ClassConstraint extends Constraint
{
    public function getTargets(): string|array
    {
        return self::CLASS_CONSTRAINT;
    }
}

final class BothConstraint extends Constraint
{
    public function getTargets(): string|array
    {
        return [self::CLASS_CONSTRAINT, self::PROPERTY_CONSTRAINT];
    }
}

enum Mode
{
    case Read;
    case Write;
}

abstract class Handler
{
    /** @return Mode::Read|list<Mode::Read>|null */
    public function mode(): Mode|array|null
    {
        return null;
    }

    public const LIMITS = ['low' => 1, 'high' => 2];

    /** @return value-of<self::LIMITS>|list<int> */
    public function limit(): int|array
    {
        return 1;
    }
}

final class ReadHandler extends Handler
{
    public function mode(): Mode|array
    {
        return Mode::Read;
    }

    public function limit(): int|array
    {
        return 2;
    }
}
