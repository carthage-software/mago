<?php

declare(strict_types=1);

final class ClassesMethodThrows
{
    /**
     * @throws RuntimeException
     */
    public function fail(): never
    {
        throw new RuntimeException('boom');
    }
}

try {
    (new ClassesMethodThrows())->fail();
} catch (RuntimeException $e) {
    echo $e->getMessage();
}
