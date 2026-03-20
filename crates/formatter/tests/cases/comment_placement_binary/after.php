<?php

// EndOfLine line comment between LHS and operator
$a =
    $x // trailing LHS
    && $y;

// OwnLine line comment between LHS and operator
$b =
    $x
    // own-line before operator
    && $y;

// EndOfLine block comment between LHS and operator
$c = $x /* inline block */ && $y;

// OwnLine block comment between LHS and operator
$d = $x /* own-line block before op */ && $y;

// EndOfLine line comment between operator and RHS
$e =
    $x && // after operator
    $y;

// OwnLine line comment between operator and RHS
$f =
    $x &&
    // own-line before RHS
    $y;

// EndOfLine block comment between operator and RHS
$g = $x && /* inline block */ $y;

// OwnLine block comment between operator and RHS
$h = $x && /* own-line block before RHS */ $y;

// Multiline block comment between operator and RHS
$i =
    $x &&
    /* multi
     * line
     * block */
    $y;

// Trailing line comment after RHS
$j = $x && $y; // trailing RHS

// Multiple line comments in same gap
$k =
    $x // first comment
    // second comment
    && $y;

// Flat chain with intermediate trailing comments
$l =
    $a // after a
    || $b // after b
    || $c;

// String concatenation with block comment
$m = $x . /* separator */ $y . $z;

// Trailing comment in chain
function check($a, $b, $c, $d)
{
    return (
        $a === $b
        && $c === $d // both pairs must match
        || $a === $c
        && $b === $d // or cross-match
    );
}

// Long chain with trailing comment
function long_chain($version_parts_count, $major_version, $version_parts, $last_part_split, $version_extra)
{
    return (
        $version_parts_count > 3
        || $version_parts_count < 2
        || !is_numeric($major_version)
        || $version_parts_count === 3
        && !is_numeric($version_parts[1])
        || !is_numeric($last_part_split[0])
        && $last_part_split !== 'x'
        && $version_extra !== 'dev' // validate version
    );
}
