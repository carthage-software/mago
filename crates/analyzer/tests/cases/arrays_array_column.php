<?php

declare(strict_types=1);

/**
 * @param list<array{id: int, name: string}> $rows
 * @return list<string>
 */
function names(array $rows): array
{
    return array_column($rows, 'name');
}

/**
 * `array_column()` reads from outside the class: a private property with a magic `@property*`
 * annotation resolves to the annotation's type, and a private property shadowed by an
 * annotation is keyed by the annotation's type as well.
 *
 * @property-read int $magicId
 * @property-read string $label
 */
final class ColumnEntry
{
    public string $name = '';
    private int $magicId = 0;

    /**
     * @var int
     */
    private $label = 0;

    public function __get(string $prop): mixed
    {
        return 0;
    }
}

/**
 * @param list<ColumnEntry> $entries
 * @return list<int>
 */
function magic_ids(array $entries): array
{
    return array_column($entries, 'magicId');
}

/**
 * @param list<ColumnEntry> $entries
 */
function keyed_by_magic_label(array $entries): void
{
    take_entries_by_label(array_column($entries, null, 'label'));
}

/**
 * @param array<string, ColumnEntry> $_entries
 */
function take_entries_by_label(array $_entries): void {}

final class UntypedColumnEntry
{
    public $untyped;
}

/**
 * An untyped public property resolves the column to `mixed`.
 *
 * @param list<UntypedColumnEntry> $entries
 * @return list<mixed>
 */
function untyped_column(array $entries): array
{
    return array_column($entries, 'untyped');
}
