<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

/** @mago-expect analysis:null-argument */
takesInt(null);
