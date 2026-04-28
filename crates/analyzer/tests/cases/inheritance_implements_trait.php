<?php

declare(strict_types=1);

trait InhAnotherTrait
{
}

/** @mago-expect analysis:invalid-implement */
class InhImplementsTrait implements InhAnotherTrait
{
}
