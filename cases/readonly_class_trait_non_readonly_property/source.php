<?php

trait HandleTrait
{
    private string $a = '';
}

readonly class CommandHandler
{
    use HandleTrait;
}

trait ReadonlyTrait
{
    public readonly string $d;
}

readonly class ValidHandler
{
    use ReadonlyTrait;
}

class NonReadonlyHandler
{
    use HandleTrait;
}
