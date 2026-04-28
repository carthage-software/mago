<?php

declare(strict_types=1);

interface InhInsIface
{
}

class InhInsImpl implements InhInsIface
{
}

function inh_check_iface(object $obj): bool {
    return $obj instanceof InhInsIface;
}

inh_check_iface(new InhInsImpl());
