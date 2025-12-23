<?php

// Test 1: Blank line breaks alignment run
$a  = 1;
$bb = 2;

$ccc  = 3;
$dddd = 4;

// Test 2: Comment breaks alignment run
$e  = 5;
$ff = 6;
// This comment breaks the run
$ggg  = 7;
$hhhh = 8;

// Test 3: Single item has no padding
$single = 'alone';

// Test 4: Different statement types break run
$i  = 9;
$jj = 10;
echo 'hello';
$kkk  = 11;
$llll = 12;

// Test 5: Mixed constants and variables don't align together
const FOO    = 1;
const BARRRR = 2;
$m  = 3;
$nn = 4;

class RunBreaking
{
    // Test 6: Blank line breaks class member run
    public const A  = 1;
    public const BB = 2;

    public const CCC  = 3;
    public const DDDD = 4;

    // Test 7: Comment breaks class member run
    public const E  = 5;
    public const FF = 6;
    // Comment breaks run
    public const GGG  = 7;
    public const HHHH = 8;

    // Test 8: Different member types break run
    public const I  = 9;
    public const JJ = 10;

    public string $k  = 'k';
    public string $ll = 'll';

    // Test 9: Attributes break run
    public string $m  = 'm';
    public string $nn = 'nn';
    #[Attribute]
    public string $ooo  = 'ooo';
    public string $pppp = 'pppp';
}
