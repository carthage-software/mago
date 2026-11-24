<?php

declare(strict_types=1);

final class CircPair
{
    public const A = self::B;
    public const B = self::A;
    public const C = self::C;
    public const D = self::E + 1;
    public const E = self::D + 1;
    public const F = [self::F];
    public const G = self::H['x'];
    public const H = ['x' => self::G];
    public const I = CircOther::J;
}

final class CircOther
{
    public const J = CircPair::I . 'x';
}

enum CircLevel: int
{
    case Low = CircLimits::MIN + 1;
    case High = self::High->value;
}

final class CircLimits
{
    public const MIN = CircLevel::Low->value;
}

class CircInheritedParent
{
    public const A = CircInheritedOther::B;
}

class CircInheritedOther
{
    public const B = CircInheritedChild::A;
}

class CircInheritedChild extends CircInheritedParent {}


interface CircOkInterface
{
    public const BASE = 'base';
}

class CircOkParent implements CircOkInterface
{
    public const A = self::BASE . '-a';
    public const B = [self::A, self::BASE];
}

final class CircOkChild extends CircOkParent
{
    public const C = parent::A . self::BASE;
    public const D = self::C . CircOkOther::E;
    public const F = CircOkLevel::One->value + CircOkLevel::Two->value;
    public const G = CircOkLevel::One->name;
}

final class CircOkOther
{
    public const E = CircOkChild::A;
}

enum CircOkLevel: int
{
    case One = 1;
    case Two = self::One->value + 1;

    public const Max = self::Two;
}

final class CircOkFlags
{
    public const READ = 1;
    public const WRITE = self::READ << 1;
    public const EXEC = self::WRITE << 1;
    public const ALL = self::READ | self::WRITE | self::EXEC;
}

