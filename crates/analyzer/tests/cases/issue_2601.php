<?php

declare(strict_types=1);

namespace Repro;

final class Item
{
    public function __construct(public readonly string $name) {}
}

function consume(string $value): void {}

function problem_or_of_two_variables(?Item $item, bool $x, bool $y): void
{
    $a = $item !== null && $x;
    $b = $item !== null && $y;

    if ($a || $b) {
        consume($item->name);
    }
}

function problem_or_of_two_variables_early_return(?Item $item, bool $x, bool $y): void
{
    $a = $item !== null && $x;
    $b = $item !== null && $y;

    if (!($a || $b)) {
        return;
    }

    consume($item->name);
}

function problem_and_of_two_negated_variables(?Item $item, bool $x, bool $y): void
{
    $a = $item === null || $x;
    $b = $item === null || $y;

    if (!$a && !$b) {
        consume($item->name);
    }
}

function problem_or_of_two_plain_null_checks(?Item $item): void
{
    $a = $item !== null;
    $b = $item !== null;

    if ($a || $b) {
        consume($item->name);
    }
}

function sound_error_unrelated_flag(?Item $item, bool $x, bool $flag): void
{
    $a = $item !== null && $x;

    if ($a || $flag) {
        // @mago-expect analysis:possibly-null-property-access,possibly-null-argument
        consume($item->name);
    }
}

function sound_error_different_subjects(?Item $item, ?Item $other, bool $x): void
{
    $a = $item !== null && $x;
    $b = $other !== null && $x;

    if ($a || $b) {
        // @mago-expect analysis:possibly-null-property-access,possibly-null-argument
        consume($item->name);
    }
}

function control_inline(?Item $item, bool $x): void
{
    if ($item !== null && $x) {
        consume($item->name);
    }
}

function control_single_variable(?Item $item, bool $x): void
{
    $ok = $item !== null && $x;

    if ($ok) {
        consume($item->name);
    }
}

function control_single_variable_early_return(?Item $item, bool $x): void
{
    $ok = $item !== null && $x;

    if (!$ok) {
        return;
    }

    consume($item->name);
}

function control_single_variable_and_other(?Item $item, bool $x, bool $y): void
{
    $ok = $item !== null && $x;

    if ($ok && $y) {
        consume($item->name);
    }
}

function control_single_variable_ternary(?Item $item, bool $x): string
{
    $ok = $item !== null && $x;

    return $ok ? $item->name : '';
}

function control_variable_or_inline_fact(?Item $item, bool $x, bool $y): void
{
    $ok = $item !== null && $x;

    if ($ok || ($item !== null && $y)) {
        consume($item->name);
    }
}

function control_inline_or_inline(?Item $item, bool $x, bool $y): void
{
    if (($item !== null && $x) || ($item !== null && $y)) {
        consume($item->name);
    }
}

function three_stored_variables(?Item $item, bool $x, bool $y, bool $z): string
{
    $a = $item !== null && $x;
    $b = $item !== null && $y;
    $c = $item !== null && $z;

    return $a || $b || $c ? $item->name : '';
}

function strict_boolean_comparisons(?Item $item, bool $x, bool $y): void
{
    $a = $item !== null && $x;
    $b = $item !== null && $y;

    if ($a === true || $b === true) {
        consume($item->name);
    }
}

function reassigned_subject(?Item $item, ?Item $replacement, bool $x, bool $y): void
{
    $a = $item !== null && $x;
    $b = $item !== null && $y;
    $item = $replacement;

    if ($a || $b) {
        // @mago-expect analysis:possibly-null-property-access,possibly-null-argument
        consume($item->name);
    }
}

function reassigned_flag(?Item $item, bool $x, bool $y, bool $flag): void
{
    $a = $item !== null && $x;
    $b = $item !== null && $y;
    $b = $flag;

    if ($a || $b) {
        // @mago-expect analysis:possibly-null-property-access,possibly-null-argument
        consume($item->name);
    }
}

/** @mago-assert-if-true !=null $item */
function maybe_has_item(?Item $item, bool $flag): bool
{
    return $item !== null && $flag;
}

function one_way_true_assertions(?Item $item, bool $x, bool $y): void
{
    $a = maybe_has_item($item, $x);
    $b = maybe_has_item($item, $y);

    if ($a || $b) {
        consume($item->name);
    }

    if (!$a && !$b) {
        // @mago-expect analysis:possibly-null-property-access,possibly-null-argument
        consume($item->name);
    }
}

/** @mago-assert-if-false !=null $item */
function maybe_lacks_item(?Item $item, bool $flag): bool
{
    return $item === null || $flag;
}

function one_way_false_assertions(?Item $item, bool $x, bool $y): void
{
    $a = maybe_lacks_item($item, $x);
    $b = maybe_lacks_item($item, $y);

    if (!$a && !$b) {
        consume($item->name);
    }

    if ($a || $b) {
        // @mago-expect analysis:possibly-null-property-access,possibly-null-argument
        consume($item->name);
    }
}
