<?php

declare(strict_types=1);

interface InhCannotInstantiate
{
}

/** @mago-expect analysis:interface-instantiation */
new InhCannotInstantiate();
