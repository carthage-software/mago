<?php

declare(strict_types=1);

final class ApiResponse
{
    public function error(): string
    {
        return 'err';
    }

    public function next(): self
    {
        return $this;
    }

    public static function statik(): string
    {
        return 'static';
    }
}

$obj = new ApiResponse;
$config = ['class' => ApiResponse::class];

$a = new ApiResponse()->error();
$b = new ApiResponse()->error()->next();
$c = new ApiResponse()
    ->error('xxxx', 'yyyy', 'zzzz', 'wwww', 'vvvv', 'uuuu', 'tttt')
    ->next();
$d = $obj->error(new ApiResponse()->error());
$e = new ApiResponse()->next()->next()->next()->error();
$f = new ApiResponse()::statik();
$g = new $config['class'];

return new ApiResponse()->error();
