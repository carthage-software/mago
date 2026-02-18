<?php declare(strict_types = 1);

namespace App\Database;

abstract readonly class BaseDatabaseQuery
{

    private function createSingleCursorPagination( // @phpstan-ignore-line
        string $entity,
        string $cursorField,
        mixed $cursorValue,
        callable $getCursorValue,
        int $limit,
        array $criteria = [],
        array $preload = [],
    ): void {}

}
