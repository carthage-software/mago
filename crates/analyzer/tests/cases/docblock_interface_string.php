<?php

declare(strict_types=1);

interface IfaceS
{
}

/**
 * @param interface-string<IfaceS> $iface
 */
function takeInterfaceS(string $iface): void
{
    echo $iface;
}

takeInterfaceS(IfaceS::class);
