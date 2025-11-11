<?php

declare(strict_types=1);

namespace Rod\MagoGeneratorReproducer\App;

class File
{
    public function foo(bool $boolean, null|object $object): void
    {
        if (null === $object || false === $boolean) {
            return;
        }
    }
}
