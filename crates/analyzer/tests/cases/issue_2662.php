<?php

declare(strict_types=1);

namespace Issue2662;

class Event
{
    public function __construct(public int $v) {}

    /** @pure */
    public function value(): int
    {
        return $this->v;
    }
}

/** @return list<Event> */
function getEvents(): array
{
    if (rand(0, 1) === 1) {
        return [new Event(1)];
    }

    return [new Event(2)];
}

function sameSource(): void
{
    $event = getEvents()[0];
    assert(1 === $event->v);

    foreach (getEvents() as $event) {
        assert(2 === $event->v);
    }
}

function differentSource(): void
{
    $event = getEvents()[0];
    assert(1 === $event->v);

    foreach ([new Event(2)] as $event) {
        assert(2 === $event->v);
    }
}

function differentName(): void
{
    $event = getEvents()[0];
    assert(1 === $event->v);

    foreach (getEvents() as $other) {
        assert(2 === $other->v);
    }
}

/**
 * @param array{id: int} $entry
 * @param list<array{id: int}> $entries
 */
function arrayEntries(array $entry, array $entries): void
{
    assert($entry['id'] === 1);

    foreach ($entries as $entry) {
        assert($entry['id'] === 2);
    }
}

/** @param array<string, int> $items */
function stringKeys(string $key, array $items): void
{
    assert($key[0] === 'a');

    foreach ($items as $key => $value) {
        assert($key[0] === 'b');
    }
}

/** @param list<list{Event}> $rows */
function destructured(Event $event, array $rows): void
{
    assert($event->v === 1);

    foreach ($rows as [$event]) {
        assert($event->v === 2);
    }
}

/** @param list<Event> $events */
function byReference(Event $event, array $events): void
{
    assert($event->v === 1);

    foreach ($events as &$event) {
        assert($event->v === 2);
    }
}

/** @param list<Event> $events */
function afterLoop(Event $event, array $events): void
{
    assert($event->v === 1);

    foreach ($events as $event) {
    }

    if ($event->v === 2) {
    }
}

/**
 * @param list<Event> $events
 * @return 1
 */
function unrelatedVariable(Event $event, array $events): int
{
    assert($event->v === 1);

    foreach ($events as $other) {
        assert($other->v === 2);
    }

    return $event->v;
}

function ordinaryAssignment(Event $event): void
{
    assert($event->v === 1);
    $event = new Event(2);
    assert($event->v === 2);
}

/** @return 1 */
function emptyLoop(Event $event): int
{
    assert($event->v === 1);

    foreach ([] as $event) {
    }

    return $event->v;
}

/** @param list<Event> $events */
function pureMethod(Event $event, array $events): void
{
    assert($event->value() === 1);

    foreach ($events as $event) {
        assert($event->value() === 2);
    }
}
