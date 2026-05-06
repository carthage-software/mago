<?php

declare(strict_types=1);

/** @inheritors InhSealedIfaceAllowed */
interface InhSealedIface {}

class InhSealedIfaceAllowed implements InhSealedIface {}

class InhSealedIfaceDisallowed implements InhSealedIface {}
