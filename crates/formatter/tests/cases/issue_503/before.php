<?php

function main(): void
{
    $container->registerForAutoconfiguration(CompilerPassInterface::class)
        ->addTag('container.excluded', ['source' => 'because it\'s a compiler pass']);
}
