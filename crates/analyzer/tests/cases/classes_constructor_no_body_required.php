<?php

declare(strict_types=1);

/** @mago-expect analysis:uninitialized-property */
final class ClassesEmptyCtor
{
    public string $name;

    public function __construct()
    {
    }
}
