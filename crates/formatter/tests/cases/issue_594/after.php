<?php

declare(strict_types=1);

if (
    file_exists(
        dirname(__DIR__)
            . '/var/cache/prod/App_KernelProdContainer.preload.php',
    )
) {
    include_file(
        dirname(__DIR__)
            . '/var/cache/prod/App_KernelProdContainer.preload.php',
    );

    require
        dirname(__DIR__)
            . '/var/cache/prod/App_KernelProdContainer.preload.php';
}
