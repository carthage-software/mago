<?php

declare(strict_types=1);

final class ClassesMcallSelfMissing
{
    public function caller(): void
    {
        /** @mago-expect analysis:non-existent-method */
        $this->bogus();
    }
}
