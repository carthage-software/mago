<?php

declare(strict_types=1);

interface InheritedDocblockHandler
{
    /**
     * @param object $response
     */
    public function handle(ArrayObject $response): void;
}

final class InheritedDocblockHandlerImplementation implements InheritedDocblockHandler
{
    public function handle(ArrayObject $response): void
    {
        echo $response->count();
    }
}
