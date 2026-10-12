<?php

final class Exemple
{
    public function aUnRolePourUneAgence(string $codeRole, Agence $agence): bool
    {
        $closure = fn(UtilisateurRoleAttribution $utilisateurRoleAttribution) => $utilisateurRoleAttribution
            ->getRole()
            ->getCode() === $codeRole
        && $utilisateurRoleAttribution->getAgences()->contains($agence);

        return !$this->getUtilisateurRoleAttribution()->filter($closure)->isEmpty();
    }
}
