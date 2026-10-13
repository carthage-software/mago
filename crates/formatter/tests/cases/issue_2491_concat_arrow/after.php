<?php

final class Exemple
{
    public function comptes(Collection $comptes): string
    {
        return $comptes->map(
            fn(CompteChargeIntervenant $compteChargeIntervenant) => $compteChargeIntervenant
                ->getCompteCollectifIntervenant()
                ->getNumeroCollectif()
            . ' / '
            . $compteChargeIntervenant->getIntitule(),
        )->join(PHP_EOL);
    }
}
