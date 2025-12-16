<?php

trait HandleTrait
{
    private string $a = '';
}

readonly class CommandHandler
{
    // @mago-expect analysis:invalid-trait-use
    use HandleTrait;
}

trait ReadonlyTrait
{
    public readonly string $d;
}

// @mago-expect analysis:missing-constructor
readonly class ValidHandler
{
    use ReadonlyTrait;
}

class NonReadonlyHandler
{
    use HandleTrait;
}
