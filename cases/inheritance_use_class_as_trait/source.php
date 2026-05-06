<?php

declare(strict_types=1);

class InhClassAsTrait {}

class InhUserOfClassAsTrait
{
    use InhClassAsTrait;
}
