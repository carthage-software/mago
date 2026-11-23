<?php

declare(strict_types=1);

final class InaccessiblePropertyTheme
{
    private string $stylesheet = 'x';

    public function __get(string $name): string
    {
        return $this->stylesheet;
    }

    public function __set(string $name, string $value): void
    {
        $this->stylesheet = $value;
    }
}

function inaccessible_property_goes_through_magic_methods(InaccessiblePropertyTheme $theme): string
{
    /** @mago-expect analysis:non-documented-property */
    $theme->stylesheet = 'y';

    return $theme->stylesheet;
}
