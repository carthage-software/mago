<?php

declare(strict_types=1);

/** @inheritors InhSealedAllowed */
class InhSealedParent
{
}

class InhSealedAllowed extends InhSealedParent
{
}

/** @mago-expect analysis:invalid-extend */
class InhSealedDisallowed extends InhSealedParent
{
}
