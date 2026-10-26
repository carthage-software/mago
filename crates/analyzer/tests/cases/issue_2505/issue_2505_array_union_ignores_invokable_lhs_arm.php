<?php

final class InvokableUnionFailure extends Exception
{
}

final class InvokableUnionCallback
{
    /** @throws InvokableUnionFailure */
    public function __invoke(): void
    {
        throw new InvokableUnionFailure();
    }
}

function add_empty_array_to_invokable_union(array|InvokableUnionCallback $value): void
{
    $value += [];
}
