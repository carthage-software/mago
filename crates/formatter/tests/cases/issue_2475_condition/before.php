<?php

final class Exemple
{
    public function dateDelivranceDesactivee(bool $leStatutDuDossierEstInferieurAOrdreDeService): bool
    {
        if ($leStatutDuDossierEstInferieurAOrdreDeService
            || (!$leStatutDuDossierEstInferieurAOrdreDeService && $this->roleService->isAdministrateurDroits())
        ) {
            return false;
        }

        return true;
    }
}
