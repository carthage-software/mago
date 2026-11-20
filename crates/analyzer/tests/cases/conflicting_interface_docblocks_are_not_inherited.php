<?php

declare(strict_types=1);

interface ConflictingDocblockFirst
{
    /** @param non-empty-string $s */
    public function take(string $s): void;
}

interface ConflictingDocblockSecond
{
    /** @param string $s */
    public function take(string $s): void;
}

final class ConflictingDocblockImplementation implements ConflictingDocblockSecond, ConflictingDocblockFirst
{
    public function take(string $s): void
    {
        echo $s;
    }
}

interface AgreeingDocblockFirst
{
    /** @param non-empty-string $s */
    public function take(string $s): void;
}

interface AgreeingDocblockSecond
{
    /** @param non-empty-string $s */
    public function take(string $s): void;
}

final class AgreeingDocblockImplementation implements AgreeingDocblockSecond, AgreeingDocblockFirst
{
    public function take(string $s): void
    {
        echo $s;
    }
}

(new ConflictingDocblockImplementation())->take('');

function agreeing_docblock_is_inherited(AgreeingDocblockImplementation $implementation): void
{
    /** @mago-expect analysis:invalid-argument */
    $implementation->take('');
}
