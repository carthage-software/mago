<?php

declare(strict_types=1);

/** @inheritors InhSealedAllowed */
class InhSealedParent {}

class InhSealedAllowed extends InhSealedParent {}

class InhSealedDisallowed extends InhSealedParent {}
