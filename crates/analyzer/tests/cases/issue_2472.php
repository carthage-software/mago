<?php

declare(strict_types=1);

/** @param ArrayObject<int, string> $object */
function castArrayObject(ArrayObject $object): void
{
    // @mago-expect analysis:invalid-type-cast
    $array = (array) $object;
    foreach (array_keys($array) as $key) {
        if (is_int($key)) {
            echo $key;
        }
    }
}

/** @param ArrayIterator<int, string> $object */
function castArrayIterator(ArrayIterator $object): void
{
    // @mago-expect analysis:invalid-type-cast
    $array = (array) $object;
    foreach (array_keys($array) as $key) {
        if (is_int($key)) {
            echo $key;
        }
    }
}

/** @param RecursiveArrayIterator<int, string> $object */
function castRecursiveArrayIterator(RecursiveArrayIterator $object): void
{
    // @mago-expect analysis:invalid-type-cast
    $array = (array) $object;
    foreach (array_keys($array) as $key) {
        if (is_int($key)) {
            echo $key;
        }
    }
}

/** @extends ArrayObject<int, string> */
final class Names extends ArrayObject
{
    public bool $enabled = true;
}

function castSubclass(Names $object): void
{
    // @mago-expect analysis:invalid-type-cast
    $array = (array) $object;
    foreach (array_keys($array) as $key) {
        if (is_int($key)) {
            echo $key;
        }
    }
}

/** @param SplFixedArray<string> $object */
function castSplFixedArray(SplFixedArray $object): void
{
    // @mago-expect analysis:invalid-type-cast
    $array = (array) $object;
    foreach (array_keys($array) as $key) {
        if (is_int($key)) {
            echo $key;
        }
    }
}

function castStdClass(stdClass $object): void
{
    $array = (array) $object;
    foreach (array_keys($array) as $key) {
        if (is_int($key)) {
            echo $key;
        }
    }
}

function castAnyObject(object $object): void
{
    // @mago-expect analysis:invalid-type-cast
    $array = (array) $object;
    foreach (array_keys($array) as $key) {
        if (is_int($key)) {
            echo $key;
        }
    }
}
