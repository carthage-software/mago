<?php

class Example
{
    // Same modifiers should align
    public int $a = 1;
    public string $bbbbb = 'value';
    public bool $cc = true;

    // Different visibility breaks alignment
    public int $x = 1;
    private string $y = 'test';

    // Same modifiers again
    private int $foo = 1;
    private string $bar = 2;

    // readonly breaks alignment
    public int $r1 = 1;
    public readonly int $r2 = 2;

    // final breaks alignment
    protected int $f1 = 1;
    protected final int $f2 = 2;

    // static breaks alignment
    public int $s1 = 1;
    public static int $s2 = 2;

    // Multiple modifiers - same should align
    public readonly int $m1 = 1;
    public readonly string $m2 = 2;
}

class Constants
{
    // Same modifiers should align
    public const A = 1;
    public const BBB = 2;
    public const CC = 3;

    // Different visibility breaks alignment
    public const X = 1;
    private const Y = 2;

    // Same visibility again
    private const FOO = 1;
    private const BARR = 2;

    // final breaks alignment
    public const F1 = 1;
    final public const F2 = 2;
}
