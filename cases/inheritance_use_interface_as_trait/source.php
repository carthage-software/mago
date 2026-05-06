<?php

declare(strict_types=1);

interface InhIfaceAsTrait {}

class InhUserOfIfaceAsTrait
{
    use InhIfaceAsTrait;
}
