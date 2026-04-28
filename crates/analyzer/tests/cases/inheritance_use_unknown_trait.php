<?php

declare(strict_types=1);

class InhUnknownTraitUser
{
    /** @mago-expect analysis:non-existent-class-like */
    use InhMissingTrait;
}
