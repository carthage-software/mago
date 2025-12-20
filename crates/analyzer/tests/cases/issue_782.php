<?php

declare(strict_types=1);

$list = [new DateTime()];
$first = current($list);
$first->format('d-m-Y');
