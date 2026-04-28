<?php

declare(strict_types=1);

/** @inheritors InhSealedIfaceAllowed */
interface InhSealedIface
{
}

class InhSealedIfaceAllowed implements InhSealedIface
{
}

/** @mago-expect analysis:invalid-implement */
class InhSealedIfaceDisallowed implements InhSealedIface
{
}
