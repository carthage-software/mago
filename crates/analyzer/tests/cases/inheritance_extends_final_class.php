<?php

declare(strict_types=1);

final class InhFinalBase
{
}

/** @mago-expect analysis:extend-final-class */
class InhExtendsFinal extends InhFinalBase
{
}
