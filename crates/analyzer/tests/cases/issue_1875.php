<?php

declare(strict_types=1);

final class ApiResponse
{
    public function error(): string
    {
        return 'err';
    }
}

$a = (new ApiResponse())->error();
$b = new ApiResponse()->error();
