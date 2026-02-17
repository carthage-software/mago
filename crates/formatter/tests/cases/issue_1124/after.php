<?php declare(strict_types = 1);

namespace App\Command;

use Doctrine\DBAL\Connection;
use Symfony\Component\Console\Style\SymfonyStyle;

final readonly class UpdateSchemaCommand
{

    private function createDatabase(Connection $connection, SymfonyStyle $io): bool // @phpstan-ignore method.unused
    {
        return true;
    }

}
