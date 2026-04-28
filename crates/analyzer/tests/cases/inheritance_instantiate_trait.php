<?php

declare(strict_types=1);

trait InhCannotInstantiateTrait
{
}

/** @mago-expect analysis:trait-instantiation */
new InhCannotInstantiateTrait();
