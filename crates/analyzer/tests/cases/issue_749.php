<?php

declare(strict_types=1);

final class Constraint
{
    public array|string $mimeTypes = [];
}

function check1(Constraint $constraint): bool
{
    if ($constraint->mimeTypes !== '' && $constraint->mimeTypes !== []) {
        return false;
    }

    return true;
}

function check2(Constraint $constraint): bool
{
    if ($constraint->mimeTypes !== '' && $constraint->mimeTypes !== ['a']) {
        return false;
    }

    return true;
}
