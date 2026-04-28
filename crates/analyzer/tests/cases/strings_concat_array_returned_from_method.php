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
    /** @mago-expect analysis:array-to-string-conversion */
    return 'foo' . $h->get();
}
