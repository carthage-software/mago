<?php

declare(strict_types=1);

namespace SdkRepeatedReaderCalls;

final class Reader
{
    private int $offset = 0;

    public function readU8(): int
    {
        return ++$this->offset;
    }
}

/** @throws \LogicException */
function query(Reader $reader, int $operation, ?int $classLikeKind): void
{
    if ($reader->readU8() !== $operation) {
        throw new \LogicException();
    }

    if ($operation === 1 && $reader->readU8() !== ($classLikeKind ?? 0)) {
        throw new \LogicException();
    }
}

/** @throws \LogicException */
function combinedQuery(Reader $reader, int $generation, int $operation, ?int $classLikeKind): void
{
    if ($reader->readU8() !== $generation || $reader->readU8() !== $operation) {
        throw new \LogicException();
    }

    if ($operation === 8 && $reader->readU8() !== ($classLikeKind ?? 0)) {
        throw new \LogicException();
    }
}

/**
 * @param list<non-empty-string> $exceptions
 * @throws \LogicException
 */
function validateExceptions(array $exceptions): void
{
    foreach ($exceptions as $exception) {
        // @mago-expect analysis:redundant-type-comparison,redundant-logical-operation(2),redundant-comparison
        if (!is_string($exception) || $exception === '' || str_contains($exception, "\0")) {
            throw new \LogicException('Invalid exception name.');
        }
    }
}

function impossibleConjunction(bool $a, bool $b, bool $c): void
{
    if ($a && $b) {
        return;
    }

    // @mago-expect analysis:paradoxical-condition,impossible-condition,redundant-logical-operation(2)
    if ($a && $b && $c) {
    }
}

function impossibleDisjunction(bool $a, bool $b, bool $c, bool $d): void
{
    if (($a && $b) || $c) {
        return;
    }

    // @mago-expect analysis:impossible-condition(2),redundant-logical-operation(3)
    if (($a && $b && $d) || $c) {
    }
}
