<?php

declare(strict_types=1);

/**
 * @param-out list<int> $values
 */
function fill_with_ints(mixed &$values): void
{
    $values = [1];
}

/**
 * @param list<int> $values
 */
function take_int_list(array $values): void
{
}

function fill_superglobal(): void
{
    fill_with_ints($_GET);
    take_int_list($_GET);
}

final class ByReferenceOutHolder
{
    public mixed $values = null;

    public function fill(): void
    {
        fill_with_ints($this->values);
        take_int_list($this->values);
    }
}
