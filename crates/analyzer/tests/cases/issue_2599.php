<?php

declare(strict_types=1);

namespace Repro;

use function in_array;

final class Source
{
    public function __construct(
        public readonly string $group,
        public readonly string $field,
    ) {}
}

final class Holder
{
    public function __construct(
        public readonly ?Source $source,
        public readonly ?string $name,
    ) {}
}

function plain_variable(?string $group): int
{
    return \in_array($group, ['A', 'B'], true) ? \strlen($group) : 0;
}

function nullable_property(Holder $h): int
{
    return \in_array($h->name, ['A', 'B'], true) ? \strlen($h->name) : 0;
}

function nullsafe_with_equality(Holder $h): string
{
    return $h->source?->group === 'A' ? $h->source->field : '';
}

function nullsafe_with_not_null(Holder $h): string
{
    return $h->source?->group !== null ? $h->source->field : '';
}

function nullsafe_with_in_array(Holder $h): string
{
    return \in_array($h->source?->group, ['A', 'B'], true) ? $h->source->field : '';
}

function nested_nullsafe_with_in_array(?Holder $h): string
{
    return in_array($h?->source?->group, ['A', 'B'], true) ? $h->source->field : '';
}

function named_strict_argument(Holder $h): string
{
    return in_array($h->source?->group, ['A', 'B'], strict: true) ? $h->source->field : '';
}

function early_return(Holder $h): string
{
    if (!in_array($h->source?->group, ['A', 'B'], true)) {
        return '';
    }

    return $h->source->field;
}

/** @param list<string> $groups */
function generic_haystack(Holder $h, array $groups): string
{
    return in_array($h->source?->group, $groups, true) ? $h->source->field : '';
}

/** @return 'A'|'B'|'' */
function narrowed_group(Holder $h): string
{
    return in_array($h->source?->group, ['A', 'B'], true) ? $h->source->group : '';
}

function consume_source(Source $source): void {}

function false_branch(Holder $h): void
{
    if (!in_array($h->source?->group, ['A', 'B'], true)) {
        // @mago-expect analysis:possibly-null-argument
        consume_source($h->source);
    }
}

function nullable_haystack(Holder $h): void
{
    if (in_array($h->source?->group, [null, 'A'], true)) {
        // @mago-expect analysis:possibly-null-argument
        consume_source($h->source);
    }
}

/** @param list<string|null> $groups */
function generic_nullable_haystack(Holder $h, array $groups): void
{
    if (in_array($h->source?->group, $groups, true)) {
        // @mago-expect analysis:possibly-null-argument
        consume_source($h->source);
    }
}

/** @param list<mixed> $groups */
function mixed_haystack(Holder $h, array $groups): void
{
    if (in_array($h->source?->group, $groups, true)) {
        // @mago-expect analysis:possibly-null-argument
        consume_source($h->source);
    }
}

function loose_comparison(Holder $h): void
{
    if (in_array($h->source?->group, [false, 'A'])) {
        // @mago-expect analysis:possibly-null-argument
        consume_source($h->source);
    }
}

function unknown_strictness(Holder $h, bool $strict): void
{
    if (in_array($h->source?->group, [false, 'A'], $strict)) {
        // @mago-expect analysis:possibly-null-argument
        consume_source($h->source);
    }
}

function reassigned_receiver(?Source $source): void
{
    // @mago-expect analysis:redundant-comparison
    if (in_array($source?->group, ['A', 'B'], ($source = null) === null)) {
        // @mago-expect analysis:null-argument
        consume_source($source);
    }
}

function clear_source(?Source &$source): true
{
    $source = null;

    return true;
}

function receiver_changed_by_argument(?Source $source): void
{
    if (in_array($source?->group, ['A', 'B'], clear_source($source))) {
        // @mago-expect analysis:possibly-null-argument
        consume_source($source);
    }
}
