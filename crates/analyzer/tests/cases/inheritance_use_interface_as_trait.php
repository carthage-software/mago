<?php

declare(strict_types=1);

interface InhIfaceAsTrait
{
}

class InhUserOfIfaceAsTrait
{
    /** @mago-expect analysis:invalid-trait-use */
    use InhIfaceAsTrait;
}
