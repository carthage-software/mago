<?php

declare(strict_types=1);

$path = "/path/to/using.php";

using ($file = fopen($path, "r")) {
   // do nothing
}