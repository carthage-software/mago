<?php

final class Exemple
{
    public function suffixe(PermisDeConstruire $permisDeConstruire): void
    {
        $this->code->setSuffixe(
            'p'
                . String_::fromString($this->formatExtension->romanNumber($permisDeConstruire->getPhase()))
                    ->upper()
                    ->toString(),
        );
    }
}
