<?php

declare(strict_types=1);

class InhExpA
{
}

class InhExpB
{
}

function inh_expects_a(InhExpA $a): void
{
}

/** @mago-expect analysis:invalid-argument */
inh_expects_a(new InhExpB());
