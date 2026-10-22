<?php

declare(strict_types=1);

final class Item
{
    /** @mutation-free */
    public function name(): string
    {
        return 'item';
    }
}

final class Holder
{
    public function __construct(
        private ?Item $item,
    ) {}

    /** @mutation-free */
    public function get(): ?Item
    {
        return $this->item;
    }

    public function reset(): bool
    {
        $this->item = null;

        return true;
    }
}

function with_and(Holder $holder): bool
{
    return $holder->get() !== null && $holder->get()->name() !== '';
}

function with_if(Holder $holder): bool
{
    if ($holder->get() !== null) {
        return $holder->get()->name() !== '';
    }

    return false;
}

function with_and_in_if(Holder $holder, bool $flag): string
{
    if ($holder->get() !== null && $flag) {
        return $holder->get()->name();
    }

    return '';
}

function with_and_in_if_narrowed_on_right(Holder $holder, bool $flag): string
{
    if ($flag && $holder->get() !== null) {
        return $holder->get()->name();
    }

    return '';
}

function with_nested_and(Holder $holder, bool $flag): bool
{
    return $flag && $holder->get() !== null && $holder->get()->name() !== '';
}

function with_instanceof_and(Holder $holder): bool
{
    return $holder->get() instanceof Item && $holder->get()->name() !== '';
}

function with_or(Holder $holder): bool
{
    return $holder->get() === null || $holder->get()->name() !== '';
}

function with_impure_call_in_between(Holder $holder): bool
{
    // @mago-expect analysis:possible-method-access-on-null
    return $holder->get() !== null && $holder->reset() && $holder->get()->name() !== '';
}

function with_or_guard(Holder $holder): string
{
    if ($holder->get() === null || $holder->get()->name() === '') {
        return '';
    }

    return $holder->get()->name();
}

function with_impure_call_on_right(Holder $holder): bool
{
    if ($holder->get() !== null && $holder->reset()) {
        // @mago-expect analysis:possible-method-access-on-null
        return $holder->get()->name() !== '';
    }

    return false;
}

function with_negation(Holder $holder): bool
{
    return !($holder->get() === null) && $holder->get()->name() !== '';
}

function with_parenthesized_and(Holder $holder, bool $a, bool $b): bool
{
    return ($a && ($holder->get() !== null && $b)) && $holder->get()->name() !== '';
}

function with_ternary_simple(Holder $holder): string
{
    return $holder->get() !== null
        ? $holder->get()->name()
        : '';
}

function with_ternary_or(Holder $holder): string
{
    return $holder->get() === null || $holder->get()->name() === ''
        ? ''
        : $holder->get()->name();
}

function with_assert(Holder $holder): string
{
    assert($holder->get() !== null && $holder->get()->name() !== '');

    return $holder->get()->name();
}

final class SubEntity
{
    /** @mutation-free */
    public function isActive(): bool
    {
        return true;
    }
}

final class DomainEntity
{
    public function __construct(
        private ?SubEntity $subEntity,
    ) {}

    /** @mutation-free */
    public function getSubEntity(): ?SubEntity
    {
        return $this->subEntity;
    }
}

function check_domain_entity(DomainEntity $entity): bool
{
    return $entity->getSubEntity() !== null && $entity->getSubEntity()->isActive();
}

function check_domain_entity_in_if(DomainEntity $entity): bool
{
    if ($entity->getSubEntity() !== null && $entity->getSubEntity()->isActive()) {
        return $entity->getSubEntity()->isActive();
    }

    return false;
}

function check_domain_entity_ternary(DomainEntity $entity): bool
{
    return $entity->getSubEntity() !== null && $entity->getSubEntity()->isActive()
        ? $entity->getSubEntity()->isActive()
        : false;
}



