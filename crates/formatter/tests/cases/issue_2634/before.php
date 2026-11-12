<?php

final class Example
{
    private const string DATE_NOW = '2025-07-24';
    private const int TRAVELLER_ADULT_COUNT = 2;

    public const array|null REWRITTEN = null;
    public const string NAME = 'x';
    public const int | string SPACED = 1;

    protected const \Stringable&\Countable INTERSECTION = OTHER;
    protected const (\Stringable&\Countable)|null DNF = null;
    protected const bool FLAG = true;

    const string SAME_WIDTH = 'a';
    const object OTHER_NAME = B;

    const UNTYPED = 1;
    const LONGER_UNTYPED = 2;
    const string TYPED_AFTER_UNTYPED = 'x';
    const int COUNT = 3;

    private int|null $rewritten = 1;
    private string $name = 'test';
    private int | string $spaced = 1;

    public $untyped = 1;
    public $longerUntyped = 2;
    public string $typedAfterUntyped = 'x';
    public int $count = 3;
}

interface Contract
{
    public const string NAME = 'x';
    public const int VERSION_NUMBER = 4;
}

enum Suit: string
{
    case Hearts = 'h';
    case Spades = 's';

    const string DEFAULT_NAME = 'x';
    const int COUNT = 4;
}

trait Shared
{
    final public const float RATIO = 1.5;
    final public const int LIMIT_VALUE = 10;
}
