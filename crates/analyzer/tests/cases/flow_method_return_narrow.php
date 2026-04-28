<?php

declare(strict_types=1);

final class Repo
{
    public function find(int $id): null|string
    {
        return $id > 0 ? 'name' : null;
    }
}

function flow_method_return_narrow(Repo $r): int
{
    $name = $r->find(1);

    if ($name !== null) {
        return strlen($name);
    }

    return 0;
}
