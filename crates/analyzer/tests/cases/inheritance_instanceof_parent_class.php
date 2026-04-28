<?php

declare(strict_types=1);

class InhInsParent
{
}

class InhInsChild extends InhInsParent
{
}

function inh_check_parent(InhInsParent $obj): bool {
    return $obj instanceof InhInsParent;
}

inh_check_parent(new InhInsChild());
