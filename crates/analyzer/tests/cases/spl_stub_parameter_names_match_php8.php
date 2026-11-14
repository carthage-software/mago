<?php

declare(strict_types=1);

$object = new ArrayObject(array: ['b' => 2, 'a' => 1], flags: 0, iteratorClass: ArrayIterator::class);
$object->offsetExists(key: 'a');
$object->uasort(callback: static fn(int $x, int $y): int => $x <=> $y);
$object->asort(SORT_STRING);

$iterator = new ArrayIterator(['x' => 'apple', 'y' => 'pear']);
$iterator->offsetSet(key: 'z', value: 'plum');

$matches = new RegexIterator(iterator: $iterator, pattern: '/^p/');
$directory = new DirectoryIterator(directory: __DIR__);

echo (int) grapheme_strpos('abc', 'b', 0, locale: 'en'), iterator_count($matches), $directory->getPath();
