<?php

declare(strict_types=1);

namespace Mago;

use Composer\Command\BaseCommand;
use Composer\Plugin\Capability\CommandProvider;

final class MagoCommandProvider implements CommandProvider
{
    /**
     * @return list<BaseCommand>
     */
    public function getCommands(): array
    {
        return [new InstallMagoBinaryCommand()];
    }
}
