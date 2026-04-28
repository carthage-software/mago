<?php

declare(strict_types=1);

final class ClassesPropInvDefArr
{
    /** @mago-expect analysis:invalid-property-default-value */
    public string $items = [];
}
