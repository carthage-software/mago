<?php

declare(strict_types=1);

class InhUnrelatedA
{
}

class InhUnrelatedB
{
}

function inh_takes_a(InhUnrelatedA $a): void {
}

/** @mago-expect analysis:invalid-argument */
inh_takes_a(new InhUnrelatedB());
