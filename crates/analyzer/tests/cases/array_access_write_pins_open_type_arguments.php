<?php

declare(strict_types=1);

final class PinnedStorageNode
{
    public string $name = 'node';
}

function pinned_storage_read(PinnedStorageNode $key): string
{
    $storage = new SplObjectStorage();
    $storage[$key] = new PinnedStorageNode();

    return $storage[$key]->name;
}

function pinned_array_object_read(): int
{
    $values = new ArrayObject();
    $values[0] = ['k' => 1];

    return $values[0]['k'];
}
