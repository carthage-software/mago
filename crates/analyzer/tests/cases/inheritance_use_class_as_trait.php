<?php

declare(strict_types=1);

class InhClassAsTrait
{
}

class InhUserOfClassAsTrait
{
    /** @mago-expect analysis:invalid-trait-use */
    use InhClassAsTrait;
}
