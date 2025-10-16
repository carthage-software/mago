<?php

/**
 * @param null $value
 */
function expectsNull($value): void
{
}

function expectsString(string $value): void
{
}

/**
 * @param array{bar?: string} $value
 * @return void
 */
function expectsArray(array $value): void
{
}

/** @var array{foo?: string} $x */
$x = [];

// Test 1: Direct negated isset check
// When !isset($x['foo']) is true, $x['foo'] should be null
if (!isset($x['foo'])) {
    /**
     * @mago-expect analysis:possibly-undefined-string-array-index
     * @mago-expect analysis:possibly-invalid-argument
     */
    expectsNull($x['foo']);
}

// Test 2: Negated isset check via else branch
// The else branch uses negation internally as well
if (isset($x['foo'])) {
    expectsString($x['foo']);
} else {
    /**
     * @mago-expect analysis:possibly-undefined-string-array-index
     * @mago-expect analysis:possibly-invalid-argument
     */
    expectsNull($x['foo']);
}

// Test 3: Nested array access with negated isset
/** @var array{foo?: array {bar?: string}} $y */
$y = [];

if (!isset($y['foo']['bar'])) {
    /**
     * @mago-expect analysis:possibly-undefined-string-array-index
     * @mago-expect analysis:possibly-undefined-string-array-index
     * @mago-expect analysis:possibly-null-array-access
     * @mago-expect analysis:possibly-invalid-argument
     */
    expectsNull($y['foo']['bar']);
} else {
    expectsArray($y['foo']);
}

if (isset($y['foo']['bar'])) {
    expectsString($y['foo']['bar']);
} else {
    /**
     * @mago-expect analysis:possibly-undefined-string-array-index
     * @mago-expect analysis:possibly-undefined-string-array-index
     * @mago-expect analysis:possibly-null-array-access
     * @mago-expect analysis:possibly-invalid-argument
     */
    expectsNull($y['foo']['bar']);
}

// Test 4: Positive isset for comparison - should correctly narrow to string
/** @var array{qux?: string} $w */
$w = [];

if (isset($w['qux'])) {
    expectsString($w['qux']);
}
