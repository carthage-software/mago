<?php

declare(strict_types=1);

final class HasArr
{
    /** @return list<int> */
    public function get(): array
    {
        return [1];
    }
}

function probe(HasArr $h): string
{
    return 'foo' . $h->get();
}
