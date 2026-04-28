<?php

declare(strict_types=1);

final class ClassesPropInvDefStr
{
    /** @mago-expect analysis:invalid-property-default-value */
    public string $name = 1;
}
