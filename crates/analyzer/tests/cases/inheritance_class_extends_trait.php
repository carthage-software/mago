<?php

declare(strict_types=1);

trait InhSomeTrait
{
}

/** @mago-expect analysis:invalid-extend */
class InhClassExtendsTrait extends InhSomeTrait
{
}
