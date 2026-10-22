<?php

declare(strict_types=1);

namespace Issue2600;

final class Item {}

final class Box
{
    public ?Item $item = null;

    /** @pure */
    public function get(): ?Item
    {
        return $this->item;
    }

    /** @phpstan-assert-if-true !null $this->get() */
    public function has(): bool
    {
        return $this->get() !== null;
    }

    /** @phpstan-assert-if-false !null $this->get() */
    public function lacks(): bool
    {
        return $this->get() === null;
    }

    /** @phpstan-assert-if-true !null $this->item */
    public function hasItem(): bool
    {
        return $this->item !== null;
    }

    public function reset(bool $result): bool
    {
        $this->item = null;

        return $result;
    }

    /** @phpstan-assert null $this->get() */
    public function resetAndAssertNull(bool $result): bool
    {
        $this->item = null;

        return $result;
    }
}

function assertIfTrueOnRightOfOr(Box $box, bool $flag): Item
{
    if ($flag || !$box->has()) {
        return new Item();
    }

    return $box->get();
}

function assertIfFalseOnRightOfOr(Box $box, bool $flag): Item
{
    if ($flag || $box->lacks()) {
        return new Item();
    }

    return $box->get();
}

function threeWayOr(Box $box, bool $flag, bool $other): Item
{
    if ($flag || $other || !$box->has()) {
        return new Item();
    }

    return $box->get();
}

function parenthesizedRightOr(Box $box, bool $flag, bool $other): Item
{
    if ($flag || ($other || !$box->has())) {
        return new Item();
    }

    return $box->get();
}

function negatedWholeOr(Box $box, bool $flag): Item
{
    if (!($flag || !$box->has())) {
        return $box->get();
    }

    return new Item();
}

function elseBranch(Box $box, bool $flag): Item
{
    if ($flag || !$box->has()) {
        return new Item();
    } else {
        return $box->get();
    }
}

function plainGuard(Box $box): Item
{
    if (!$box->has()) {
        return new Item();
    }

    return $box->get();
}

function assertingCallOnLeftOfOr(Box $box, bool $flag): Item
{
    if (!$box->has() || $flag) {
        return new Item();
    }

    return $box->get();
}

function positiveAnd(Box $box, bool $flag): Item
{
    if ($flag && $box->has()) {
        return $box->get();
    }

    return new Item();
}

function propertyTargetWithOr(Box $box, bool $flag): Item
{
    if ($flag || !$box->hasItem()) {
        return new Item();
    }

    return $box->item;
}

function shortCircuitedTrueAssertion(Box $box, bool $flag): Item
{
    if ($flag || $box->has()) {
        // @mago-expect analysis:nullable-return-statement,invalid-return-statement
        return $box->get();
    }

    return new Item();
}

function shortCircuitedFalseAssertion(Box $box, bool $flag): Item
{
    if ($flag || !$box->lacks()) {
        // @mago-expect analysis:nullable-return-statement,invalid-return-statement
        return $box->get();
    }

    return new Item();
}

function afterConditionalAssertion(Box $box, bool $flag): Item
{
    if ($flag || !$box->has()) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}

function conditionalMutationInvalidatesAssertion(Box $box, bool $flag, bool $result): Item
{
    if (!$box->has()) {
        return new Item();
    }

    $flag || $box->reset($result);

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}

function conditionalMutationWithAssertion(Box $box, bool $flag, bool $result): Item
{
    if (!$box->has()) {
        return new Item();
    }

    $flag || $box->resetAndAssertNull($result);

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}

function shortCircuitedMutationPreservesAssertion(Box $box, bool $result): Item
{
    if (!$box->has()) {
        return new Item();
    }

    // @mago-expect analysis:redundant-logical-operation
    true || $box->reset($result);

    return $box->get();
}

function mutationOnRightInvalidatesAssertion(Box $box, bool $result): Item
{
    if (!$box->has() || $box->reset($result)) {
        return new Item();
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}

function stop(Box $box): never
{
    $box->item = null;

    exit;
}

function terminatingRightOfOr(Box $box, bool $flag): Item
{
    if (!$box->has()) {
        return new Item();
    }

    $flag || stop($box);

    return $box->get();
}

function terminatingRightOfAnd(Box $box, bool $flag): Item
{
    if (!$box->has()) {
        return new Item();
    }

    $flag && stop($box);

    return $box->get();
}

function terminatingRightOfOrInIf(Box $box, bool $flag): Item
{
    if (!$box->has()) {
        return new Item();
    }

    if ($flag || stop($box)) {
    }

    return $box->get();
}

function terminatingRightOfAndInIf(Box $box, bool $flag): Item
{
    if (!$box->has()) {
        return new Item();
    }

    if ($flag && stop($box)) {
    }

    return $box->get();
}

function terminatingNestedOr(Box $box, bool $flag, bool $other): Item
{
    if (!$box->has()) {
        return new Item();
    }

    if ($flag || ($other || stop($box))) {
    }

    return $box->get();
}

function terminatingNestedAnd(Box $box, bool $flag, bool $other): Item
{
    if (!$box->has()) {
        return new Item();
    }

    if ($flag && ($other && stop($box))) {
    }

    return $box->get();
}

/** @param list<bool> $flags */
function terminatingRightOfOrInLoop(Box $box, array $flags): Item
{
    if (!$box->has()) {
        return new Item();
    }

    foreach ($flags as $flag) {
        $flag || stop($box);

        return $box->get();
    }

    return new Item();
}

/** @param list<bool> $flags */
function terminatingRightOfAndInLoop(Box $box, array $flags): Item
{
    if (!$box->has()) {
        return new Item();
    }

    foreach ($flags as $flag) {
        $flag && stop($box);

        return $box->get();
    }

    return new Item();
}

function mutatingLeftOfTerminatingOr(Box $box, bool $flag): Item
{
    if (!$box->has()) {
        return new Item();
    }

    $box->reset($flag) || stop($box);

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}

function mutatingLeftOfTerminatingAnd(Box $box, bool $flag): Item
{
    if (!$box->has()) {
        return new Item();
    }

    if ($box->reset($flag) && stop($box)) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}

function conditionalMutationInAndInvalidatesAssertion(Box $box, bool $flag, bool $result): Item
{
    if (!$box->has()) {
        return new Item();
    }

    $flag && $box->reset($result);

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}

/** @throws \RuntimeException */
function throwAfterReset(Box $box): never
{
    $box->item = null;

    throw new \RuntimeException();
}

function caughtThrowOnRightOfOr(Box $box, bool $flag): Item
{
    if (!$box->has()) {
        return new Item();
    }

    try {
        $flag || throwAfterReset($box);
    } catch (\RuntimeException) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}

function caughtThrowOnRightOfAnd(Box $box, bool $flag): Item
{
    if (!$box->has()) {
        return new Item();
    }

    try {
        $flag && throwAfterReset($box);
    } catch (\RuntimeException) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $box->get();
}
