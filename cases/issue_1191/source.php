<?php

namespace Fixture;

function getItems(): array
{
    return [1, 2, 3];
}

function process(iterable $items): void {}

/** @return list<int> */
function getNumbers(): array
{
    return [1, 2, 3];
}

function bar(): ?array
{
    return null;
}

function baz(): array|false
{
    return false;
}

/** @return void */
function qux(array $data): void {}

/** @param list<string> $data */
function quux(array $data): void {}

class Foo
{
    public array $items = [];

    /** @var array<string, int> */
    public array $map = [];

    public function getList(): array
    {
        return [];
    }

    public function setList(array $list): void {}

    /** @param list<int> $list */
    public function setTypedList(array $list): void {}

    /** @return array<string, mixed> */
    public function getMap(): array
    {
        return [];
    }

    public function getIterable(): iterable
    {
        return [];
    }
}

function multi(array $a, iterable $b): array
{
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
