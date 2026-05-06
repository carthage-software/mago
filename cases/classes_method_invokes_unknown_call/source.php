<?php

declare(strict_types=1);

final class ClassesMcallSelfMissing
{
    public function caller(): void
    {
        $this->bogus();
    }
}
