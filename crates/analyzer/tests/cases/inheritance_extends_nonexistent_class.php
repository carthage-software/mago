<?php

declare(strict_types=1);

/** @mago-expect analysis:non-existent-class-like */
class InhExtendsMissing extends DefinitelyNotAClass
{
}
