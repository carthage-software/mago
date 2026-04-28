<?php

declare(strict_types=1);

final class State
{
    public null|string $value = null;

    public function reset(): void
    {
        $this->value = null;
    }

    public function setValue(string $v): void
    {
        $this->value = $v;
    }
}

function flow_method_call_invalidates(State $s, string $v): null|int
{
    $s->setValue($v);

    if ($s->value === null) {
        return null;
    }

    $s->reset();

    return null;
}
