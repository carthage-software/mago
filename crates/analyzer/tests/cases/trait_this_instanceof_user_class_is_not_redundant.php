<?php

declare(strict_types=1);

trait TraitInstanceofNamed
{
    public function name(): string
    {
        if ($this instanceof TraitInstanceofFirst) {
            return 'first';
        }

        return 'second';
    }
}

final class TraitInstanceofFirst
{
    use TraitInstanceofNamed;
}

final class TraitInstanceofSecond
{
    use TraitInstanceofNamed;
}
