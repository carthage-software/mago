<?php declare(strict_types=1);

$hex1 = '1a2b';
$hex2 = 'ff00';

$bin1 = pack('H*', str_pad($hex1, 16, '0', STR_PAD_LEFT));
$bin2 = pack('H*', str_pad($hex2, 16, '0', STR_PAD_LEFT));

function use_int(int $x): void { echo $x; }
function use_string(string $x): void { echo $x; }

$xor = $bin1 ^ $bin2;
use_string($xor);

$bin1 = 10;
$bin2 = 15;
$xor = $bin1 ^ $bin2;
use_int($xor);

$bin1 = 10;
$bin2 = false;
$xor = $bin1 ^ $bin2;
use_int($xor);

$bin1 = 10;
$bin2 = true;
$xor = $bin1 ^ $bin2;
use_int($xor);

$bin1 = 15.23;
$bin2 = 19.17;
$xor = $bin1 ^ $bin2;
use_int($xor);
