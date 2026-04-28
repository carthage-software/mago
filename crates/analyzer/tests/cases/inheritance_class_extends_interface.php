<?php

declare(strict_types=1);

interface InhSomeInterface
{
}

/** @mago-expect analysis:invalid-extend */
class InhClassExtendsInterface extends InhSomeInterface
{
}
