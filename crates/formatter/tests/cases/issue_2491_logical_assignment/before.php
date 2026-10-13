<?php

final class Exemple
{
    public function peutLister(string $codeAction, object $entity): bool
    {
        $isGrantedLister = String_::fromString('MODIFIER')->containsAny($codeAction)->isTrue()
            || $this->security->isGranted(String_::fromString($codeAction)->replace('MODIFIER', 'LISTER')->toString(), $entity);

        return $isGrantedLister;
    }
}
