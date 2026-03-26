<?php

namespace Fixture;

function getItems(): array // @mago-expect analysis:imprecise-type
{
    return [1, 2, 3];
}

function process(iterable $items): void {} // @mago-expect analysis:imprecise-type

/** @return list<int> */
function getNumbers(): array
{
    return [1, 2, 3];
}

function bar(): ?array // @mago-expect analysis:imprecise-type
{
    return null;
}

function baz(): array|false // @mago-expect analysis:imprecise-type
{
    return false;
}

/** @return void */
function qux(array $data): void {} // @mago-expect analysis:imprecise-type

/** @param list<string> $data */
function quux(array $data): void {}

class Foo
{
    public array $items = []; // @mago-expect analysis:imprecise-type

    /** @var array<string, int> */
    public array $map = [];

    public function getList(): array // @mago-expect analysis:imprecise-type
    {
        return [];
    }

    public function setList(array $list): void {} // @mago-expect analysis:imprecise-type

    /** @param list<int> $list */
    public function setTypedList(array $list): void {}

    /** @return array<string, mixed> */
    public function getMap(): array
    {
        return [];
    }

    public function getIterable(): iterable // @mago-expect analysis:imprecise-type
    {
        return [];
    }
}

function multi(
    // @mago-expect analysis:imprecise-type
    array $a,
    // @mago-expect analysis:imprecise-type
    iterable $b,
): array { // @mago-expect analysis:imprecise-type
    return [];
}

/** @return array<array-key, mixed> */
function explicitArray(): array
{
    return [];
}

/** @return iterable<mixed, mixed> */
function explicitIterable(): iterable
{
    return [];
}

/** @param array<array-key, mixed> $data */
function explicitParam(array $data): void {}

/** @param iterable<mixed, mixed> $items */
function explicitIterableParam(iterable $items): void {}
